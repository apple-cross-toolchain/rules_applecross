#!/bin/bash

set -euo pipefail

# Downloads a pre-packaged Apple SDK archive to the workspace root, where
# MODULE.bazel's `apple_sdk_path` expects to find it.
#
# On macOS `tools/package-sdks.sh` produces that archive locally. Machines
# without macOS (CI included) fetch a pre-built one with this script instead,
# so the checked-in MODULE.bazel works unmodified in both cases.
#
# Environment:
#   SDKS_URL      Required. URL of the packaged Apple SDK archive.
#   SDKS_TOKEN    Optional. Credential for private hosts. Sent as
#                 "Authorization: token" for GitHub API asset URLs and as
#                 "Authorization: Bearer" otherwise, matching what the
#                 repository rule does with ~/.netrc. Falls back to
#                 SDKS_PASSWORD.

SCRIPT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_ROOT/.." && pwd)"

if [[ -z "${SDKS_URL:-}" ]]; then
  echo "error: SDKS_URL must point at a packaged Apple SDK archive" >&2
  exit 1
fi

# MODULE.bazel is the single source of truth for the archive name; nothing
# here needs to be kept in sync with it by hand.
SDK_ARCHIVE="$(sed -n 's/^[[:space:]]*apple_sdk_path = "\([^"]*\)".*/\1/p' "$PROJECT_ROOT/MODULE.bazel" | head -1)"
if [[ -z "$SDK_ARCHIVE" ]]; then
  echo "error: no apple_sdk_path found in $PROJECT_ROOT/MODULE.bazel" >&2
  exit 1
fi

DESTINATION="$PROJECT_ROOT/$SDK_ARCHIVE"
if [[ -e "$DESTINATION" ]]; then
  echo "$SDK_ARCHIVE is already present; skipping download"
  exit 0
fi

TOKEN="${SDKS_TOKEN:-${SDKS_PASSWORD:-}}"
CURL_ARGS=(
  --fail
  --location
  --silent
  --show-error
  --retry 3
  --retry-connrefused
)

# GitHub's web release URLs ignore Authorization headers on private repos and
# return 404 even for a valid token. When we have a token, resolve such a URL
# through the release API to the asset's api.github.com URL, which does honor
# token auth. Without a token the direct URL is left alone (works for public
# repos). Handles both the per-tag and the always-latest forms:
#   github.com/OWNER/REPO/releases/download/TAG/NAME
#   github.com/OWNER/REPO/releases/latest/download/NAME
RELEASE_URL=""
ASSET_NAME=""
if [[ -n "$TOKEN" ]]; then
  if [[ "$SDKS_URL" =~ ^https://github\.com/([^/]+)/([^/]+)/releases/download/([^/]+)/([^/]+)$ ]]; then
    RELEASE_URL="https://api.github.com/repos/${BASH_REMATCH[1]}/${BASH_REMATCH[2]}/releases/tags/${BASH_REMATCH[3]}"
    ASSET_NAME="${BASH_REMATCH[4]}"
  elif [[ "$SDKS_URL" =~ ^https://github\.com/([^/]+)/([^/]+)/releases/latest/download/([^/]+)$ ]]; then
    RELEASE_URL="https://api.github.com/repos/${BASH_REMATCH[1]}/${BASH_REMATCH[2]}/releases/latest"
    ASSET_NAME="${BASH_REMATCH[3]}"
  fi
fi
if [[ -n "$RELEASE_URL" ]]; then
  RELEASE_JSON="$(mktemp)"
  trap 'rm -f "$RELEASE_JSON"' EXIT
  # --fail-with-body keeps GitHub's error JSON so auth failures are
  # diagnosable from the log; --location follows the 301 GitHub answers for
  # renamed repositories (same-host, so curl keeps the Authorization header).
  if ! curl --fail-with-body --location --silent --show-error \
    --retry 3 --retry-connrefused \
    --header "Authorization: Bearer $TOKEN" \
    --output "$RELEASE_JSON" \
    "$RELEASE_URL"; then
    echo "error: failed to resolve $RELEASE_URL; check that SDKS_TOKEN grants read access to the repository. GitHub said:" >&2
    cat "$RELEASE_JSON" >&2
    echo >&2
    exit 1
  fi
  SDKS_URL="$(python3 -c '
import json, sys
name = sys.argv[1]
release = json.load(sys.stdin)
for asset in release.get("assets", []):
    if asset["name"] == name:
        print(asset["url"])
        break
else:
    sys.exit("error: release %s has no asset named %s"
             % (release.get("tag_name"), name))
' "$ASSET_NAME" < "$RELEASE_JSON")"
  rm -f "$RELEASE_JSON"
  trap - EXIT
fi

if [[ "$SDKS_URL" == *"api.github.com"* ]]; then
  # GitHub serves release asset metadata as JSON without this.
  CURL_ARGS+=(--header "Accept: application/octet-stream")
  if [[ -n "$TOKEN" ]]; then
    CURL_ARGS+=(--header "Authorization: token $TOKEN")
  fi
elif [[ -n "$TOKEN" ]]; then
  CURL_ARGS+=(--header "Authorization: Bearer $TOKEN")
fi

# Download to a temporary file so an interrupted run never leaves a truncated
# archive behind for the repository rule to choke on.
TMP_DESTINATION="$(mktemp "$DESTINATION.XXXXXX")"
trap 'rm -f "$TMP_DESTINATION"' EXIT

echo "Downloading $SDK_ARCHIVE"
curl "${CURL_ARGS[@]}" --output "$TMP_DESTINATION" "$SDKS_URL"

# A credential or Accept header problem can still yield a 200 with a short
# error document; catch that here rather than in the middle of extraction.
SIZE="$(( $(wc -c < "$TMP_DESTINATION") ))"
if (( SIZE < 1048576 )); then
  echo "error: downloaded archive is only $SIZE bytes; SDKS_URL did not serve an SDK archive" >&2
  exit 1
fi

# mktemp creates the file 0600; keep the archive readable like any other
# checked-out input.
chmod 644 "$TMP_DESTINATION"
mv "$TMP_DESTINATION" "$DESTINATION"
trap - EXIT
echo "Wrote $DESTINATION ($SIZE bytes)"
