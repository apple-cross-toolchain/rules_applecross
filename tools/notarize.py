"""Notarize a Mac app, disk image or installer with Apple's notary service,
as `xcrun notarytool submit --wait` does, from any host.

    bazel run //tools:notarize -- --key AuthKey_ABC123.p8 --key-id ABC123 \\
        --issuer 01234567-89ab-cdef-0123-456789abcdef path/to/App.app

It authenticates with an App Store Connect API key (the .p8 file, its key ID
and its issuer ID), as notarytool's --key, --key-id and --issuer do, through
Apple's notary web API: it creates a submission, uploads the file to the
storage Apple names for it, waits for a decision, and prints Apple's log. An
.app is zipped first, symbolic links kept, as `ditto -c -k --keepParent`
makes it. Staple the result with //tools:staple.
"""

import argparse
import datetime
import hashlib
import hmac
import json
import os
import stat
import sys
import time
import urllib.parse
import urllib.request
import zipfile

from app_store_connect import Client, add_key_arguments, request

API = "https://appstoreconnect.apple.com/notary/v2/submissions"

def _sign_v4(key, msg):
    return hmac.new(key, msg.encode(), hashlib.sha256).digest()


def upload(path, digest, size, credentials):
    """PUTs the file into the S3 object the submission names, signed with
    AWS Signature Version 4 and the temporary credentials Apple issued."""
    bucket, key = credentials["bucket"], credentials["object"]
    region = "us-west-2"
    host = "{}.s3.{}.amazonaws.com".format(bucket, region)
    now = datetime.datetime.now(datetime.timezone.utc)
    amz_date, day = now.strftime("%Y%m%dT%H%M%SZ"), now.strftime("%Y%m%d")
    path_part = "/" + urllib.parse.quote(key, safe="/-_.~")
    headers = {
        "host": host,
        "x-amz-content-sha256": digest,
        "x-amz-date": amz_date,
        "x-amz-security-token": credentials["awsSessionToken"],
    }
    signed = ";".join(sorted(headers))
    canonical = "\n".join(["PUT", path_part, "", "".join("{}:{}\n".format(k, headers[k]) for k in sorted(headers)), signed, digest])
    scope = "{}/{}/s3/aws4_request".format(day, region)
    to_sign = "\n".join(["AWS4-HMAC-SHA256", amz_date, scope, hashlib.sha256(canonical.encode()).hexdigest()])
    k = _sign_v4(("AWS4" + credentials["awsSecretAccessKey"]).encode(), day)
    for part in (region, "s3", "aws4_request"):
        k = _sign_v4(k, part)
    signature = hmac.new(k, to_sign.encode(), hashlib.sha256).hexdigest()
    request_headers = {k: v for k, v in headers.items() if k != "host"}
    request_headers["Authorization"] = "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}".format(
        credentials["awsAccessKeyId"], scope, signed, signature)
    request_headers["Content-Length"] = str(size)
    with open(path, "rb") as f:
        request("PUT", "https://{}{}".format(host, path_part), body=f, headers=request_headers, answers_json=False)


def zip_app(app, out):
    """App.app zipped with its parent directory kept and symbolic links stored
    as links, as `ditto -c -k --keepParent` makes it."""
    parent = os.path.dirname(os.path.abspath(app))
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        for root, dirs, files in os.walk(app, followlinks=False):
            for name in sorted(dirs) + sorted(files):
                full = os.path.join(root, name)
                rel = os.path.relpath(full, parent)
                st = os.lstat(full)
                if stat.S_ISLNK(st.st_mode):
                    info = zipfile.ZipInfo(rel)
                    info.create_system = 3
                    info.external_attr = (stat.S_IFLNK | 0o755) << 16
                    z.writestr(info, os.readlink(full))
                    if name in dirs:
                        dirs.remove(name)
                elif stat.S_ISDIR(st.st_mode):
                    info = zipfile.ZipInfo(rel + "/")
                    info.create_system = 3
                    info.external_attr = (stat.S_IFDIR | 0o755) << 16
                    z.writestr(info, b"")
                else:
                    z.write(full, rel)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    add_key_arguments(parser)
    parser.add_argument("--timeout", type=int, default=3600, help="seconds to wait for a decision")
    parser.add_argument("path", help=".app, .zip, .dmg or .pkg")
    args = parser.parse_args()
    cwd = os.environ.get("BUILD_WORKING_DIRECTORY", os.getcwd())
    path = os.path.join(cwd, os.path.expanduser(args.path))
    api = Client(args)

    upload_path = path
    if path.rstrip("/").endswith(".app"):
        upload_path = os.path.join(os.path.dirname(path.rstrip("/")), os.path.basename(path.rstrip("/")) + ".notarize.zip")
        zip_app(path.rstrip("/"), upload_path)
    try:
        h = hashlib.sha256()
        with open(upload_path, "rb") as f:
            for block in iter(lambda: f.read(1 << 20), b""):
                h.update(block)
        digest, size = h.hexdigest(), os.path.getsize(upload_path)

        created = api("POST", API, {"submissionName": os.path.basename(upload_path), "sha256": digest})["data"]
        submission = created["id"]
        print("submission {}: uploading {} bytes".format(submission, size), file=sys.stderr)
        upload(upload_path, digest, size, created["attributes"])

        deadline = time.time() + args.timeout
        while True:
            status = api("GET", "{}/{}".format(API, submission))["data"]["attributes"]["status"]
            if status != "In Progress":
                break
            if time.time() > deadline:
                raise SystemExit("submission {}: still in progress after {}s".format(submission, args.timeout))
            time.sleep(15)
        print("submission {}: {}".format(submission, status), file=sys.stderr)
        logs = api("GET", "{}/{}/logs".format(API, submission))
        url = logs.get("data", {}).get("attributes", {}).get("developerLogUrl")
        if url:
            with urllib.request.urlopen(url, timeout=120) as response:
                print(json.dumps(json.load(response), indent=2))
        return 0 if status == "Accepted" else 1
    finally:
        if upload_path != path and os.path.exists(upload_path):
            os.remove(upload_path)


if __name__ == "__main__":
    sys.exit(main())
