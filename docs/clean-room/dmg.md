# dmg

`dmg create --volname NAME --srcfolder DIR [--format UDZO|ULFO|UDRO] OUT.dmg`
writes a Mac disk image on any platform, where a Mac runs
`hdiutil create -srcfolder DIR -volname NAME -format UDZO OUT.dmg`. It is the
image step of a release made off a Mac: the app is signed, notarized and
stapled first, and the image after it (rcodesign signs, notarizes and staples
both).

- **The volume** is HFS+ (`tools/apple_tools/crates/dmg/src/hfs.rs`), written
  from Apple's published specification, Technical Note TN1150 ("HFS Plus
  Volume Format"): the volume header and its alternate, the allocation bitmap,
  the extents and attributes B-trees (empty: each file is one contiguous
  extent) and the catalog B-tree. Names are ASCII; other names are refused
  rather than stored without HFS+'s decomposition. hdiutil makes APFS today;
  HFS+ mounts on every macOS, and Gatekeeper treats the apps on it the same.
- **The image** is UDIF (`udif.rs`), with no partition map, as
  `hdiutil create -layout NONE` writes it. The trailer's and the resource
  fork's fields are those of hdiutil's own images of the same folders: one
  unsegmented image, image variant 2, a `blkx` table and hdiutil's `plst`
  resource. Chunks are 1 MiB, zlib (UDZO), lzfse (ULFO) or uncompressed
  (UDRO), zero runs stored as such.
- **The window** (optional: `--background`, `--window`, `--icon`) is a
  `.DS_Store` (`dsstore.rs`) after the format's public documentation: the
  buddy allocator, one B-tree leaf, the `bwsp` and `icvp` property lists and
  `Iloc` icon positions. The background picture is named by an Alias Manager
  record built from the volume's own catalog node IDs.
- The image is the same for the same inputs: catalog node IDs follow the
  names, every date is `--time` (SOURCE_DATE_EPOCH by default), and files are
  owned by `--uid`/`--gid` (0), as `hdiutil create -anyowners` leaves them.

Checked on a Mac (macOS 27): `hdiutil verify` and `fsck_hfs -fn` pass; the
images mount read-only; the files, their modes and the Applications symlink
are those of hdiutil's images of the same apps (Visor 0.59.1 and Shireworks
Disk Utility 0.1.3); and the signed, stapled apps on our images pass
`codesign --verify --deep --strict`, `stapler validate` and
`spctl -a -t exec` ("accepted, source=Notarized Developer ID"). A laid-out
image opens in the Finder with its background, window size and icon
positions.
