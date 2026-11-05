//! `pngcrush -q -iphone -f 0 -rem text IN OUT`, as rules_apple runs Xcode's
//! for an iOS bundle's PNGs (copypng): an Apple-optimized ("CgBI") PNG, byte
//! for byte Xcode's for 8-bit RGB and RGBA images (docs/clean-room/pngcrush.md).
//! Other PNGs are refused, or copied as they are with
//! `--config=relax_pngcrush`.

use std::process::ExitCode;

const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// The colour chunks libpng writes before a palette, in its order.
const BEFORE_PLTE: [&[u8; 4]; 5] = [b"gAMA", b"sRGB", b"iCCP", b"sBIT", b"cHRM"];
/// Those it writes after.
const AFTER_PLTE: [&[u8; 4]; 3] = [b"bKGD", b"pHYs", b"tIME"];
/// Removed: text (`-rem text`), and chunks Xcode's libpng doesn't know
/// that aren't safe to copy.
const DROPPED: [&[u8; 4]; 5] = [b"tEXt", b"zTXt", b"iTXt", b"iDOT", b"cICP"];
/// Kept where it was, after the colour chunks: unknown to Xcode's libpng but
/// safe to copy.
const COPIED: [&[u8; 4]; 1] = [b"eXIf"];

/// sRGB's gamma and chromaticities, which Xcode writes beside an sRGB chunk
/// in place of any given.
const SRGB_GAMMA: u32 = 45455;
const SRGB_CHROMATICITIES: [u32; 8] = [31270, 32900, 64000, 33000, 30000, 60000, 15000, 6000];

/// Images of this many pixels or more, two rows high or more, are split in
/// two for parallel decoding (an iDOT chunk).
const SPLIT_PIXELS: u64 = 4096;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: pngcrush: {e}");
            ExitCode::FAILURE
        }
    }
}

fn relaxed() -> bool {
    std::env::var("RULES_APPLECROSS_RELAX_PNGCRUSH").is_ok_and(|v| v == "1")
        || std::env::var("RULES_APPLECROSS_RESOURCE_OUTPUT_MODE").is_ok_and(|v| v == "relaxed")
}

fn run(args: &[String]) -> Result<(), String> {
    let (input, output) = match args {
        [q, iphone, f, zero, rem, text, input, output]
            if [q, iphone, f, zero, rem, text].map(String::as_str) == ["-q", "-iphone", "-f", "0", "-rem", "text"] =>
        {
            (input, output)
        }
        _ => return Err(format!("only `pngcrush -q -iphone -f 0 -rem text IN OUT`, rules_apple's copypng, is supported, not {args:?}")),
    };
    let data = std::fs::read(input).map_err(|e| format!("{input}: {e}"))?;
    let out = match iphone(&data) {
        Ok(out) => out,
        Err(reason) if relaxed() => {
            eprintln!("note: {input}: {reason}; copied as it is (--config=relax_pngcrush)");
            data
        }
        Err(reason) => return Err(format!("{input}: {reason}; --config=relax_pngcrush copies such PNGs as they are")),
    };
    std::fs::write(output, out).map_err(|e| format!("{output}: {e}"))
}

struct Chunk<'a> {
    kind: [u8; 4],
    data: &'a [u8],
}

fn chunks(data: &[u8]) -> Result<Vec<Chunk<'_>>, String> {
    if !data.starts_with(SIGNATURE) {
        return Err("not a PNG".into());
    }
    let mut out = Vec::new();
    let mut p = SIGNATURE.len();
    while p < data.len() {
        let header = data.get(p..p + 8).ok_or("a truncated chunk")?;
        let n = u32::from_be_bytes(header[..4].try_into().unwrap()) as usize;
        let kind: [u8; 4] = header[4..8].try_into().unwrap();
        let body = data.get(p + 8..p + 8 + n).ok_or("a truncated chunk")?;
        out.push(Chunk { kind, data: body });
        p += 12 + n;
        if &kind == b"IEND" {
            break;
        }
    }
    Ok(out)
}

/// The Apple-optimized PNG Xcode's pngcrush writes for `data`, or why this
/// one isn't established.
fn iphone(data: &[u8]) -> Result<Vec<u8>, String> {
    let all = chunks(data)?;
    let ihdr = all.first().filter(|c| &c.kind == b"IHDR" && c.data.len() == 13).ok_or("no IHDR chunk first")?;
    let (bit_depth, color_type, interlace) = (ihdr.data[8], ihdr.data[9], ihdr.data[12]);
    if bit_depth != 8 || !matches!(color_type, 2 | 6) || interlace != 0 {
        return Err(format!("only 8-bit RGB and RGBA PNGs, not interlaced, are supported by this pngcrush (bit depth {bit_depth}, colour type {color_type}, interlace {interlace})"));
    }
    // The ancillary chunks: each kind once, before the image data but for
    // text.
    let mut kept: Vec<([u8; 4], Vec<u8>)> = Vec::new();
    let mut copied: Vec<([u8; 4], &[u8])> = Vec::new();
    let mut after_data = false;
    for c in &all[1..] {
        let name = String::from_utf8_lossy(&c.kind).into_owned();
        match &c.kind {
            b"IDAT" => after_data = true,
            b"IEND" => {}
            k if DROPPED.contains(&k) => {}
            _ if after_data => return Err(format!("a {name} chunk after the image data is not supported by this pngcrush")),
            k if BEFORE_PLTE.contains(&k) || AFTER_PLTE.contains(&k) => {
                if kept.iter().any(|(n, _)| n == k) {
                    return Err(format!("two {name} chunks are not supported by this pngcrush"));
                }
                if k == b"sBIT" && color_type == 2 {
                    return Err("an RGB image's sBIT chunk is not supported by this pngcrush".into());
                }
                // Xcode's libpng drops chromaticities out of range: refused.
                if k == b"cHRM" && !plausible_chromaticities(c.data) {
                    return Err("a cHRM chunk Xcode's libpng would ignore is not supported by this pngcrush".into());
                }
                kept.push((*k, c.data.to_vec()));
            }
            k if COPIED.contains(&k) => copied.push((*k, c.data)),
            _ => return Err(format!("the {name} chunk is not supported by this pngcrush")),
        }
    }
    if kept.iter().any(|(k, _)| k == b"sRGB") {
        kept.retain(|(k, _)| k != b"gAMA" && k != b"cHRM");
        kept.push((*b"gAMA", SRGB_GAMMA.to_be_bytes().to_vec()));
        kept.push((*b"cHRM", SRGB_CHROMATICITIES.iter().flat_map(|v| v.to_be_bytes()).collect()));
    }
    if let Some((_, icc)) = kept.iter_mut().find(|(k, _)| k == b"iCCP") {
        *icc = raw_profile(icc)?;
    }

    let decoder = png::Decoder::new(data);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut pixels = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut pixels).map_err(|e| e.to_string())?;
    let (w, h) = (frame.width as usize, frame.height as usize);
    let bpp = if color_type == 6 { 4 } else { 3 };
    let pixels = &pixels[..frame.buffer_size()];
    let opaque = bpp == 3 || pixels.chunks_exact(4).all(|p| p[3] == 255);

    // Unfiltered rows of premultiplied BGRA, rounded to nearest.
    let mut rows = Vec::with_capacity(h * (1 + 4 * w));
    for row in pixels.chunks_exact(w * bpp) {
        rows.push(0);
        for p in row.chunks_exact(bpp) {
            let a = if bpp == 4 { p[3] } else { 255 } as u32;
            let m = |c: u8| ((c as u32 * a + 127) / 255) as u8;
            rows.extend_from_slice(&[m(p[2]), m(p[1]), m(p[0]), a as u8]);
        }
    }

    let mut out = SIGNATURE.to_vec();
    chunk(&mut out, b"CgBI", &(if opaque { 0x5000_2006u32 } else { 0x5000_2002 }).to_be_bytes());
    let mut header = ihdr.data.to_vec();
    header[9] = 6;
    chunk(&mut out, b"IHDR", &header);
    for k in BEFORE_PLTE {
        if let Some((_, d)) = kept.iter().find(|(n, _)| n == k) {
            chunk(&mut out, k, d);
        }
    }
    for (k, d) in &copied {
        chunk(&mut out, k, d);
    }
    for k in AFTER_PLTE {
        if let Some((_, d)) = kept.iter().find(|(n, _)| n == k) {
            chunk(&mut out, k, d);
        }
    }
    if (w * h) as u64 >= SPLIT_PIXELS && h >= 2 {
        // The first part takes the odd row.
        let first = h.div_ceil(2);
        let at = first * (1 + 4 * w);
        let parts = deflate(&[&rows[..at], &rows[at..]])?;
        // Each part's first row, rows and offset from the iDOT chunk's start
        // to its IDAT chunk's.
        let idot_size = 12 + 28;
        let idot = [2, 0, first, idot_size, first, h - first, idot_size + 12 + parts[0].len()];
        chunk(&mut out, b"iDOT", &idot.iter().flat_map(|v| (*v as u32).to_be_bytes()).collect::<Vec<u8>>());
        for p in &parts {
            chunk(&mut out, b"IDAT", p);
        }
    } else {
        chunk(&mut out, b"IDAT", &deflate(&[&rows])?.remove(0));
    }
    chunk(&mut out, b"IEND", &[]);
    Ok(out)
}

/// An iCCP chunk's profile, deflated again raw as the image data is.
fn raw_profile(icc: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let name = icc.iter().position(|&b| b == 0).ok_or("an iCCP chunk without a name")?;
    if icc.get(name + 1) != Some(&0) {
        return Err("an iCCP chunk of another compression method is not supported by this pngcrush".into());
    }
    let mut profile = Vec::new();
    flate2::read::ZlibDecoder::new(&icc[name + 2..]).read_to_end(&mut profile).map_err(|e| format!("an iCCP profile: {e}"))?;
    // A profile whose header gives another length Xcode's libpng drops:
    // refused.
    if profile.len() < 4 || u32::from_be_bytes(profile[..4].try_into().unwrap()) as usize != profile.len() {
        return Err("an iCCP profile Xcode's libpng would ignore (its length isn't its header's) is not supported by this pngcrush".into());
    }
    let mut out = icc[..name + 2].to_vec();
    out.extend(deflate(&[&profile])?.remove(0));
    Ok(out)
}

/// Whether a cHRM chunk's white point and primaries are each an x and y of
/// at most 1 together, with a positive white y.
fn plausible_chromaticities(data: &[u8]) -> bool {
    if data.len() != 32 {
        return false;
    }
    let v: Vec<u32> = data.chunks_exact(4).map(|b| u32::from_be_bytes(b.try_into().unwrap())).collect();
    v.iter().all(|&x| x <= i32::MAX as u32) && v[1] > 0 && v.chunks_exact(2).all(|p| p[0] as u64 + p[1] as u64 <= 100_000)
}

/// One raw deflate stream over `parts` (zlib's level 9, memLevel 9,
/// Z_FILTERED), each but the last ending in a full flush: the bytes of each.
fn deflate(parts: &[&[u8]]) -> Result<Vec<Vec<u8>>, String> {
    use libz_sys as z;
    // Zeroed, so that zlib takes its own allocators, and boxed: zlib's state
    // points back at the stream, which mustn't move.
    let mut boxed = Box::new(std::mem::MaybeUninit::<z::z_stream>::zeroed());
    let stream = boxed.as_mut_ptr();
    let version = unsafe { z::zlibVersion() };
    let status = unsafe { z::deflateInit2_(stream, 9, z::Z_DEFLATED, -15, 9, z::Z_FILTERED, version, std::mem::size_of::<z::z_stream>() as i32) };
    if status != z::Z_OK {
        return Err(format!("zlib: deflateInit2 {status}"));
    }
    let mut out = Vec::new();
    let mut buffer = vec![0u8; 1 << 16];
    for (i, part) in parts.iter().enumerate() {
        let flush = if i + 1 == parts.len() { z::Z_FINISH } else { z::Z_FULL_FLUSH };
        let mut bytes = Vec::new();
        unsafe {
            (*stream).next_in = part.as_ptr() as *mut u8;
            (*stream).avail_in = part.len() as u32;
        }
        loop {
            let status = unsafe {
                (*stream).next_out = buffer.as_mut_ptr();
                (*stream).avail_out = buffer.len() as u32;
                z::deflate(stream, flush)
            };
            if status != z::Z_OK && status != z::Z_STREAM_END && status != z::Z_BUF_ERROR {
                unsafe { z::deflateEnd(stream) };
                return Err(format!("zlib: deflate {status}"));
            }
            let left = unsafe { (*stream).avail_out } as usize;
            bytes.extend_from_slice(&buffer[..buffer.len() - left]);
            // Done once zlib leaves room: all input taken and flushed.
            if left != 0 && unsafe { (*stream).avail_in } == 0 {
                break;
            }
        }
        out.push(bytes);
    }
    unsafe { z::deflateEnd(stream) };
    Ok(out)
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = unsafe { libz_sys::crc32(0, out[start..].as_ptr(), (out.len() - start) as u32) } as u32;
    out.extend_from_slice(&crc.to_be_bytes());
}
