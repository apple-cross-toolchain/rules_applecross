#!/usr/bin/env python3
"""Minimal Info-ZIP `zip` replacement for executor images without one.

Supports the invocation rules_apple's process-and-sign script uses:
  zip -qX --symlinks -@ --compression-method store|deflate ARCHIVE
with the entry list supplied on stdin, one path per line.
"""

import os
import stat
import sys
import time
import zipfile


def _dos_time(mtime):
    t = time.localtime(mtime)
    year = max(t.tm_year, 1980)
    return (year, t.tm_mon, t.tm_mday, t.tm_hour, t.tm_min, t.tm_sec)


def main(argv):
    compression = zipfile.ZIP_DEFLATED
    read_stdin = False
    archive = None
    i = 0
    while i < len(argv):
        arg = argv[i]
        if arg == "--compression-method":
            i += 1
            compression = {
                "store": zipfile.ZIP_STORED,
                "deflate": zipfile.ZIP_DEFLATED,
            }[argv[i]]
        elif arg == "-@":
            read_stdin = True
        elif arg.startswith("-"):
            # -q, -X, --symlinks: always-on behaviors of this implementation.
            pass
        else:
            archive = arg
        i += 1

    if not archive:
        print("zip: no archive name given", file=sys.stderr)
        return 1

    paths = []
    if read_stdin:
        for line in sys.stdin:
            path = line.rstrip("\n")
            if path:
                paths.append(path)

    seen = set()
    with zipfile.ZipFile(archive, "w", allowZip64=True) as zf:
        for path in paths:
            # Info-ZIP strips a leading "./" from stored names.
            name = path[2:] if path.startswith("./") else path
            if not name or name == "." or name in seen:
                continue
            seen.add(name)
            st = os.lstat(path)
            if stat.S_ISLNK(st.st_mode):
                info = zipfile.ZipInfo(name, date_time=_dos_time(st.st_mtime))
                info.create_system = 3
                info.external_attr = (st.st_mode & 0xFFFF) << 16
                zf.writestr(info, os.readlink(path), zipfile.ZIP_STORED)
            else:
                zf.write(path, name, compress_type=compression)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
