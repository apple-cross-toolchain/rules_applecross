#!/bin/bash
#
# Copyright 2015 The Bazel Authors. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# xcrunwrapper runs the command passed to it using xcrun. The first arg
# passed is the name of the tool to be invoked via xcrun. (For example, libtool
# or clang).
# xcrunwrapper replaces __BAZEL_XCODE_DEVELOPER_DIR__ with $DEVELOPER_DIR (or
# reasonable default) and __BAZEL_XCODE_SDKROOT__ with a valid path based on
# SDKROOT (or reasonable default).
# These values (__BAZEL_XCODE_*) are a shared secret withIosSdkCommands.java.

set -eu

if [[ "$OSTYPE" == "darwin"* ]]; then
  # A simple implementation of the realpath utility:
  # http://www.gnu.org/software/coreutils/manual/html_node/realpath-invocation.html
  # since macOS does not have anything equivalent.
  # Returns the actual path even for non symlinks and multi-level symlinks.
  function realpath() {
    local previous="$1"
    local next=$(readlink "${previous}")
    while [ -n "${next}" ]; do
      previous="${next}"
      next=$(readlink "${previous}")
    done
    echo "${previous}"
  }
fi

TOOLNAME=$1
shift

# This is equivalent to spawning `xcode-select -p` and getting its result, but
# 4 times faster.
if [[ -z "${DEVELOPER_DIR:-}" ]] ; then
  DEVELOPER_DIR="$(realpath /var/db/xcode_select_link)"
fi
export DEVELOPER_DIR

# Construct the path to the SDK root directory. This is less future-proof than
# querying it with `xcrun --sdk <sdk name> --show-sdk-path`, but invoking that
# command everytime is expensive, and it's unlikely that the location of SDKs
# inside Xcode is going to change soon.
SDKROOT="${DEVELOPER_DIR}/Platforms/${APPLE_SDK_PLATFORM}.platform/Developer/SDKs/${APPLE_SDK_PLATFORM}${APPLE_SDK_VERSION_OVERRIDE}.sdk"

# Subsitute toolkit path placeholders.
UPDATED_ARGS=()
for ARG in "$@" ; do
  ARG="${ARG//__BAZEL_XCODE_DEVELOPER_DIR__/${DEVELOPER_DIR}}"
  ARG="${ARG//__BAZEL_XCODE_SDKROOT__/${SDKROOT}}"
  UPDATED_ARGS+=("${ARG}")
done

# Workaround for an xcrun bug
unset SDKROOT

/usr/bin/xcrun "${TOOLNAME}" "${UPDATED_ARGS[@]}"
