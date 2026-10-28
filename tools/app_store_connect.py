"""App Store Connect's API, as notarize and upload use it: an API key (the
.p8 file App Store Connect issues, its key ID and its issuer ID) signs a JWT
for each request, as notarytool's and altool's --key, --key-id and --issuer
do.

Each tool takes the key with --key, --key-id and --issuer, or from
APP_STORE_CONNECT_KEY_PATH, APP_STORE_CONNECT_KEY_ID and
APP_STORE_CONNECT_ISSUER_ID.
"""

import base64
import hashlib
import hmac
import io
import json
import os
import time
import urllib.error
import urllib.request

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


def request(method, url, jwt=None, body=None, headers=None, answers_json=True):
    """The JSON a request answers with ({} for none, or when `answers_json`
    is false, as for a file upload); an HTTP error exits, naming the URL
    without its query, which may carry credentials. Apple's notary service
    answers JSON as application/octet-stream, so the caller says which."""
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
    return json.loads(data) if data and answers_json else {}


def add_key_arguments(parser):
    """--key, --key-id and --issuer, defaulting to the environment's."""
    for flag, env, what in (("--key", "APP_STORE_CONNECT_KEY_PATH", "App Store Connect API key (.p8)"),
                            ("--key-id", "APP_STORE_CONNECT_KEY_ID", "the key's ID"),
                            ("--issuer", "APP_STORE_CONNECT_ISSUER_ID", "the key's issuer ID")):
        parser.add_argument(flag, default=os.environ.get(env), required=not os.environ.get(env),
                            help="{} (default: ${})".format(what, env))


class Client:
    """Requests to App Store Connect, each with a fresh token."""

    def __init__(self, args):
        cwd = os.environ.get("BUILD_WORKING_DIRECTORY", os.getcwd())
        with open(os.path.join(cwd, os.path.expanduser(args.key))) as f:
            self._d = private_key(f.read())
        self._key_id = args.key_id
        self._issuer = args.issuer

    def __call__(self, method, url, body=None):
        return request(method, url, token(self._d, self._key_id, self._issuer), body)
