//! A UDIF disk image (what `hdiutil create -format UDZO` writes) around a raw
//! volume: the data fork of compressed chunks, the resource fork's `blkx` table
//! (a `mish` block per partition, mapping sectors to chunks) as an XML plist,
//! and the 512-byte `koly` trailer with the CRC32 checksums.

use std::io::Write;

const SECTOR: usize = 512;
/// Sectors per chunk (1 MiB).
const CHUNK: usize = 2048;

/// How the chunks are stored.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Format {
    /// zlib (`UDZO`).
    Zlib,
    /// lzfse (`ULFO`, macOS 10.11 and later).
    Lzfse,
    /// uncompressed, read-only (`UDRO`).
    Raw,
}

impl Format {
    pub fn parse(s: &str) -> Result<Format, String> {
        match s {
            "UDZO" => Ok(Format::Zlib),
            "ULFO" => Ok(Format::Lzfse),
            "UDRO" => Ok(Format::Raw),
            _ => Err(format!("{s}: the formats are UDZO, ULFO and UDRO")),
        }
    }
}

const ZERO: u32 = 0x0000_0000;
const RAW: u32 = 0x0000_0001;
const ZLIB: u32 = 0x8000_0005;
const LZFSE: u32 = 0x8000_0007;
const END: u32 = 0xffff_ffff;

fn put32(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_be_bytes());
}
fn put64(v: &mut Vec<u8>, x: u64) {
    v.extend_from_slice(&x.to_be_bytes());
}

/// A UDIF checksum field: CRC32 (type 2, 32 bits) and 31 unused words.
fn checksum(crc: u32) -> Vec<u8> {
    let mut v = Vec::new();
    put32(&mut v, 2);
    put32(&mut v, 32);
    put32(&mut v, crc);
    v.resize(136, 0);
    v
}

fn base64(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            out.push(if i <= c.len() { T[(n >> (18 - 6 * i) & 63) as usize] as char } else { '=' });
        }
    }
    out
}

/// The image of `volume` (whole-disk, no partition map), as `hdiutil create -layout NONE` writes one.
pub fn write(volume: &[u8], format: Format) -> Result<Vec<u8>, String> {
    if volume.len() % SECTOR != 0 {
        return Err("a volume is whole sectors".into());
    }
    let sectors = volume.len() / SECTOR;
    let mut fork = Vec::new();
    let mut chunks = Vec::new();
    for (i, c) in volume.chunks(CHUNK * SECTOR).enumerate() {
        let (kind, bytes) = if c.iter().all(|&b| b == 0) {
            (ZERO, Vec::new())
        } else {
            match format {
                Format::Raw => (RAW, c.to_vec()),
                Format::Zlib => {
                    let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
                    z.write_all(c).map_err(|e| e.to_string())?;
                    (ZLIB, z.finish().map_err(|e| e.to_string())?)
                }
                Format::Lzfse => (LZFSE, lzfse::encode(c)),
            }
        };
        let mut e = Vec::new();
        put32(&mut e, kind);
        put32(&mut e, 0);
        put64(&mut e, (i * CHUNK) as u64);
        put64(&mut e, (c.len() / SECTOR) as u64);
        put64(&mut e, fork.len() as u64);
        put64(&mut e, bytes.len() as u64);
        chunks.push(e);
        fork.extend_from_slice(&bytes);
    }
    let mut end = Vec::new();
    put32(&mut end, END);
    put32(&mut end, 0);
    put64(&mut end, sectors as u64);
    put64(&mut end, 0);
    put64(&mut end, fork.len() as u64);
    put64(&mut end, 0);
    chunks.push(end);
    // the partition's mish block: its sectors, a CRC32 of its uncompressed data, its chunks
    let volume_crc = crc32fast::hash(volume);
    let mut mish = Vec::new();
    mish.extend_from_slice(b"mish");
    put32(&mut mish, 1);
    put64(&mut mish, 0);
    put64(&mut mish, sectors as u64);
    put64(&mut mish, 0);
    put32(&mut mish, (CHUNK + 8) as u32);
    put32(&mut mish, 0xffff_fffe);
    mish.extend_from_slice(&[0u8; 24]);
    mish.extend_from_slice(&checksum(volume_crc));
    put32(&mut mish, chunks.len() as u32);
    for c in &chunks {
        mish.extend_from_slice(c);
    }
    let name = "whole disk (Apple_HFS : 0)";
    let lines = |data: &[u8]| -> String { base64(data).as_bytes().chunks(44).map(|l| format!("\t\t\t\t{}\n", std::str::from_utf8(l).unwrap())).collect() };
    // hdiutil's `plst` resource: 1032 bytes, two of them set
    let mut plst = vec![0u8; 1032];
    plst[517] = 1;
    plst[519] = 1;
    let (lines, plst_lines) = (lines(&mish), lines(&plst));
    let plist = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n\t<key>resource-fork</key>\n\t<dict>\n\t\t<key>blkx</key>\n\t\t<array>\n\t\t\t<dict>\n\t\t\t\t<key>Attributes</key>\n\t\t\t\t<string>0x0050</string>\n\t\t\t\t<key>CFName</key>\n\t\t\t\t<string>{name}</string>\n\t\t\t\t<key>Data</key>\n\t\t\t\t<data>\n{lines}\t\t\t\t</data>\n\t\t\t\t<key>ID</key>\n\t\t\t\t<string>0</string>\n\t\t\t\t<key>Name</key>\n\t\t\t\t<string>{name}</string>\n\t\t\t</dict>\n\t\t</array>\n\t\t<key>plst</key>\n\t\t<array>\n\t\t\t<dict>\n\t\t\t\t<key>Attributes</key>\n\t\t\t\t<string>0x0050</string>\n\t\t\t\t<key>Data</key>\n\t\t\t\t<data>\n{plst_lines}\t\t\t\t</data>\n\t\t\t\t<key>ID</key>\n\t\t\t\t<string>0</string>\n\t\t\t\t<key>Name</key>\n\t\t\t\t<string></string>\n\t\t\t</dict>\n\t\t</array>\n\t</dict>\n</dict>\n</plist>\n"
    );
    let mut out = fork.clone();
    let xml_offset = out.len() as u64;
    out.extend_from_slice(plist.as_bytes());
    // the master checksum: a CRC32 over each partition's checksum word
    let master = crc32fast::hash(&volume_crc.to_be_bytes());
    let mut koly = Vec::new();
    koly.extend_from_slice(b"koly");
    put32(&mut koly, 4);
    put32(&mut koly, 512);
    put32(&mut koly, 1); // flattened
    put64(&mut koly, 0);
    put64(&mut koly, 0);
    put64(&mut koly, fork.len() as u64);
    put64(&mut koly, 0);
    put64(&mut koly, 0);
    // one unsegmented image: segment 0 of 0, no segment ID, no checksum of the data fork
    put32(&mut koly, 0);
    put32(&mut koly, 0);
    koly.extend_from_slice(&[0u8; 16]);
    koly.extend_from_slice(&[0u8; 136]);
    put64(&mut koly, xml_offset);
    put64(&mut koly, plist.len() as u64);
    // the code signature's offset and length (none), then reserved
    koly.extend_from_slice(&[0u8; 120]);
    koly.extend_from_slice(&checksum(master));
    put32(&mut koly, 2);
    put64(&mut koly, sectors as u64);
    koly.extend_from_slice(&[0u8; 12]);
    assert_eq!(koly.len(), 512);
    out.extend_from_slice(&koly);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_as_rfc_4648() {
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"f"), "Zg==");
    }

    #[test]
    fn an_image_ends_in_its_trailer() {
        let mut vol = vec![0u8; 3 * 1024 * 1024];
        vol[1024] = b'H';
        vol[2 * 1024 * 1024 + 7] = 1;
        for f in [Format::Zlib, Format::Raw, Format::Lzfse] {
            let img = write(&vol, f).unwrap();
            let k = &img[img.len() - 512..];
            assert_eq!(&k[..4], b"koly");
            assert_eq!(u64::from_be_bytes(k[492..500].try_into().unwrap()), (vol.len() / 512) as u64);
            let xml_at = u64::from_be_bytes(k[216..224].try_into().unwrap()) as usize;
            assert!(img[xml_at..].starts_with(b"<?xml"));
        }
    }
}
