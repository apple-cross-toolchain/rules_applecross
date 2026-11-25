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

/// One B+tree as CoreUI stores its tables: a header block, leaf pages and,
/// once a leaf has split, an index page. Entries are (key, value) with the
/// key either a block of its own or, for inline trees, a 32-bit value
/// stored in the page.
///
/// Entries are inserted one at a time, in the order Xcode inserts them,
/// right after their blocks are allocated: a leaf holds (block size − 16) / 8
/// entries; one more splits it, the left leaf keeping the larger half, and
/// the new leaf is the next block allocated, followed, at the first split,
/// by the index page (docs/clean-room/actool.md "Paged trees": every tree of
/// 23 probe catalogs of 127 to 1,000 entries).
pub struct Tree {
    pub header: u32,
    block_size: u32,
    inline_keys: bool,
    /// Leaves in key order: their pages and entries (key block or inline
    /// key, value block, key bytes).
    leaves: Vec<(u32, Vec<(u32, u32, Vec<u8>)>)>,
    index: Option<u32>,
}

impl Tree {
    /// Allocates the header and the first leaf page.
    pub fn reserve(bom: &mut Bom, block_size: u32, inline_keys: bool) -> Tree {
        let header = bom.alloc(Vec::new());
        let page = bom.alloc(Vec::new());
        Tree { header, block_size, inline_keys, leaves: vec![(page, Vec::new())], index: None }
    }

    fn capacity(&self) -> usize {
        (self.block_size as usize - 16) / 8
    }

    fn order(&self, a: &(u32, u32, Vec<u8>), b: &(u32, u32, Vec<u8>)) -> std::cmp::Ordering {
        if self.inline_keys {
            a.0.cmp(&b.0)
        } else {
            a.2.cmp(&b.2)
        }
    }

    /// Inserts an entry whose key (or inline key) and value blocks were just
    /// allocated, splitting a full leaf. More than one index page (a tree of
    /// some 16,000 entries at 1 KB pages) isn't established and is refused.
    pub fn insert(&mut self, bom: &mut Bom, key: u32, value: u32, key_bytes: Vec<u8>) -> Result<(), String> {
        let e = (key, value, key_bytes);
        let last = self.leaves.len() - 1;
        let at = self.leaves.iter().position(|(_, es)| es.last().is_some_and(|l| self.order(l, &e) != std::cmp::Ordering::Less)).unwrap_or(last);
        let pos = self.leaves[at].1.iter().position(|x| self.order(x, &e) == std::cmp::Ordering::Greater).unwrap_or(self.leaves[at].1.len());
        self.leaves[at].1.insert(pos, e);
        if self.leaves[at].1.len() > self.capacity() {
            let keep = self.leaves[at].1.len().div_ceil(2);
            let right = self.leaves[at].1.split_off(keep);
            let page = bom.alloc(Vec::new());
            self.leaves.insert(at + 1, (page, right));
            if self.index.is_none() {
                self.index = Some(bom.alloc(Vec::new()));
            }
            if self.leaves.len() - 1 > self.capacity() {
                return Err(format!("a catalog whose Assets.car needs more than one {}-byte index page for a tree is not supported by this actool", self.block_size));
            }
        }
        Ok(())
    }

    /// Writes the header, the leaves and the index page. A leaf holds its
    /// entries (value, then key), four zero bytes and then, when every key
    /// has the same length (which the header records), a copy of each key;
    /// keys of different lengths are recorded as length 0xffffffff,
    /// uncopied. Leaves link to the next (forward) and previous (backward).
    /// The index page lists each leaf but the last with its last key, then
    /// the last leaf, then the copies of those keys.
    pub fn finish(&self, bom: &mut Bom) -> Result<(), String> {
        let all = || self.leaves.iter().flat_map(|(_, es)| es.iter());
        let count = all().count();
        let first = all().next().map(|e| e.2.len());
        let fixed = all().all(|e| Some(e.2.len()) == first);
        let copied = !self.inline_keys && fixed;
        let key_length = if self.inline_keys {
            0
        } else if fixed {
            first.unwrap_or(0) as u32
        } else {
            u32::MAX
        };
        let root = self.index.unwrap_or(self.leaves[0].0);
        let mut h = Vec::with_capacity(29);
        h.extend_from_slice(b"tree");
        for v in [1u32, root, self.block_size, count as u32] {
            h.extend_from_slice(&v.to_be_bytes());
        }
        h.push(self.inline_keys as u8);
        h.extend_from_slice(&key_length.to_be_bytes());
        h.extend_from_slice(&0u32.to_be_bytes());
        bom.set(self.header, h);

        let page = |leaf: bool, entries: &[(u32, u32, &[u8])], forward: u32, backward: u32, trailing: u32| -> Vec<u8> {
            let key_bytes: usize = if copied { entries.iter().map(|e| e.2.len()).sum() } else { 0 };
            let mut p = vec![0u8; self.block_size as usize + key_bytes];
            p[0..2].copy_from_slice(&(leaf as u16).to_be_bytes());
            p[2..4].copy_from_slice(&(entries.len() as u16).to_be_bytes());
            p[4..8].copy_from_slice(&forward.to_be_bytes());
            p[8..12].copy_from_slice(&backward.to_be_bytes());
            for (n, (key, value, _)) in entries.iter().enumerate() {
                let o = 12 + 8 * n;
                p[o..o + 4].copy_from_slice(&value.to_be_bytes());
                p[o + 4..o + 8].copy_from_slice(&key.to_be_bytes());
            }
            let o = 12 + 8 * entries.len();
            p[o..o + 4].copy_from_slice(&trailing.to_be_bytes());
            if copied {
                let mut o = o + 4;
                for (_, _, k) in entries {
                    p[o..o + k.len()].copy_from_slice(k);
                    o += k.len();
                }
            }
            p
        };
        for (n, (leaf, es)) in self.leaves.iter().enumerate() {
            let forward = self.leaves.get(n + 1).map_or(0, |l| l.0);
            let backward = if n > 0 { self.leaves[n - 1].0 } else { 0 };
            let entries: Vec<(u32, u32, &[u8])> = es.iter().map(|(k, v, b)| (*k, *v, b.as_slice())).collect();
            bom.set(*leaf, page(true, &entries, forward, backward, 0));
        }
        if let Some(index) = self.index {
            // Each leaf but the last, by its last key; the last after them.
            let entries: Vec<(u32, u32, &[u8])> = self.leaves[..self.leaves.len() - 1]
                .iter()
                .map(|(leaf, es)| {
                    let (k, _, b) = es.last().expect("a split leaf has entries");
                    (*k, *leaf, b.as_slice())
                })
                .collect();
            bom.set(index, page(false, &entries, 0, 0, self.leaves[self.leaves.len() - 1].0));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A full 1 KB leaf (126 entries) and one more split into 64 and 63, the
    /// new leaf and the index page the next blocks allocated.
    #[test]
    fn a_full_leaf_splits() {
        let mut bom = Bom::new();
        let mut tree = Tree::reserve(&mut bom, 1024, true);
        for i in 0..127u32 {
            let v = bom.alloc(Vec::new());
            tree.insert(&mut bom, i, v, Vec::new()).unwrap();
        }
        tree.finish(&mut bom).unwrap();
        assert_eq!(tree.leaves.iter().map(|l| l.1.len()).collect::<Vec<_>>(), vec![64, 63]);
        // Header 1, first leaf 2, values 3 to 129: the split at the last.
        assert_eq!(tree.leaves[1].0, 130);
        assert_eq!(tree.index, Some(131));
    }
}
