#!/usr/bin/env python3

import hashlib
import importlib.util
import os
import plistlib
import struct
import tempfile
import unittest
from pathlib import Path
from unittest import mock


_RUNNER_PATH = Path(__file__).with_name("ios_vm_runner.py")
_SPEC = importlib.util.spec_from_file_location("ios_vm_runner", _RUNNER_PATH)
runner = importlib.util.module_from_spec(_SPEC)
assert _SPEC.loader
_SPEC.loader.exec_module(runner)


def _minimal_macho() -> bytes:
    header = struct.pack(
        "<8I",
        0xFEEDFACF,
        0x0100000C,
        0,
        8,  # MH_BUNDLE
        2,
        144,
        0,
        0,
    )
    text = struct.pack(
        "<II16sQQQQiiII",
        0x19,
        72,
        b"__TEXT".ljust(16, b"\0"),
        0,
        0x4000,
        0,
        0x200,
        5,
        5,
        0,
        0,
    )
    linkedit = struct.pack(
        "<II16sQQQQiiII",
        0x19,
        72,
        b"__LINKEDIT".ljust(16, b"\0"),
        0x4000,
        0x4000,
        0x200,
        0x20,
        1,
        1,
        0,
        0,
    )
    return (header + text + linkedit).ljust(0x220, b"\0")


class IosVmRunnerTest(unittest.TestCase):
    def test_unconditional_uikit_load_is_removed(self):
        framework_path = b"/System/Library/Frameworks/UIKit.framework/UIKit\0"
        command_size = (24 + len(framework_path) + 7) & ~7
        data = bytearray(struct.pack(
            "<8I",
            0xFEEDFACF,
            0x0100000C,
            0,
            8,
            2,
            command_size + 48,
            0,
            0,
        ))
        data.extend(struct.pack(
            "<6I",
            runner._LC_LOAD_DYLIB,
            command_size,
            24,
            0,
            0,
            0,
        ))
        data.extend(framework_path)
        data.extend(b"\0" * (32 + command_size - len(data)))
        data.extend(struct.pack("<12I", runner._LC_DYLD_INFO_ONLY, 48, *([0] * 10)))

        # This synthetic image has no bind stream because it imports no symbols.
        runner._remove_unavailable_dylibs(data)

        self.assertEqual(struct.unpack_from("<II", data, 16), (1, 48))

    def test_adhoc_sign_for_slot(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "input"
            output = Path(temporary) / "output"
            source.write_bytes(_minimal_macho())

            cdhash = runner.adhoc_sign_for_slot(source, output, 32768, "Test")
            data = bytearray(output.read_bytes())
            self.assertEqual(len(data), 32768)

            _, _, _, code_signature, _, _, _ = runner._load_commands(data)
            self.assertIsNotNone(code_signature)
            signature_offset, signature_size = struct.unpack_from(
                "<II",
                data,
                code_signature + 8,
            )
            self.assertEqual(signature_offset + signature_size, len(data))

            signature = data[signature_offset:]
            magic, superblob_length, count = struct.unpack_from(">III", signature, 0)
            self.assertEqual(magic, 0xFADE0CC0)
            self.assertEqual(count, 3)
            code_directory_offset = struct.unpack_from(">I", signature, 16)[0]
            code_directory_length = struct.unpack_from(
                ">I",
                signature,
                code_directory_offset + 4,
            )[0]
            code_directory = signature[
                code_directory_offset:code_directory_offset + code_directory_length
            ]
            self.assertEqual(cdhash, hashlib.sha256(code_directory).digest()[:20])
            self.assertEqual(struct.unpack_from(">I", code_directory, 12)[0], 2)
            self.assertEqual(code_directory[37], 2)
            self.assertEqual(code_directory[39], 14)
            self.assertLessEqual(superblob_length, signature_size)

    def test_merge_trust_cache(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary) / "base.tc"
            output = Path(temporary) / "output.tc"
            uuid = b"A" * 16
            existing = bytes.fromhex("11" * 20) + bytes((2, 0))
            base.write_bytes(struct.pack("<I16sI", 1, uuid, 1) + existing)
            added = bytes.fromhex("00" * 20)

            runner.merge_trust_cache(base, output, added)
            data = output.read_bytes()
            version, actual_uuid, count = struct.unpack_from("<I16sI", data, 0)
            self.assertEqual((version, actual_uuid, count), (1, uuid, 2))
            self.assertEqual(data[24:46], added + bytes((2, 0)))

    def test_locate_test_executable_rejects_resources(self):
        with tempfile.TemporaryDirectory() as temporary:
            bundle = Path(temporary) / "Tests.xctest"
            bundle.mkdir()
            (bundle / "Info.plist").write_bytes(plistlib.dumps({
                "CFBundleExecutable": "Tests",
            }))
            (bundle / "Tests").write_bytes(_minimal_macho())
            (bundle / "Fixture.txt").write_text("fixture")
            with self.assertRaisesRegex(runner.RunnerError, "resources"):
                runner.locate_test_executable(bundle, Path(temporary))

    def test_filter_combines_rule_and_bazel_values(self):
        with mock.patch.dict(os.environ, {"TESTBRIDGE_TEST_ONLY": "Suite/testTwo"}):
            self.assertEqual(
                runner.test_filter("Suite/testOne"),
                "Suite/testOne,Suite/testTwo",
            )

    def test_filter_rejects_negative_values(self):
        with mock.patch.dict(os.environ, {}, clear=True):
            with self.assertRaisesRegex(runner.RunnerError, "negative"):
                runner.test_filter("-Suite/testOne")

    def test_lowercase_nested_panic_is_recognized(self):
        console = object.__new__(runner.QemuConsole)
        console.window = bytearray(b"nested panic count exceeds limit 5")
        self.assertTrue(console.has_panic())

    def test_process_arguments_are_shell_quoted(self):
        self.assertEqual(
            runner.test_arguments(["plain", "two words", "a'b"]),
            "plain 'two words' 'a'\"'\"'b'",
        )

    def test_process_arguments_reject_newlines(self):
        with self.assertRaisesRegex(runner.RunnerError, "control characters"):
            runner.test_arguments(["first\nsecond"])

    def test_manifest_binds_all_inputs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = {}
            digests = {}
            for name in ("bootkc", "device_tree", "ramdisk", "trust_cache"):
                path = root / name
                path.write_bytes(name.encode())
                inputs[name] = path
                digests[name] = hashlib.sha256(name.encode()).hexdigest()
            manifest = root / "slot.json"
            manifest.write_text(__import__("json").dumps({
                "architecture": "arm64",
                "input_sha256": digests,
                "offset": 4096,
                "placeholder_sha256": "00" * 32,
                "size": 4096,
                "version": 2,
            }))

            loaded = runner.load_slot_manifest(manifest, inputs)
            self.assertEqual(loaded["size"], 4096)

            inputs["ramdisk"].write_bytes(b"changed")
            with self.assertRaisesRegex(runner.RunnerError, "ramdisk does not match"):
                runner.load_slot_manifest(manifest, inputs)

    def test_manifest_rejects_unknown_version(self):
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / "slot.json"
            manifest.write_text('{"version": 1}')
            with self.assertRaisesRegex(runner.RunnerError, "version 2"):
                runner.load_slot_manifest(manifest, {})

    def test_bundle_rejects_executable_traversal(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            bundle = root / "Tests.xctest"
            bundle.mkdir()
            (bundle / "Info.plist").write_bytes(plistlib.dumps({
                "CFBundleExecutable": "../Tests",
            }))
            (root / "Tests").write_bytes(_minimal_macho())
            with self.assertRaisesRegex(runner.RunnerError, "must be a file name"):
                runner.locate_test_executable(bundle, root)


if __name__ == "__main__":
    unittest.main()
