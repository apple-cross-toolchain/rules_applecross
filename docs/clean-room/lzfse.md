# LZFSE as libcompression encodes it

CoreUI compresses rendition chunks with libcompression's streaming encoder
(`compression_stream`, the whole input in one call); a rendition chunk
decoded and re-encoded that way reproduces actool's bytes. actool's
`dmp2` streams use its buffer encoder (`compression_encode_buffer`).

Both differ from the reference encoder (lzfse
e634ca58b4821d9f3d560cdc6df5dec02ffc93fd) in two tunables, found by
tracing the decoded (literal, match, distance) symbols and block headers of
both encodings and confirmed by randomized comparison:

- With no match at the current position, pending literals are emitted
  (one largest run, 315 bytes) once more than 3 such runs are pending; the
  reference waits for 8.
- LZVN (inputs under 4 KiB) hashes the 24 bits at a position as
  `(x * 2654435761) >> (32 - 14)`; the reference uses
  `(x * (1 + 2^6 + 2^12)) >> 12`.

The streaming encoder also works in windows. It holds at most 3 x 256 KiB.
In a window that does not end the input it starts matches only up to 400
bytes before the data it holds, though matches extend into those bytes. It
then keeps 256 KiB of history behind the position encoding stopped at, and
refills: the next window ends 512 KiB after that position.

`crates/lzfse/vendor` is the reference implementation with those changes,
marked. With them, 120 randomized structured inputs of 1 KB to 2 MB
re-encode byte for byte with both of libcompression's encoders, as do 500
of 10 bytes to 1.2 MB with its buffer encoder and 600 under 4 KiB through
LZVN; without them, about one in seven differed.

Where the buffer encoder's output would not fit the destination it is
given, it stores the input raw; the destination size actool gives it is not
established.
