"""Upload an iOS, tvOS or visionOS app to App Store Connect, for TestFlight
and the App Store, as `xcrun altool --upload-package` or Transporter does,
from any host.

    bazel run //tools:upload -- --key AuthKey_ABC123.p8 --key-id ABC123 \\
        --issuer 01234567-89ab-cdef-0123-456789abcdef path/to/App.ipa

It uses App Store Connect's build uploads API: it finds the app whose bundle
identifier the .ipa's app carries, creates a build upload for its version and
build number, reserves the file, sends it in the parts App Store Connect
names, commits it with its MD5, and waits until App Store Connect has
accepted or refused it. The app must already exist in App Store Connect and
be signed for distribution (Apple Distribution and an App Store profile).
"""

import argparse
import hashlib
import os
import plistlib
import sys
import time
import urllib.parse
import zipfile

from app_store_connect import Client, add_key_arguments, request

API = "https://api.appstoreconnect.apple.com/v1"

# The platform App Store Connect files a build under, by the app's
# CFBundleSupportedPlatforms.
PLATFORMS = {
    "iPhoneOS": "IOS",
    "AppleTVOS": "TV_OS",
    "XROS": "VISION_OS",
}


def app_info(ipa):
    """The Info.plist of the app in an .ipa's Payload."""
    with zipfile.ZipFile(ipa) as z:
        names = [n for n in z.namelist() if n.count("/") == 2 and n.startswith("Payload/") and n.endswith(".app/Info.plist")]
        if len(names) != 1:
            raise SystemExit("{}: expected one Payload/*.app/Info.plist, found {}".format(ipa, names or "none"))
        return plistlib.loads(z.read(names[0]))


def platform(info):
    for supported in info.get("CFBundleSupportedPlatforms", []):
        if supported in PLATFORMS:
            return PLATFORMS[supported]
    raise SystemExit("the app is for {}, which App Store Connect does not take as an .ipa".format(
        info.get("CFBundleSupportedPlatforms")))


def md5(path):
    h = hashlib.md5()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def send(path, operations):
    """PUTs (or whatever each operation says) each part of the file."""
    with open(path, "rb") as f:
        for i, op in enumerate(operations):
            f.seek(op.get("offset", 0))
            part = f.read(op["length"])
            headers = {h["name"]: h["value"] for h in op.get("requestHeaders", [])}
            request(op.get("method", "PUT"), op["url"], body=part, headers=headers, answers_json=False)
            print("sent part {} of {} ({} bytes)".format(i + 1, len(operations), len(part)), file=sys.stderr)


def describe(details):
    return "; ".join("{}: {}".format(d.get("code"), d.get("description")) for d in details or [])


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    add_key_arguments(parser)
    parser.add_argument("--timeout", type=int, default=3600, help="seconds to wait for App Store Connect to accept the build")
    parser.add_argument("path", help="the .ipa")
    args = parser.parse_args()
    path = os.path.join(os.environ.get("BUILD_WORKING_DIRECTORY", os.getcwd()), os.path.expanduser(args.path))
    if not path.endswith(".ipa"):
        raise SystemExit("{}: only .ipa files are supported".format(path))
    api = Client(args)

    info = app_info(path)
    bundle_id = info["CFBundleIdentifier"]
    apps = api("GET", "{}/apps?{}".format(API, urllib.parse.urlencode({"filter[bundleId]": bundle_id, "fields[apps]": "bundleId"})))["data"]
    # filter[bundleId] also matches identifiers that begin with it.
    apps = [a for a in apps if a["attributes"]["bundleId"] == bundle_id]
    if not apps:
        raise SystemExit("App Store Connect has no app with bundle identifier {}: create it there first".format(bundle_id))

    upload = api("POST", API + "/buildUploads", {"data": {
        "type": "buildUploads",
        "attributes": {
            "cfBundleShortVersionString": info["CFBundleShortVersionString"],
            "cfBundleVersion": info["CFBundleVersion"],
            "platform": platform(info),
        },
        "relationships": {"app": {"data": {"type": "apps", "id": apps[0]["id"]}}},
    }})["data"]
    print("build upload {}: {} {} ({})".format(upload["id"], bundle_id, info["CFBundleShortVersionString"], info["CFBundleVersion"]), file=sys.stderr)

    size = os.path.getsize(path)
    reserved = api("POST", API + "/buildUploadFiles", {"data": {
        "type": "buildUploadFiles",
        "attributes": {"assetType": "ASSET", "fileName": os.path.basename(path), "fileSize": size, "uti": "com.apple.ipa"},
        "relationships": {"buildUpload": {"data": {"type": "buildUploads", "id": upload["id"]}}},
    }})["data"]
    send(path, reserved["attributes"]["uploadOperations"])
    api("PATCH", "{}/buildUploadFiles/{}".format(API, reserved["id"]), {"data": {
        "type": "buildUploadFiles",
        "id": reserved["id"],
        "attributes": {"uploaded": True, "sourceFileChecksums": {"file": {"hash": md5(path), "algorithm": "MD5"}}},
    }})

    deadline = time.time() + args.timeout
    while True:
        state = api("GET", "{}/buildUploads/{}".format(API, upload["id"]))["data"]["attributes"]["state"]
        if state.get("state") in ("COMPLETE", "FAILED"):
            break
        if time.time() > deadline:
            raise SystemExit("build upload {}: still {} after {}s".format(upload["id"], state.get("state"), args.timeout))
        time.sleep(15)
    for kind in ("infos", "warnings", "errors"):
        if state.get(kind):
            print("{}: {}".format(kind, describe(state[kind])), file=sys.stderr)
    print("build upload {}: {}".format(upload["id"], state["state"]), file=sys.stderr)
    return 0 if state["state"] == "COMPLETE" else 1


if __name__ == "__main__":
    sys.exit(main())
