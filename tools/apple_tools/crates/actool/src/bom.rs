//! BOM ("BOMStore") containers, as actool writes them; see docs/clean-room/actool.md.

pub struct Bom {
    /// Blocks by index; index 0 is the null block.
    blocks: Vec<Vec<u8>>,
    vars: Vec<(String, u32)>,
}

fn align16(n: usize) -> usize {
    (n + 15) & !15
}

impl Bom {
    pub fn new() -> Bom {
        Bom { blocks: vec![Vec::new()], vars: Vec::new() }
    }

    pub fn alloc(&mut self, data: Vec<u8>) -> u32 {
        self.blocks.push(data);
        (self.blocks.len() - 1) as u32
    }

    pub fn set(&mut self, index: u32, data: Vec<u8>) {
        self.blocks[index as usize] = data;
    }

    pub fn var(&mut self, name: &str, index: u32) {
        self.vars.push((name.to_string(), index));
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = vec![0u8; 0x200];
        let mut addresses = vec![(0u32, 0u32)];
        for b in &self.blocks[1..] {
            out.resize(align16(out.len()), 0);
            addresses.push((out.len() as u32, b.len() as u32));
            out.extend_from_slice(b);
        }

        out.resize(align16(out.len()), 0);
        let vars_offset = out.len();
        out.extend_from_slice(&(self.vars.len() as u32).to_be_bytes());
        for (name, index) in &self.vars {
            out.extend_from_slice(&index.to_be_bytes());
            out.push(name.len() as u8);
            out.extend_from_slice(name.as_bytes());
        }
        let vars_length = out.len() - vars_offset;

        out.resize(align16(out.len()), 0);
        let index_offset = out.len();
        let used = self.blocks.len() - 1;
        let capacity = std::cmp::max(256, (used + 1 + 7) & !7);
        out.extend_from_slice(&(capacity as u32).to_be_bytes());
        for i in 0..capacity {
            let (a, l) = addresses.get(i).copied().unwrap_or((0, 0));
            out.extend_from_slice(&a.to_be_bytes());
            out.extend_from_slice(&l.to_be_bytes());
        }
        // Free list: no entries, and room for two.
        out.extend_from_slice(&[0u8; 20]);
        let index_length = out.len() - index_offset;

        let mut header = Vec::with_capacity(32);
        header.extend_from_slice(b"BOMStore");
        for v in [1u32, used as u32, index_offset as u32, index_length as u32, vars_offset as u32, vars_length as u32] {
            header.extend_from_slice(&v.to_be_bytes());
        }
        out[..32].copy_from_slice(&header);
        out
    }
}

/// One B+tree as CoreUI stores its tables: a header block and a single leaf
/// page. Entries are (key, value) with the key either a block of its own or,
/// for inline trees, a 32-bit value stored in the page.
pub struct Tree {
    pub header: u32,
    pub page: u32,
    pub block_size: u32,
    pub inline_keys: bool,
}

impl Tree {
    /// Allocates the header and page blocks, to be filled by [`Tree::finish`].
    pub fn reserve(bom: &mut Bom, block_size: u32, inline_keys: bool) -> Tree {
        let header = bom.alloc(Vec::new());
        let page = bom.alloc(Vec::new());
        Tree { header, page, block_size, inline_keys }
    }

    /// Writes the header and the leaf page for `entries` of (key block or
    /// inline key, value block, key bytes), in the order given. The page
    /// holds the entries, four zero bytes, and then, when every key has the
    /// same length (which the header records), a copy of every key; keys
    /// of different lengths are recorded as length 0xffffffff, uncopied.
    pub fn finish(&self, bom: &mut Bom, entries: &[(u32, u32, Vec<u8>)]) {
        let fixed = entries.windows(2).all(|w| w[0].2.len() == w[1].2.len());
        let copied = !self.inline_keys && fixed;
        let key_length = if self.inline_keys {
            0
        } else if fixed {
            entries.first().map_or(0, |e| e.2.len() as u32)
        } else {
            u32::MAX
        };
        let mut h = Vec::with_capacity(29);
        h.extend_from_slice(b"tree");
        for v in [1u32, self.page, self.block_size, entries.len() as u32] {
            h.extend_from_slice(&v.to_be_bytes());
        }
        h.push(self.inline_keys as u8);
        h.extend_from_slice(&key_length.to_be_bytes());
        h.extend_from_slice(&0u32.to_be_bytes());
        bom.set(self.header, h);

        let key_bytes: usize = if copied { entries.iter().map(|e| e.2.len()).sum() } else { 0 };
        let mut p = vec![0u8; self.block_size as usize + key_bytes];
        p[0..2].copy_from_slice(&1u16.to_be_bytes());
        p[2..4].copy_from_slice(&(entries.len() as u16).to_be_bytes());
        for (n, (key, value, _)) in entries.iter().enumerate() {
            let o = 12 + 8 * n;
            p[o..o + 4].copy_from_slice(&value.to_be_bytes());
            p[o + 4..o + 8].copy_from_slice(&key.to_be_bytes());
        }
        if copied {
            let mut o = 12 + 8 * entries.len() + 4;
            for (_, _, k) in entries {
                p[o..o + k.len()].copy_from_slice(k);
                o += k.len();
            }
        }
        bom.set(self.page, p);
    }
}
