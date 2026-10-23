import os
import plistlib
import struct
import sys
import unittest
import zipfile

SDK = "27.0"
XCODE = "2700"
XCODE_BUILD = "27A0"

CPU_TYPES = {0x01000007: "x86_64", 0x0100000C: "arm64", 0x0200000C: "arm64_32"}
MH_EXECUTE = 2
MH_DYLIB = 6
LC_LOAD_DYLIB = 0xC
LC_ID_DYLIB = 0xD
LC_RPATH = 0x8000001C
LC_SEGMENT_64 = 0x19
LC_BUILD_VERSION = 0x32

LIBSYSTEM = "/usr/lib/libSystem.B.dylib"
LIBSWIFTCORE = "/usr/lib/swift/libswiftCore.dylib"

EXPECTED = {
    "ios_device": dict(name="DeviceApp", sdk="iPhoneOS", platform=2, archs={"arm64"}, minos="17.0"),
    "ios_simulator": dict(name="App", sdk="iPhoneSimulator", platform=7, archs={"arm64"}, minos="17.0"),
    "macos_c": dict(name="CTool", platform=1, archs={"x86_64"}, minos="14.0", swift=False),
    "macos_swift": dict(name="SwiftTool", platform=1, archs={"arm64"}, minos="14.0", swift=True),
    "tvos_simulator": dict(name="TVApp", sdk="AppleTVSimulator", platform=8, archs={"x86_64"}, minos="17.0"),
    "visionos_simulator": dict(name="VisionApp", sdk="XRSimulator", platform=12, archs={"arm64"}, minos="1.0"),
    "watchos_simulator": dict(name="WatchApp", sdk="WatchSimulator", platform=9, archs={"arm64"}, minos="10.0"),
}


def version(value):
    major, minor, patch = value >> 16, (value >> 8) & 0xFF, value & 0xFF
    return "{}.{}".format(major, minor) + (".{}".format(patch) if patch else "")


def c_string(data, offset):
    return data[offset : data.index(b"\0", offset)].decode()


def parse_slice(data):
    magic = struct.unpack_from("<I", data)[0]
    if magic not in (0xFEEDFACE, 0xFEEDFACF):
        raise AssertionError("not a Mach-O slice: {:#x}".format(magic))
    cputype, _, filetype, ncmds, _, _ = struct.unpack_from("<iIIIII", data, 4)
    offset = 32 if magic == 0xFEEDFACF else 28
    result = dict(arch=CPU_TYPES.get(cputype, hex(cputype)), filetype=filetype, dylibs=[], rpaths=[], sections={})
    for _ in range(ncmds):
        cmd, cmdsize = struct.unpack_from("<II", data, offset)
        if cmd == LC_BUILD_VERSION:
            platform, minos, sdk = struct.unpack_from("<III", data, offset + 8)
            result.update(platform=platform, minos=version(minos), sdk=version(sdk))
        elif cmd in (LC_LOAD_DYLIB, LC_ID_DYLIB):
            name = c_string(data, offset + struct.unpack_from("<I", data, offset + 8)[0])
            if cmd == LC_ID_DYLIB:
                result["id"] = name
            else:
                result["dylibs"].append(name)
        elif cmd == LC_RPATH:
            result["rpaths"].append(c_string(data, offset + struct.unpack_from("<I", data, offset + 8)[0]))
        elif cmd == LC_SEGMENT_64:
            nsects = struct.unpack_from("<I", data, offset + 64)[0]
            for index in range(nsects):
                section = offset + 72 + index * 80
                sectname = data[section : section + 16].rstrip(b"\0").decode()
                segname = data[section + 16 : section + 32].rstrip(b"\0").decode()
                size, file_offset = struct.unpack_from("<QI", data, section + 40)
                result["sections"][segname + "," + sectname] = data[file_offset : file_offset + size]
        offset += cmdsize
    return result


def parse_macho(data):
    if struct.unpack_from(">I", data)[0] == 0xCAFEBABE:
        count = struct.unpack_from(">I", data, 4)[0]
        slices = []
        for index in range(count):
            _, _, offset, size, _ = struct.unpack_from(">iIIII", data, 8 + index * 20)
            slices.append(parse_slice(data[offset : offset + size]))
        return slices
    return [parse_slice(data)]


def bundle_files(path):
    if os.path.isdir(path):
        files = {}
        parent = os.path.dirname(path)
        for directory, _, names in os.walk(path):
            for name in names:
                full = os.path.join(directory, name)
                with open(full, "rb") as f:
                    files[os.path.relpath(full, parent)] = f.read()
        return files
    with zipfile.ZipFile(path) as archive:
        return {name: archive.read(name) for name in archive.namelist() if not name.endswith("/")}


class SdkFreeTest(unittest.TestCase):
    outputs = {}

    def check_binary(self, slices, expected, filetype):
        self.assertEqual({s["arch"] for s in slices}, expected["archs"])
        for s in slices:
            self.assertEqual(s["filetype"], filetype)
            self.assertEqual(s["platform"], expected["platform"])
            self.assertEqual(s["minos"], expected["minos"])
            self.assertEqual(s["sdk"], SDK)
            self.assertIn(LIBSYSTEM, s["dylibs"])

    def check_info_plist(self, plist, expected):
        self.assertEqual(plist["CFBundleExecutable"], expected["name"])
        self.assertEqual(plist["MinimumOSVersion"], expected["minos"])
        self.assertEqual(plist["CFBundleSupportedPlatforms"], [expected["sdk"]])
        self.assertEqual(plist["DTPlatformName"], expected["sdk"].lower())
        self.assertEqual(plist["DTSDKName"], expected["sdk"].lower() + SDK)
        self.assertEqual(plist["DTPlatformVersion"], SDK)
        self.assertEqual(plist["DTXcode"], XCODE)
        self.assertEqual(plist["DTXcodeBuild"], XCODE_BUILD)

    def check_app(self, key):
        expected = EXPECTED[key]
        files = bundle_files(self.outputs[key])
        prefix = next(
            name[: -len("Info.plist")]
            for name in files
            if name.endswith(".app/Info.plist") and name.count(".app/") == 1
        )
        self.check_info_plist(plistlib.loads(files[prefix + "Info.plist"]), expected)
        slices = parse_macho(files[prefix + expected["name"]])
        self.check_binary(slices, expected, MH_EXECUTE)
        for s in slices:
            self.assertIn(LIBSWIFTCORE, s["dylibs"])
        return prefix, files, slices

    def test_ios_device(self):
        self.check_app("ios_device")

    def test_ios_simulator(self):
        prefix, files, slices = self.check_app("ios_simulator")
        self.assertEqual(files[prefix + "Greeting.txt"], b"Hello from a resource\n")
        self.assertEqual(files[prefix + "en.lproj/Localizable.strings"], b'"greeting" = "Hello";\n')
        framework = prefix + "Frameworks/Greeter.framework/"
        self.assertEqual(plistlib.loads(files[framework + "Info.plist"])["CFBundlePackageType"], "FMWK")
        framework_slices = parse_macho(files[framework + "Greeter"])
        self.check_binary(framework_slices, EXPECTED["ios_simulator"], MH_DYLIB)
        for s in framework_slices:
            self.assertEqual(s["id"], "@rpath/Greeter.framework/Greeter")
        for s in slices:
            self.assertIn("@rpath/Greeter.framework/Greeter", s["dylibs"])
            self.assertIn("@executable_path/Frameworks", s["rpaths"])

    def check_tool(self, key):
        expected = EXPECTED[key]
        with open(self.outputs[key], "rb") as f:
            slices = parse_macho(f.read())
        self.check_binary(slices, expected, MH_EXECUTE)
        for s in slices:
            self.assertEqual(LIBSWIFTCORE in s["dylibs"], expected["swift"])
            plist = plistlib.loads(s["sections"]["__TEXT,__info_plist"])
            self.assertTrue(plist["CFBundleIdentifier"].startswith("com.example.stub-sdks."))

    def test_macos_c(self):
        self.check_tool("macos_c")

    def test_macos_swift(self):
        self.check_tool("macos_swift")

    def test_tvos_simulator(self):
        self.check_app("tvos_simulator")

    def test_visionos_simulator(self):
        self.check_app("visionos_simulator")

    def test_watchos_simulator(self):
        self.check_app("watchos_simulator")


if __name__ == "__main__":
    for argument in sys.argv[1:]:
        key, _, path = argument.partition("=")
        SdkFreeTest.outputs[key] = os.path.join(os.environ.get("TEST_SRCDIR", ""), os.environ.get("TEST_WORKSPACE", ""), path)
    unittest.main(argv=sys.argv[:1], verbosity=2)
