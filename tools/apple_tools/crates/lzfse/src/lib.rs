//! LZFSE as libcompression encodes it: the reference implementation with
//! its tunables, buffer and streaming encoders (docs/clean-room/lzfse.md).

use std::ffi::c_void;

extern "C" {
    fn lzfse_encode_scratch_size() -> usize;
    fn lzfse_decode_scratch_size() -> usize;
    fn lzfse_encode_buffer(dst: *mut u8, dst_size: usize, src: *const u8, src_size: usize, scratch: *mut c_void) -> usize;
    fn lzfse_encode_stream_buffer(dst: *mut u8, dst_size: usize, src: *const u8, src_size: usize, scratch: *mut c_void) -> usize;
    fn lzfse_decode_buffer(dst: *mut u8, dst_size: usize, src: *const u8, src_size: usize, scratch: *mut c_void) -> usize;
    fn lzvn_encode_scratch_size() -> usize;
    fn lzvn_encode_buffer(dst: *mut u8, dst_size: usize, src: *const u8, src_size: usize, scratch: *mut c_void) -> usize;
}

type Encoder = unsafe extern "C" fn(*mut u8, usize, *const u8, usize, *mut c_void) -> usize;

fn run(encoder: Encoder, src: &[u8]) -> Vec<u8> {
    // A raw block (12-byte header, the input, a 4-byte end marker) is the
    // largest output either encoder produces.
    let mut dst = vec![0u8; src.len() + 16];
    let mut scratch = vec![0u8; unsafe { lzfse_encode_scratch_size() } + 1];
    let n = unsafe { encoder(dst.as_mut_ptr(), dst.len(), src.as_ptr(), src.len(), scratch.as_mut_ptr().cast()) };
    assert!(n > 0, "LZFSE refused an input of {} bytes", src.len());
    dst.truncate(n);
    dst
}

/// libcompression's buffer encoder's output for `src`.
pub fn encode(src: &[u8]) -> Vec<u8> {
    run(lzfse_encode_buffer, src)
}

/// LZVN's encoding of `src`, if its encoder fits it into `capacity` bytes.
/// It gives up short of the end of its buffer, so this can be `None` for an
/// output of at most `capacity` bytes.
pub fn encode_lzvn(src: &[u8], capacity: usize) -> Option<Vec<u8>> {
    let mut dst = vec![0u8; capacity];
    let mut scratch = vec![0u8; unsafe { lzvn_encode_scratch_size() } + 1];
    let n = unsafe { lzvn_encode_buffer(dst.as_mut_ptr(), dst.len(), src.as_ptr(), src.len(), scratch.as_mut_ptr().cast()) };
    (n > 0).then(|| {
        dst.truncate(n);
        dst
    })
}

/// libcompression's streaming encoder's output for `src` given in one piece.
pub fn encode_stream(src: &[u8]) -> Vec<u8> {
    run(lzfse_encode_stream_buffer, src)
}

/// Decodes `src`, whose decoded size is known to be `size`.
pub fn decode(src: &[u8], size: usize) -> Option<Vec<u8>> {
    let mut dst = vec![0u8; size + 1];
    let mut scratch = vec![0u8; unsafe { lzfse_decode_scratch_size() } + 1];
    let n = unsafe { lzfse_decode_buffer(dst.as_mut_ptr(), dst.len(), src.as_ptr(), src.len(), scratch.as_mut_ptr().cast()) };
    (n == size).then(|| {
        dst.truncate(n);
        dst
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let mut x: u32 = 1;
        let data: Vec<u8> = (0..1 << 20)
            .map(|i| {
                x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                if i % 7 < 4 { (i / 97) as u8 } else { (x >> 24) as u8 }
            })
            .collect();
        for encoded in [encode(&data), encode_stream(&data)] {
            assert_eq!(decode(&encoded, data.len()).as_deref(), Some(&data[..]));
        }
    }
}
