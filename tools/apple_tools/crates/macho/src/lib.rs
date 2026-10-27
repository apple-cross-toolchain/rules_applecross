//! The parts of Mach-O that signing reads and writes.

use std::fmt;

pub const MH_MAGIC_64: u32 = 0xfeed_facf;
pub const FAT_MAGIC: u32 = 0xcafe_babe;
pub const MH_EXECUTE: u32 = 0x2;
pub const LC_SEGMENT_64: u32 = 0x19;
pub const LC_CODE_SIGNATURE: u32 = 0x1d;
pub const LC_BUILD_VERSION: u32 = 0x32;
pub const LC_VERSION_MIN_IPHONEOS: u32 = 0x25;
pub const LC_UUID: u32 = 0x1b;
pub const PLATFORM_IOS: u32 = 2;
pub const PLATFORM_TVOS: u32 = 3;
pub const CPU_TYPE_ARM64: u32 = 0x0100_000c;
/// The load commands naming a dylib the image links.
const LC_DYLIB_LINKS: [u32; 5] = [0xc, 0x8000_0018, 0x8000_001f, 0x20, 0x8000_0023];

#[derive(Debug)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

fn err<T>(msg: impl Into<String>) -> Result<T, Error> {
    Err(Error(msg.into()))
}

fn u32le(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}

fn u64le(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}

fn put32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

fn put64(d: &mut [u8], o: usize, v: u64) {
    d[o..o + 8].copy_from_slice(&v.to_le_bytes());
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub name: String,
    /// Offset of the load command in the image.
    pub command_offset: usize,
    pub vmaddr: u64,
    pub vmsize: u64,
    pub fileoff: u64,
    pub filesize: u64,
}

#[derive(Clone, Debug)]
pub struct Image {
    pub data: Vec<u8>,
    pub cputype: u32,
    pub cpusubtype: u32,
    pub filetype: u32,
    pub segments: Vec<Segment>,
    /// (command offset, dataoff, datasize)
    pub code_signature: Option<(usize, u32, u32)>,
    /// (platform, minos) from LC_BUILD_VERSION.
    pub build_version: Option<(u32, u32)>,
    /// The SDK version from LC_BUILD_VERSION, as X.Y.Z nibbles (xxxx.yy.zz).
    pub sdk_version: Option<u32>,
    /// The install names of the dylibs the image links, in load command
    /// order.
    pub dylibs: Vec<String>,
    pub uuid: Option<[u8; 16]>,
    ncmds: u32,
    sizeofcmds: u32,
}

impl Image {
    pub fn parse(data: Vec<u8>) -> Result<Image, Error> {
        if data.len() < 32 || u32le(&data, 0) != MH_MAGIC_64 {
            return err("not a 64-bit Mach-O image");
        }
        let cputype = u32le(&data, 4);
        let cpusubtype = u32le(&data, 8);
        let filetype = u32le(&data, 12);
        let ncmds = u32le(&data, 16);
        let sizeofcmds = u32le(&data, 20);
        let mut segments = Vec::new();
        let mut code_signature = None;
        let mut build_version = None;
        let mut sdk_version = None;
        let mut dylibs = Vec::new();
        let mut uuid = None;
        let mut off = 32usize;
        for _ in 0..ncmds {
            if off + 8 > data.len() {
                return err("load commands run past the end of the file");
            }
            let cmd = u32le(&data, off);
            let size = u32le(&data, off + 4) as usize;
            if size < 8 || off + size > data.len() {
                return err("malformed load command");
            }
            match cmd {
                LC_SEGMENT_64 => {
                    let name = String::from_utf8_lossy(&data[off + 8..off + 24]).trim_end_matches('\0').to_string();
                    segments.push(Segment {
                        name,
                        command_offset: off,
                        vmaddr: u64le(&data, off + 24),
                        vmsize: u64le(&data, off + 32),
                        fileoff: u64le(&data, off + 40),
                        filesize: u64le(&data, off + 48),
                    });
                }
                LC_CODE_SIGNATURE => code_signature = Some((off, u32le(&data, off + 8), u32le(&data, off + 12))),
                LC_BUILD_VERSION => {
                    build_version = Some((u32le(&data, off + 8), u32le(&data, off + 12)));
                    sdk_version = Some(u32le(&data, off + 16));
                }
                LC_UUID if size >= 24 => uuid = Some(data[off + 8..off + 24].try_into().unwrap()),
                c if LC_DYLIB_LINKS.contains(&c) => {
                    let name_off = u32le(&data, off + 8) as usize;
                    if name_off >= size {
                        return err("malformed dylib load command");
                    }
                    let name = &data[off + name_off..off + size];
                    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
                    dylibs.push(String::from_utf8_lossy(&name[..end]).into_owned());
                }
                _ => {}
            }
            off += size;
        }
        Ok(Image { data, cputype, cpusubtype, filetype, segments, code_signature, build_version, sdk_version, dylibs, uuid, ncmds, sizeofcmds })
    }

    pub fn segment(&self, name: &str) -> Option<&Segment> {
        self.segments.iter().find(|s| s.name == name)
    }

    /// The file contents of a section, if the image has it in the file.
    pub fn section(&self, segment: &str, section: &str) -> Option<&[u8]> {
        let seg = self.segment(segment)?;
        let nsects = u32le(&self.data, seg.command_offset + 64) as usize;
        let fixed = |b: &[u8]| String::from_utf8_lossy(b).trim_end_matches('\0').to_string();
        for i in 0..nsects {
            let s = seg.command_offset + 72 + i * 80;
            if fixed(&self.data[s..s + 16]) == section {
                let size = u64le(&self.data, s + 40) as usize;
                let offset = u32le(&self.data, s + 48) as usize;
                return self.data.get(offset..offset.checked_add(size)?);
            }
        }
        None
    }

    /// The end of the bytes a signature covers: the image without any
    /// signature already present.
    pub fn unsigned_end(&self) -> usize {
        match self.code_signature {
            Some((_, dataoff, _)) => dataoff as usize,
            None => self.data.len(),
        }
    }

    /// The first file offset of section data, which load commands may not reach.
    fn first_section_offset(&self) -> usize {
        let mut first = self.data.len();
        let mut off = 32usize;
        for _ in 0..self.ncmds {
            let cmd = u32le(&self.data, off);
            let size = u32le(&self.data, off + 4) as usize;
            if cmd == LC_SEGMENT_64 {
                let nsects = u32le(&self.data, off + 64) as usize;
                for i in 0..nsects {
                    let s = off + 72 + i * 80;
                    let flags = u32le(&self.data, s + 64);
                    let section_type = flags & 0xff;
                    let offset = u32le(&self.data, s + 48) as usize;
                    // Zero-fill sections occupy no file space.
                    let zerofill = matches!(section_type, 0x1 | 0xc | 0x12);
                    if offset != 0 && !zerofill {
                        first = first.min(offset);
                    }
                }
            }
            off += size;
        }
        first
    }

    /// Reserves `size` bytes for a signature, as `codesign_allocate -a` does.
    /// Returns the signature's file offset.
    pub fn allocate_signature(&mut self, size: u32, page_size: u64) -> Result<u32, Error> {
        let linkedit = self.segment("__LINKEDIT").cloned().ok_or_else(|| Error("no __LINKEDIT segment".into()))?;
        let (dataoff, command) = match self.code_signature {
            Some((command, dataoff, _)) => (dataoff, command),
            None => {
                let end = (linkedit.fileoff + linkedit.filesize) as usize;
                if end != self.data.len() {
                    return err("__LINKEDIT does not end the file");
                }
                let dataoff = (end + 15) & !15;
                let command = 32 + self.sizeofcmds as usize;
                if command + 16 > self.first_section_offset() {
                    return err("no room in the header for LC_CODE_SIGNATURE");
                }
                self.data[command..command + 16].fill(0);
                put32(&mut self.data, command, LC_CODE_SIGNATURE);
                put32(&mut self.data, command + 4, 16);
                self.ncmds += 1;
                self.sizeofcmds += 16;
                put32(&mut self.data, 16, self.ncmds);
                put32(&mut self.data, 20, self.sizeofcmds);
                (dataoff as u32, command)
            }
        };
        put32(&mut self.data, command + 8, dataoff);
        put32(&mut self.data, command + 12, size);
        self.code_signature = Some((command, dataoff, size));
        // Like codesign, write over a signature already there: bytes of it the
        // new one does not cover stay; only space added is zeroed.
        let new_len = dataoff as usize + size as usize;
        self.data.resize(new_len, 0);
        let filesize = new_len as u64 - linkedit.fileoff;
        let vmsize = filesize.div_ceil(page_size) * page_size;
        put64(&mut self.data, linkedit.command_offset + 48, filesize);
        put64(&mut self.data, linkedit.command_offset + 32, vmsize);
        for s in &mut self.segments {
            if s.name == "__LINKEDIT" {
                s.filesize = filesize;
                s.vmsize = vmsize;
            }
        }
        Ok(dataoff)
    }
}

/// A file that is either one image or a fat archive of several.
pub enum Universal {
    Thin(Image),
    Fat(Vec<(u32, Image)>),
}

impl Universal {
    pub fn parse(data: Vec<u8>) -> Result<Universal, Error> {
        if data.len() >= 8 && u32::from_be_bytes(data[0..4].try_into().unwrap()) == FAT_MAGIC {
            let n = u32::from_be_bytes(data[4..8].try_into().unwrap()) as usize;
            let mut slices = Vec::new();
            for i in 0..n {
                let o = 8 + i * 20;
                let be = |k: usize| u32::from_be_bytes(data[o + k..o + k + 4].try_into().unwrap());
                let (offset, size, align) = (be(8) as usize, be(12) as usize, be(16));
                slices.push((align, Image::parse(data[offset..offset + size].to_vec())?));
            }
            Ok(Universal::Fat(slices))
        } else {
            Ok(Universal::Thin(Image::parse(data)?))
        }
    }

    pub fn images(&self) -> Vec<&Image> {
        match self {
            Universal::Thin(i) => vec![i],
            Universal::Fat(s) => s.iter().map(|(_, i)| i).collect(),
        }
    }

    pub fn images_mut(&mut self) -> Vec<&mut Image> {
        match self {
            Universal::Thin(i) => vec![i],
            Universal::Fat(s) => s.iter_mut().map(|(_, i)| i).collect(),
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Universal::Thin(i) => i.data,
            Universal::Fat(slices) => {
                let mut header = Vec::new();
                header.extend_from_slice(&FAT_MAGIC.to_be_bytes());
                header.extend_from_slice(&(slices.len() as u32).to_be_bytes());
                let mut offset = 8 + 20 * slices.len();
                let mut body: Vec<(usize, Vec<u8>)> = Vec::new();
                for (align, image) in &slices {
                    let a = 1usize << align;
                    offset = offset.div_ceil(a) * a;
                    header.extend_from_slice(&image.cputype.to_be_bytes());
                    header.extend_from_slice(&image.cpusubtype.to_be_bytes());
                    header.extend_from_slice(&(offset as u32).to_be_bytes());
                    header.extend_from_slice(&(image.data.len() as u32).to_be_bytes());
                    header.extend_from_slice(&align.to_be_bytes());
                    body.push((offset, image.data.clone()));
                    offset += image.data.len();
                }
                let mut out = header;
                for (o, d) in body {
                    out.resize(o, 0);
                    out.extend_from_slice(&d);
                }
                out
            }
        }
    }
}
