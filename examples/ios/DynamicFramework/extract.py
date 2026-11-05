"""extract.py ARCHIVE OUT_DIR ENTRY...: writes the named entries of a zip
archive under OUT_DIR, keeping their paths and executable bits."""

import os
import sys
import zipfile


def main(archive, out_dir, *entries):
    with zipfile.ZipFile(archive) as z:
        for entry in entries:
            path = os.path.join(out_dir, entry)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "wb") as f:
                f.write(z.read(entry))
            if (z.getinfo(entry).external_attr >> 16) & 0o111:
                os.chmod(path, 0o755)


if __name__ == "__main__":
    main(*sys.argv[1:])
