From https://github.com/lzfse/lzfse at e634ca58b4821d9f3d560cdc6df5dec02ffc93fd,
unchanged except, each marked:

- `lzfse_encode_base.c`: pending literals are emitted past 3 (not 8) times
  the largest literal run, as libcompression's encoders do;
- `lzvn_encode_base.c`: LZVN hashes multiplicatively, as libcompression's
  encoders do;
- `lzfse_encode_base.c`, `lzfse_internal.h`: the encoder can stop starting
  matches at a limit, as the streaming encoder does at the end of a window;
- `lzfse_encode_stream.c` (new): encodes a buffer the way libcompression's
  streaming encoder given it in one call does.

See docs/clean-room/lzfse.md.
