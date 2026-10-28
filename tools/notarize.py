"""Notarize a Mac app, disk image or installer with Apple's notary service,
as `xcrun notarytool submit --wait` does, from any host.

    bazel run //tools:notarize -- --key AuthKey_ABC123.p8 --key-id ABC123 \\
        --issuer 01234567-89ab-cdef-0123-456789abcdef path/to/App.app

It authenticates with an App Store Connect API key (the .p8 file, its key ID
and its issuer ID), as notarytool's --key, --key-id and --issuer do (or
APP_STORE_CONNECT_KEY_PATH, APP_STORE_CONNECT_KEY_ID and
APP_STORE_CONNECT_ISSUER_ID), through
Apple's notary web API: it creates a submission, uploads the file to the
storage Apple names for it, waits for a decision, and prints Apple's log. An
.app is zipped first, symbolic links kept, as `ditto -c -k --keepParent`
makes it. Staple the result with //tools:staple.
"""

import argparse
import base64
import datetime
import hashlib
import hmac
import io
import json
import os
import stat
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import zipfile

API = "https://appstoreconnect.apple.com/notary/v2/submissions"

# P-256 (secp256r1)
_P = 0xFFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF
_A = _P - 3
_N = 0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551
_G = (0x6B17D1F2E12C4247F8BCE6E563A440F277037D812DEB33A0F4A13945D898C296,
      0x4FE342E2FE1A7F9B8EE7EB4A7C0F9E162BCE33576B315ECECBB6406837BF51F5)


def _add(p, q):
    if p is None:
        return q
    if q is None:
        return p
    if p[0] == q[0] and (p[1] + q[1]) % _P == 0:
        return None
    if p == q:
        m = (3 * p[0] * p[0] + _A) * pow(2 * p[1], -1, _P)
    else:
        m = (q[1] - p[1]) * pow(q[0] - p[0], -1, _P)
    x = (m * m - p[0] - q[0]) % _P
    return (x, (m * (p[0] - x) - p[1]) % _P)


def _multiply(k, point=_G):
    result = None
    while k:
        if k & 1:
            result = _add(result, point)
        point = _add(point, point)
        k >>= 1
    return result


def _rfc6979_k(d, digest):
    """The deterministic nonce of RFC 6979, for SHA-256 and P-256."""
    x = d.to_bytes(32, "big")
    h = (int.from_bytes(digest, "big") % _N).to_bytes(32, "big")
    v, k = b"\x01" * 32, b"\x00" * 32
    k = hmac.new(k, v + b"\x00" + x + h, hashlib.sha256).digest()
    v = hmac.new(k, v, hashlib.sha256).digest()
    k = hmac.new(k, v + b"\x01" + x + h, hashlib.sha256).digest()
    v = hmac.new(k, v, hashlib.sha256).digest()
    while True:
        v = hmac.new(k, v, hashlib.sha256).digest()
        candidate = int.from_bytes(v, "big")
        if 1 <= candidate < _N:
            return candidate
        k = hmac.new(k, v + b"\x00", hashlib.sha256).digest()
        v = hmac.new(k, v, hashlib.sha256).digest()


def es256(d, message):
    """An ES256 signature of message: r and s, 32 bytes each."""
    digest = hashlib.sha256(message).digest()
    z = int.from_bytes(digest, "big")
    k = _rfc6979_k(d, digest)
    r = _multiply(k)[0] % _N
    s = pow(k, -1, _N) * (z + r * d) % _N
    return r.to_bytes(32, "big") + s.to_bytes(32, "big")


def _der(b, at):
    """(tag, content start, end) of the DER element at `at`."""
    tag, first = b[at], b[at + 1]
    if first < 0x80:
        return tag, at + 2, at + 2 + first
    n = first & 0x7F
    length = int.from_bytes(b[at + 2:at + 2 + n], "big")
    return tag, at + 2 + n, at + 2 + n + length


def private_key(pem):
    """The private scalar of a PKCS#8 P-256 key, as App Store Connect issues them."""
    body = "".join(l for l in pem.splitlines() if l and not l.startswith("-----"))
    der = base64.b64decode(body)
    _, start, _ = _der(der, 0)                 # PrivateKeyInfo
    _, _, after_version = _der(der, start)     # version
    _, _, after_algorithm = _der(der, after_version)
    _, key_start, _ = _der(der, after_algorithm)   # OCTET STRING: ECPrivateKey
    _, ec_start, _ = _der(der, key_start)
    _, _, after_ec_version = _der(der, ec_start)
    tag, d_start, d_end = _der(der, after_ec_version)
    if tag != 0x04 or d_end - d_start != 32:
        raise SystemExit("the API key is not a P-256 private key")
    return int.from_bytes(der[d_start:d_end], "big")


def _b64url(b):
    return base64.urlsafe_b64encode(b).rstrip(b"=")


def token(d, key_id, issuer, now=None):
    """The JWT App Store Connect expects, valid for 20 minutes."""
    now = int(now if now is not None else time.time())
    header = {"alg": "ES256", "kid": key_id, "typ": "JWT"}
    payload = {"iss": issuer, "iat": now, "exp": now + 1200, "aud": "appstoreconnect-v1"}
    signing_input = _b64url(json.dumps(header, separators=(",", ":")).encode()) + b"." + \
        _b64url(json.dumps(payload, separators=(",", ":")).encode())
    return (signing_input + b"." + _b64url(es256(d, signing_input))).decode()


def _request(method, url, jwt=None, body=None, headers=None):
    headers = dict(headers or {})
    if jwt:
        headers["Authorization"] = "Bearer " + jwt
    if body is not None and not isinstance(body, (bytes, io.IOBase)):
        body = json.dumps(body).encode()
        headers["Content-Type"] = "application/json"
    request = urllib.request.Request(url, data=body, method=method, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=600) as response:
            data = response.read()
    except urllib.error.HTTPError as e:
        raise SystemExit("{} {}: HTTP {}: {}".format(method, url.split("?")[0], e.code, e.read()[:2000].decode("utf-8", "replace")))
    return json.loads(data) if data else {}


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
        _request("PUT", "https://{}{}".format(host, path_part), body=f, headers=request_headers)


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
    # The key's flags default to the environment, so that none of it need
    # appear on a command line.
    for flag, env, what in (("--key", "APP_STORE_CONNECT_KEY_PATH", "App Store Connect API key (.p8)"),
                            ("--key-id", "APP_STORE_CONNECT_KEY_ID", "the key's ID"),
                            ("--issuer", "APP_STORE_CONNECT_ISSUER_ID", "the key's issuer ID")):
        parser.add_argument(flag, default=os.environ.get(env), required=not os.environ.get(env),
                            help="{} (default: ${})".format(what, env))
    parser.add_argument("--timeout", type=int, default=3600, help="seconds to wait for a decision")
    parser.add_argument("path", help=".app, .zip, .dmg or .pkg")
    args = parser.parse_args()
    cwd = os.environ.get("BUILD_WORKING_DIRECTORY", os.getcwd())
    path = os.path.join(cwd, os.path.expanduser(args.path))
    key_path = os.path.join(cwd, os.path.expanduser(args.key))
    d = private_key(open(key_path).read())

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

        jwt = token(d, args.key_id, args.issuer)
        created = _request("POST", API, jwt, {"submissionName": os.path.basename(upload_path), "sha256": digest})["data"]
        submission = created["id"]
        print("submission {}: uploading {} bytes".format(submission, size), file=sys.stderr)
        upload(upload_path, digest, size, created["attributes"])

        deadline = time.time() + args.timeout
        while True:
            jwt = token(d, args.key_id, args.issuer)
            status = _request("GET", "{}/{}".format(API, submission), jwt)["data"]["attributes"]["status"]
            if status != "In Progress":
                break
            if time.time() > deadline:
                raise SystemExit("submission {}: still in progress after {}s".format(submission, args.timeout))
            time.sleep(15)
        print("submission {}: {}".format(submission, status), file=sys.stderr)
        logs = _request("GET", "{}/{}/logs".format(API, submission), token(d, args.key_id, args.issuer))
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
