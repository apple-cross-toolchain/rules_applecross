//! The metallib container (FORMAT.md, "The metallib container").

use sha2::{Digest, Sha256};

pub struct Function {
    pub name: String,
    /// 0 vertex, 1 fragment, 2 kernel
    pub kind: u8,
    /// Index into `Library::modules`.
    pub module: usize,
    pub air: (u32, u32),
    pub lang: (u32, u32),
    /// The public metadata block: u32 size, tags, ENDT.
    pub public: Vec<u8>,
}

pub struct Library {
    pub header: [u16; 6],
    /// Wrapped bitcode, each padded to 16 bytes.
    pub modules: Vec<Vec<u8>>,
    pub functions: Vec<Function>,
}

const EMPTY: &[u8] = &[4, 0, 0, 0, b'E', b'N', b'D', b'T'];

fn tag(name: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut v = name.to_vec();
    v.extend_from_slice(&(data.len() as u16).to_le_bytes());
    v.extend_from_slice(data);
    v
}

/// The `0B17C0DE` wrapper (llvm-as writes one itself for apple triples), padded to 16 bytes.
pub fn wrap(bitcode: &[u8]) -> Vec<u8> {
    let mut v = if bitcode.starts_with(&[0xde, 0xc0, 0x17, 0x0b]) {
        bitcode.to_vec()
    } else {
        let mut w = Vec::new();
        for x in [0x0B17C0DEu32, 0, 20, bitcode.len() as u32, 0xFFFFFFFF] {
            w.extend_from_slice(&x.to_le_bytes());
        }
        w.extend_from_slice(bitcode);
        w
    };
    while v.len() % 16 != 0 {
        v.push(0);
    }
    v
}

/// A public metadata block from tags (each `tag(..)`-encoded).
pub fn block(tags: &[Vec<u8>]) -> Vec<u8> {
    let mut body: Vec<u8> = tags.concat();
    body.extend_from_slice(b"ENDT");
    let mut v = (body.len() as u32).to_le_bytes().to_vec();
    v.extend(body);
    v
}

pub fn tagged(name: &[u8; 4], data: &[u8]) -> Vec<u8> {
    tag(name, data)
}

impl Library {
    pub fn write(&self) -> Vec<u8> {
        let mut module_offsets = Vec::new();
        let mut mods = Vec::new();
        for m in &self.modules {
            module_offsets.push(mods.len() as u64);
            mods.extend_from_slice(m);
        }
        let (mut entries, mut public, mut private) = (Vec::new(), Vec::new(), Vec::new());
        for f in &self.functions {
            let module = &self.modules[f.module];
            let mut name = f.name.as_bytes().to_vec();
            name.push(0);
            let mut offt = Vec::new();
            for x in [public.len() as u64, private.len() as u64, module_offsets[f.module]] {
                offt.extend_from_slice(&x.to_le_bytes());
            }
            let mut vers = Vec::new();
            for x in [f.air.0, f.air.1, f.lang.0, f.lang.1] {
                vers.extend_from_slice(&(x as u16).to_le_bytes());
            }
            let mut body = Vec::new();
            body.extend(tag(b"NAME", &name));
            body.extend(tag(b"TYPE", &[f.kind]));
            body.extend(tag(b"HASH", &Sha256::digest(module)));
            body.extend(tag(b"MDSZ", &(module.len() as u64).to_le_bytes()));
            body.extend(tag(b"OFFT", &offt));
            body.extend(tag(b"VERS", &vers));
            body.extend_from_slice(b"ENDT");
            entries.extend_from_slice(&((body.len() + 4) as u32).to_le_bytes());
            entries.extend(body);
            public.extend_from_slice(if f.public.is_empty() { EMPTY } else { &f.public });
            private.extend_from_slice(EMPTY);
        }
        let mut flist = (self.functions.len() as u32).to_le_bytes().to_vec();
        flist.extend(entries);
        let o_f = 0x58u64;
        let o_pub = o_f + flist.len() as u64 + 4;
        let o_priv = o_pub + public.len() as u64;
        let o_mod = o_priv + private.len() as u64;
        let total = o_mod + mods.len() as u64;
        let mut out = b"MTLB".to_vec();
        for x in self.header {
            out.extend_from_slice(&x.to_le_bytes());
        }
        for x in [total, o_f, flist.len() as u64 - 4, o_pub, public.len() as u64, o_priv, private.len() as u64, o_mod, mods.len() as u64] {
            out.extend_from_slice(&x.to_le_bytes());
        }
        out.extend(flist);
        out.extend_from_slice(b"ENDT");
        out.extend(public);
        out.extend(private);
        out.extend(mods);
        out
    }
}
