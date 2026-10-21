/*
LZFSE encoding that matches the output of libcompression's streaming
encoder fed the whole input at once, which is how CoreUI's asset compiler
compresses renditions.

Observed against it (docs/clean-room/lzfse.md): lzfse_encode_buffer() hands
lzfse_encode_base() the whole input (below 4 GiB). The streaming encoder
holds at most 3 * 256 KiB. Within a window that does not end the input it
starts matches only up to 400 bytes before the data it holds, though
matches may extend into those bytes. It then keeps 256 KiB of history
behind the position it stopped at and refills, so the next window ends 512
KiB after that position.

That lives here and in the src_encode_limit field of lzfse_encoder_state,
marked. libcompression's buffer and streaming encoders also differ from the
reference in two tunables, marked in lzfse_encode_base.c and
lzvn_encode_base.c; the rest of the reference sources are unmodified (lzfse
e634ca58b4821d9f3d560cdc6df5dec02ffc93fd).
*/

#include "lzfse.h"
#include "lzfse_internal.h"

#include <string.h>

// Defined in lzfse_encode.c, not declared by its headers.
size_t lzfse_encode_buffer_with_scratch(uint8_t *__restrict dst_buffer,
                                        size_t dst_size,
                                        const uint8_t *__restrict src_buffer,
                                        size_t src_size,
                                        void *__restrict scratch_buffer);

#define STREAM_BLOCK ((lzfse_offset)262144)
#define STREAM_CAPACITY (3 * STREAM_BLOCK)
#define STREAM_LOOKAHEAD ((lzfse_offset)400)

size_t lzfse_encode_stream_buffer(uint8_t *__restrict dst_buffer,
                                  size_t dst_size,
                                  const uint8_t *__restrict src_buffer,
                                  size_t src_size,
                                  void *__restrict scratch_buffer) {
  if (src_size < LZFSE_ENCODE_LZVN_THRESHOLD || src_size >= 0xffffffffU)
    return lzfse_encode_buffer_with_scratch(dst_buffer, dst_size, src_buffer,
                                            src_size, scratch_buffer);

  lzfse_encoder_state *state = scratch_buffer;
  memset(state, 0x00, sizeof *state);
  if (lzfse_encode_init(state) != LZFSE_STATUS_OK)
    goto try_uncompressed;
  state->dst = dst_buffer;
  state->dst_begin = dst_buffer;
  state->dst_end = &dst_buffer[dst_size];
  state->src = src_buffer;
  state->src_encode_i = 0;

  {
    const lzfse_offset total = (lzfse_offset)src_size;
    lzfse_offset window_start = 0; // absolute offset of state->src[0]
    lzfse_offset data_end = total < STREAM_CAPACITY ? total : STREAM_CAPACITY;
    state->src_end = data_end;
    state->src_encode_limit = data_end < total ? data_end - STREAM_LOOKAHEAD : 0;
    while (data_end < total) {
      if (lzfse_encode_base(state) != LZFSE_STATUS_OK)
        goto try_uncompressed;
      lzfse_offset stop = window_start + state->src_encode_i;
      lzfse_offset new_start = stop - STREAM_BLOCK;
      lzfse_encode_translate(state, new_start - window_start);
      window_start = new_start;
      data_end = stop + 2 * STREAM_BLOCK < total ? stop + 2 * STREAM_BLOCK : total;
      state->src_end = data_end - window_start;
      state->src_encode_limit =
          data_end < total ? data_end - STREAM_LOOKAHEAD - window_start : 0;
    }
  }
  if (lzfse_encode_base(state) != LZFSE_STATUS_OK)
    goto try_uncompressed;
  if (lzfse_encode_finish(state) != LZFSE_STATUS_OK)
    goto try_uncompressed;
  return state->dst - dst_buffer;

try_uncompressed:
  if (src_size + 12 <= dst_size && src_size < INT32_MAX) {
    // Same raw block as the reference encoder writes when compression fails.
    uncompressed_block_header header = {.magic = LZFSE_UNCOMPRESSED_BLOCK_MAGIC,
                                        .n_raw_bytes = (uint32_t)src_size};
    uint8_t *dst_end = dst_buffer;
    memcpy(dst_end, &header, sizeof header);
    dst_end += sizeof header;
    memcpy(dst_end, src_buffer, src_size);
    dst_end += src_size;
    store4(dst_end, LZFSE_ENDOFSTREAM_BLOCK_MAGIC);
    dst_end += 4;
    return dst_end - dst_buffer;
  }
  return 0;
}
