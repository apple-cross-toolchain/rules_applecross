# vImage's power function, probed

`crates/ibtool/src/colorsync.rs` reproduces the colour conversions Xcode's
ibtool performs, bit for bit (docs/clean-room/ibtool.md, Colour
conversion). Its exp2 and log2 polynomials match vImage's but for listed
exceptions, `crates/ibtool/src/vimage_pow.bin`. These programs regenerate
and verify that list on a Mac. They call public ColorSync and vImage
functions in their own process and compare outputs; nothing is
disassembled.

| Program | What it does |
| --- | --- |
| `conversion_steps.m` | ColorSync's parametric description of a conversion (`GenericRGB DisplayP3`, `DisplayP3 sRGB`, `sRGB GenericRGB`, ...) |
| `exp2_exceptions.c` | vImage's 2^f for every float f in a range against the exp2 polynomial; writes the differences |
| `log2_recover.c` | vImage's log2 of every float in a range, recovered from x^(2^k) with the exact exp2 |
| `log2_exceptions.c` | the recovered log2 against the two log2 polynomials; writes the differences |
| `validate_pow.c` | the whole power function against vImage on random bases and exponents |
| `validate_curves.c` | the three conversion curves against vImage for every float in a range |

To regenerate: `regenerate.sh` (about ten minutes on 16 cores). It writes
`vimage_pow.bin` here, to compare with the crate's.
