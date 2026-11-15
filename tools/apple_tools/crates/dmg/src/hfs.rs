//! An HFS+ volume written from a directory tree, after Apple's Technical Note
//! TN1150 ("HFS Plus Volume Format"): the volume header and its alternate, the
//! allocation bitmap, the extents and attributes B-trees (empty: every fork is
//! one contiguous extent) and the catalog B-tree (folders, files, symlinks and
//! their threads). Read-only and deterministic: catalog node IDs in key order,
//! one timestamp for everything.

use std::collections::BTreeMap;

pub const BLOCK: u32 = 4096;
const NODE: usize = 8192;
const SMALL_NODE: usize = 4096;
const ROOT_PARENT: u32 = 1;
const ROOT: u32 = 2;
const FIRST_USER_ID: u32 = 16;

/// Seconds between the HFS+ epoch (1904-01-01) and the Unix one.
const EPOCH_OFFSET: u64 = 2_082_844_800;

/// A Finder flag on a folder or file (`kIsInvisible`).
pub const INVISIBLE: u16 = 0x4000;

/// One item in the tree to write.
#[derive(Clone, Debug)]
pub enum Node {
    Folder { children: BTreeMap<String, Node>, mode: u16, finder_flags: u16 },
    File { data: Vec<u8>, mode: u16, finder_flags: u16, type_creator: Option<([u8; 4], [u8; 4])> },
    Symlink { target: String },
}

impl Node {
    #[cfg(test)]
    pub fn folder() -> Node {
        Node::Folder { children: BTreeMap::new(), mode: 0o755, finder_flags: 0 }
    }
}

/// What the volume is made of and how it is stamped.
pub struct Volume {
    pub name: String,
    pub root: Node,
    /// Unix seconds for every date on the volume.
    pub time: u64,
    pub uid: u32,
    pub gid: u32,
    /// The folder (a path from the root, "" for the root) the Finder opens on mount.
    pub open_folder: Option<String>,
    /// Free space to leave, in bytes.
    pub free: u64,
}

/// A name as HFS+ stores it: UTF-16 code units, `:` written as `/` (the
/// catalog's separator is `:`).
fn hfs_name(name: &str) -> Result<Vec<u16>, String> {
    if !name.is_ascii() {
        return Err(format!("{name:?}: only ASCII names are supported (HFS+ needs its decomposed form otherwise)"));
    }
    if name.is_empty() || name.len() > 255 {
        return Err(format!("{name:?}: a name is 1 to 255 characters"));
    }
    Ok(name.chars().map(|c| if c == ':' { '/' as u16 } else { c as u16 }).collect())
}

/// HFS+'s case-insensitive order (TN1150's FastUnicodeCompare, for ASCII: A-Z fold to a-z).
fn compare_names(a: &[u16], b: &[u16]) -> std::cmp::Ordering {
    let fold = |c: u16| if (b'A' as u16..=b'Z' as u16).contains(&c) { c + 32 } else { c };
    a.iter().map(|&c| fold(c)).cmp(b.iter().map(|&c| fold(c)))
}

#[derive(Clone)]
struct Key {
    parent: u32,
    name: Vec<u16>,
}

impl Key {
    fn bytes(&self) -> Vec<u8> {
        let mut k = Vec::new();
        put16(&mut k, (6 + 2 * self.name.len()) as u16);
        put32(&mut k, self.parent);
        put16(&mut k, self.name.len() as u16);
        for c in &self.name {
            put16(&mut k, *c);
        }
        k
    }
    fn order(&self, other: &Key) -> std::cmp::Ordering {
        self.parent.cmp(&other.parent).then_with(|| compare_names(&self.name, &other.name))
    }
}

fn put16(v: &mut Vec<u8>, x: u16) {
    v.extend_from_slice(&x.to_be_bytes());
}
fn put32(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_be_bytes());
}
fn put64(v: &mut Vec<u8>, x: u64) {
    v.extend_from_slice(&x.to_be_bytes());
}

/// One fork: (logical size, first block, block count).
#[derive(Clone, Copy, Default)]
struct Fork {
    size: u64,
    start: u32,
    blocks: u32,
}

impl Fork {
    fn bytes(&self, clump: u32) -> Vec<u8> {
        let mut v = Vec::new();
        put64(&mut v, self.size);
        put32(&mut v, clump);
        put32(&mut v, self.blocks);
        put32(&mut v, if self.blocks > 0 { self.start } else { 0 });
        put32(&mut v, self.blocks);
        v.resize(80, 0);
        v
    }
}

/// A catalog entry before layout.
enum Entry {
    Folder { id: u32, parent: u32, name: Vec<u16>, valence: u32, mode: u16, finder_flags: u16 },
    File { id: u32, parent: u32, name: Vec<u16>, mode: u16, finder_flags: u16, type_creator: [u8; 8], data: usize },
}

/// The B-tree nodes for `records` (sorted by key): leaves, then the index levels
/// above them, behind a header node. Returns the file's bytes.
fn btree(records: &[(Key, Vec<u8>)], node_size: usize, max_key: u16, attributes: u32, compare_type: u8, extra_nodes: u32) -> Vec<u8> {
    // nodes: (kind, height, records); node 0 is the header
    let mut levels: Vec<Vec<Vec<(Vec<u8>, Vec<u8>)>>> = Vec::new();
    let fits = |recs: &[(Vec<u8>, Vec<u8>)], add: usize| -> bool {
        let used: usize = 14 + recs.iter().map(|(k, d)| k.len() + d.len()).sum::<usize>() + add + 2 * (recs.len() + 2);
        used <= node_size
    };
    let mut leaves: Vec<Vec<(Vec<u8>, Vec<u8>)>> = Vec::new();
    for (k, d) in records {
        let kb = k.bytes();
        let mut db = d.clone();
        if (kb.len() + db.len()) % 2 == 1 {
            db.push(0);
        }
        match leaves.last_mut() {
            Some(n) if fits(n, kb.len() + db.len()) => n.push((kb, db)),
            _ => leaves.push(vec![(kb, db)]),
        }
    }
    let leaf_count = leaves.len();
    levels.push(leaves);
    // index levels: each child's first key (variable-length index keys), then its node number
    let mut next_number = 1u32;
    let mut numbers: Vec<Vec<u32>> = Vec::new();
    loop {
        let level = levels.last().unwrap();
        let ids: Vec<u32> = (0..level.len() as u32).map(|i| next_number + i).collect();
        next_number += level.len() as u32;
        numbers.push(ids.clone());
        if level.len() <= 1 {
            break;
        }
        let mut parents: Vec<Vec<(Vec<u8>, Vec<u8>)>> = Vec::new();
        for (node, id) in level.iter().zip(&ids) {
            let key = node[0].0.clone();
            let ptr = id.to_be_bytes().to_vec();
            match parents.last_mut() {
                Some(n) if fits(n, key.len() + 4) => n.push((key, ptr)),
                _ => parents.push(vec![(key, ptr)]),
            }
        }
        levels.push(parents);
    }
    let depth = if records.is_empty() { 0 } else { levels.len() as u16 };
    let total_nodes = if records.is_empty() { 1 + extra_nodes } else { next_number + extra_nodes };
    let mut out = vec![0u8; total_nodes as usize * node_size];
    // the record nodes
    for (h, (level, ids)) in levels.iter().zip(&numbers).enumerate() {
        if records.is_empty() {
            break;
        }
        for (i, (node, id)) in level.iter().zip(ids).enumerate() {
            let mut n = Vec::with_capacity(node_size);
            put32(&mut n, if i + 1 < ids.len() { ids[i + 1] } else { 0 });
            put32(&mut n, if i > 0 { ids[i - 1] } else { 0 });
            n.push(if h == 0 { 0xff } else { 0 });
            n.push(h as u8 + 1);
            put16(&mut n, node.len() as u16);
            put16(&mut n, 0);
            let mut offsets = Vec::new();
            for (k, d) in node {
                offsets.push(n.len() as u16);
                n.extend_from_slice(k);
                n.extend_from_slice(d);
            }
            offsets.push(n.len() as u16);
            write_node(&mut out, *id as usize, node_size, n, &offsets);
        }
    }
    // the header node: descriptor, header record, user data record, map record
    let root = if records.is_empty() { 0 } else { *numbers.last().unwrap().first().unwrap() };
    let (first_leaf, last_leaf) = if records.is_empty() { (0, 0) } else { (numbers[0][0], numbers[0][leaf_count - 1]) };
    let mut h = Vec::with_capacity(node_size);
    put32(&mut h, 0);
    put32(&mut h, 0);
    h.push(1);
    h.push(0);
    put16(&mut h, 3);
    put16(&mut h, 0);
    let header_at = h.len() as u16;
    put16(&mut h, depth);
    put32(&mut h, root);
    put32(&mut h, records.len() as u32);
    put32(&mut h, first_leaf);
    put32(&mut h, last_leaf);
    put16(&mut h, node_size as u16);
    put16(&mut h, max_key);
    put32(&mut h, total_nodes);
    let used = if records.is_empty() { 1 } else { next_number };
    put32(&mut h, total_nodes - used);
    put16(&mut h, 0);
    put32(&mut h, node_size as u32);
    h.push(0);
    h.push(compare_type);
    put32(&mut h, attributes);
    h.extend_from_slice(&[0u8; 64]);
    let user_at = h.len() as u16;
    h.extend_from_slice(&[0u8; 128]);
    let map_at = h.len() as u16;
    let map_len = node_size - map_at as usize - 8;
    let mut map = vec![0u8; map_len];
    for i in 0..used as usize {
        map[i / 8] |= 0x80 >> (i % 8);
    }
    h.extend_from_slice(&map);
    let free_at = h.len() as u16;
    write_node(&mut out, 0, node_size, h, &[header_at, user_at, map_at, free_at]);
    out
}

fn write_node(out: &mut [u8], index: usize, node_size: usize, body: Vec<u8>, offsets: &[u16]) {
    let base = index * node_size;
    out[base..base + body.len()].copy_from_slice(&body);
    // offsets from the end of the node backwards: record 0's last
    for (i, o) in offsets.iter().enumerate() {
        let at = base + node_size - 2 * (i + 1);
        out[at..at + 2].copy_from_slice(&o.to_be_bytes());
    }
}

fn blocks_for(bytes: u64) -> u32 {
    bytes.div_ceil(BLOCK as u64) as u32
}

/// The volume as a raw image (whole-disk HFS+, no partition map).
pub fn write(v: &Volume) -> Result<Vec<u8>, String> {
    let date = (v.time + EPOCH_OFFSET) as u32;
    // walk the tree: catalog IDs in key order (parent first, then children by name)
    let mut entries: Vec<Entry> = Vec::new();
    let mut contents: Vec<Vec<u8>> = Vec::new();
    let mut next_id = FIRST_USER_ID;
    let mut paths: BTreeMap<String, u32> = BTreeMap::new();
    let Node::Folder { children, mode, finder_flags } = &v.root else { return Err("the root is a folder".into()) };
    entries.push(Entry::Folder { id: ROOT, parent: ROOT_PARENT, name: hfs_name(&v.name)?, valence: children.len() as u32, mode: *mode, finder_flags: *finder_flags });
    paths.insert(String::new(), ROOT);
    let mut stack: Vec<(u32, String, &BTreeMap<String, Node>)> = vec![(ROOT, String::new(), children)];
    let (mut files, mut folders) = (0u32, 0u32);
    while let Some((parent, path, kids)) = stack.pop() {
        let mut sorted: Vec<(&String, &Node)> = kids.iter().collect();
        let names: Vec<Vec<u16>> = sorted.iter().map(|(n, _)| hfs_name(n)).collect::<Result<_, _>>()?;
        let mut order: Vec<usize> = (0..sorted.len()).collect();
        order.sort_by(|&a, &b| compare_names(&names[a], &names[b]));
        for w in order.windows(2) {
            if compare_names(&names[w[0]], &names[w[1]]) == std::cmp::Ordering::Equal {
                return Err(format!("{path}/{}: two names differ only in case", sorted[w[1]].0));
            }
        }
        let ordered: Vec<(&String, &Node, Vec<u16>)> = order.iter().map(|&i| (sorted[i].0, sorted[i].1, names[i].clone())).collect();
        sorted.clear();
        let mut subfolders = Vec::new();
        for (name, node, hname) in ordered {
            let id = next_id;
            next_id += 1;
            let p = if path.is_empty() { name.clone() } else { format!("{path}/{name}") };
            match node {
                Node::Folder { children, mode, finder_flags } => {
                    folders += 1;
                    entries.push(Entry::Folder { id, parent, name: hname, valence: children.len() as u32, mode: *mode, finder_flags: *finder_flags });
                    paths.insert(p.clone(), id);
                    subfolders.push((id, p, children));
                }
                Node::File { data, mode, finder_flags, type_creator } => {
                    files += 1;
                    let mut tc = [0u8; 8];
                    if let Some((t, c)) = type_creator {
                        tc[..4].copy_from_slice(t);
                        tc[4..].copy_from_slice(c);
                    }
                    entries.push(Entry::File { id, parent, name: hname, mode: 0o100000 | (*mode & 0o7777), finder_flags: *finder_flags, type_creator: tc, data: contents.len() });
                    contents.push(data.clone());
                }
                Node::Symlink { target } => {
                    files += 1;
                    entries.push(Entry::File { id, parent, name: hname, mode: 0o120755, finder_flags: 0, type_creator: *b"slnkrhap", data: contents.len() });
                    contents.push(target.as_bytes().to_vec());
                }
            }
        }
        // depth-first, children in name order (the stack pops the last first)
        for s in subfolders.into_iter().rev() {
            stack.push(s);
        }
    }
    // layout: block 0 (boot blocks, volume header), allocation file, extents, catalog, attributes, data, alternate header
    let data_blocks: u64 = contents.iter().map(|c| blocks_for(c.len() as u64) as u64).sum();
    let extents_file = btree(&[], SMALL_NODE, 10, 2, 0, 0);
    let attributes_file = btree(&[], NODE, 266, 6, 0, 0);
    // the catalog needs the data forks' places, which need the catalog's size: lay out with a
    // first pass on placeholder forks (a fork record's size doesn't depend on its values)
    let catalog_records = |forks: &[Fork]| -> Vec<(Key, Vec<u8>)> { catalog(&entries, forks, date, v.uid, v.gid, &v.name) };
    let placeholder = vec![Fork::default(); contents.len()];
    let catalog_size = btree(&catalog_records(&placeholder), NODE, 516, 6, 0xCF, 0).len() as u64;
    let fixed_blocks = 1 + blocks_for(extents_file.len() as u64) as u64 + blocks_for(catalog_size) as u64 + blocks_for(attributes_file.len() as u64) as u64 + data_blocks + 1;
    let free_blocks = v.free.div_ceil(BLOCK as u64);
    // the allocation file covers every block, itself included
    let mut total = fixed_blocks + free_blocks;
    let mut alloc_blocks = 0u64;
    loop {
        let need = blocks_for((total + alloc_blocks).div_ceil(8));
        if need as u64 == alloc_blocks {
            break;
        }
        alloc_blocks = need as u64;
    }
    total += alloc_blocks;
    let total = total as u32;
    let alloc = Fork { size: alloc_blocks * BLOCK as u64, start: 1, blocks: alloc_blocks as u32 };
    let ext = Fork { size: extents_file.len() as u64, start: alloc.start + alloc.blocks, blocks: blocks_for(extents_file.len() as u64) };
    let cat = Fork { size: catalog_size, start: ext.start + ext.blocks, blocks: blocks_for(catalog_size) };
    let attr = Fork { size: attributes_file.len() as u64, start: cat.start + cat.blocks, blocks: blocks_for(attributes_file.len() as u64) };
    let mut next_block = attr.start + attr.blocks;
    let mut forks = Vec::new();
    for c in &contents {
        let b = blocks_for(c.len() as u64);
        forks.push(Fork { size: c.len() as u64, start: if b > 0 { next_block } else { 0 }, blocks: b });
        next_block += b;
    }
    let catalog_file = btree(&catalog_records(&forks), NODE, 516, 6, 0xCF, 0);
    if catalog_file.len() as u64 != catalog_size {
        return Err("the catalog's size changed between passes".into());
    }
    let used_blocks = next_block + 1;
    let mut image = vec![0u8; total as usize * BLOCK as usize];
    // the allocation bitmap: every block in use but the free ones before the last
    let mut bitmap = vec![0u8; alloc.size as usize];
    let mut mark = |b: u32| bitmap[b as usize / 8] |= 0x80 >> (b % 8);
    for b in 0..next_block {
        mark(b);
    }
    mark(total - 1);
    let put = |image: &mut Vec<u8>, f: &Fork, bytes: &[u8]| {
        let at = f.start as usize * BLOCK as usize;
        image[at..at + bytes.len()].copy_from_slice(bytes);
    };
    put(&mut image, &alloc, &bitmap);
    put(&mut image, &ext, &extents_file);
    put(&mut image, &cat, &catalog_file);
    put(&mut image, &attr, &attributes_file);
    for (f, c) in forks.iter().zip(&contents) {
        if f.blocks > 0 {
            put(&mut image, f, c);
        }
    }
    // the volume header, at 1024 and 1024 from the end
    let open = match &v.open_folder {
        Some(p) => *paths.get(p.trim_matches('/')).ok_or_else(|| format!("{p}: no such folder to open"))?,
        None => 0,
    };
    let mut h = Vec::new();
    h.extend_from_slice(b"H+");
    put16(&mut h, 4);
    put32(&mut h, 0x100); // unmounted cleanly
    h.extend_from_slice(b"10.0");
    put32(&mut h, 0); // no journal
    for _ in 0..4 {
        put32(&mut h, date); // create (local), modify, backup, checked
    }
    put32(&mut h, files);
    put32(&mut h, folders);
    put32(&mut h, BLOCK);
    put32(&mut h, total);
    put32(&mut h, total - used_blocks);
    put32(&mut h, next_block);
    put32(&mut h, 65536);
    put32(&mut h, 65536);
    put32(&mut h, next_id);
    put32(&mut h, 0);
    put64(&mut h, 1); // MacRoman
    // Finder info: blessed folder, startup app, folder to open, ... and a volume ID
    let mut finder = [0u32; 8];
    finder[2] = open;
    let digest = crc32fast::hash(&catalog_file);
    finder[6] = digest;
    finder[7] = date ^ total;
    for f in finder {
        put32(&mut h, f);
    }
    h.extend_from_slice(&alloc.bytes(BLOCK));
    h.extend_from_slice(&ext.bytes(ext.size as u32));
    h.extend_from_slice(&cat.bytes(cat.size as u32));
    h.extend_from_slice(&attr.bytes(attr.size as u32));
    h.extend_from_slice(&Fork::default().bytes(0));
    assert_eq!(h.len(), 512);
    image[1024..1536].copy_from_slice(&h);
    let alt = image.len() - 1024;
    image[alt..alt + 512].copy_from_slice(&h);
    Ok(image)
}

/// The catalog's records, sorted by key.
fn catalog(entries: &[Entry], forks: &[Fork], date: u32, uid: u32, gid: u32, volume: &str) -> Vec<(Key, Vec<u8>)> {
    let mut recs: Vec<(Key, Vec<u8>)> = Vec::new();
    let perms = |mode: u16| -> Vec<u8> {
        let mut p = Vec::new();
        put32(&mut p, uid);
        put32(&mut p, gid);
        p.push(0);
        p.push(0);
        put16(&mut p, mode);
        put32(&mut p, 0);
        p
    };
    for e in entries {
        match e {
            Entry::Folder { id, parent, name, valence, mode, finder_flags } => {
                let mut r = Vec::new();
                put16(&mut r, 1);
                put16(&mut r, 0);
                put32(&mut r, *valence);
                put32(&mut r, *id);
                for _ in 0..5 {
                    put32(&mut r, date);
                }
                r.extend_from_slice(&perms(0o040000 | (*mode & 0o7777)));
                // FolderInfo: window bounds, flags, location; ExtendedFolderInfo
                r.extend_from_slice(&[0u8; 8]);
                put16(&mut r, *finder_flags);
                r.extend_from_slice(&[0u8; 6]);
                r.extend_from_slice(&[0u8; 16]);
                put32(&mut r, 0);
                put32(&mut r, 0);
                recs.push((Key { parent: *parent, name: name.clone() }, r));
                // its thread: keyed by its ID, naming its parent and itself
                let mut t = Vec::new();
                put16(&mut t, 3);
                put16(&mut t, 0);
                put32(&mut t, *parent);
                let n = if *id == ROOT { hfs_name(volume).unwrap_or_default() } else { name.clone() };
                put16(&mut t, n.len() as u16);
                for c in &n {
                    put16(&mut t, *c);
                }
                recs.push((Key { parent: *id, name: Vec::new() }, t));
            }
            Entry::File { id, parent, name, mode, finder_flags, type_creator, data } => {
                let mut r = Vec::new();
                put16(&mut r, 2);
                put16(&mut r, 0x0002); // has a thread
                put32(&mut r, 0);
                put32(&mut r, *id);
                for _ in 0..5 {
                    put32(&mut r, date);
                }
                r.extend_from_slice(&perms(*mode));
                // FileInfo: type, creator, flags, location; ExtendedFileInfo
                r.extend_from_slice(type_creator);
                put16(&mut r, *finder_flags);
                r.extend_from_slice(&[0u8; 6]);
                r.extend_from_slice(&[0u8; 16]);
                put32(&mut r, 0);
                put32(&mut r, 0);
                r.extend_from_slice(&forks[*data].bytes(0));
                r.extend_from_slice(&Fork::default().bytes(0));
                recs.push((Key { parent: *parent, name: name.clone() }, r));
                let mut t = Vec::new();
                put16(&mut t, 4);
                put16(&mut t, 0);
                put32(&mut t, *parent);
                put16(&mut t, name.len() as u16);
                for c in name {
                    put16(&mut t, *c);
                }
                recs.push((Key { parent: *id, name: Vec::new() }, t));
            }
        }
    }
    recs.sort_by(|a, b| a.0.order(&b.0));
    recs
}

/// Read a directory into a tree (regular files, folders, symlinks; modes kept).
pub fn read_dir(path: &std::path::Path) -> Result<Node, String> {
    use std::os::unix::fs::PermissionsExt;
    let mut children = BTreeMap::new();
    let mut entries: Vec<_> = std::fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))?.collect::<Result<_, _>>().map_err(|e| e.to_string())?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().to_string();
        let p = e.path();
        let meta = std::fs::symlink_metadata(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let mode = (meta.permissions().mode() & 0o7777) as u16;
        let node = if meta.file_type().is_symlink() {
            Node::Symlink { target: std::fs::read_link(&p).map_err(|e| e.to_string())?.to_string_lossy().to_string() }
        } else if meta.is_dir() {
            match read_dir(&p)? {
                Node::Folder { children, .. } => Node::Folder { children, mode, finder_flags: 0 },
                n => n,
            }
        } else {
            Node::File { data: std::fs::read(&p).map_err(|e| e.to_string())?, mode, finder_flags: 0, type_creator: None }
        };
        children.insert(name, node);
    }
    Ok(Node::Folder { children, mode: 0o755, finder_flags: 0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Node {
        let mut app = Node::folder();
        if let Node::Folder { children, .. } = &mut app {
            children.insert("Info.plist".into(), Node::File { data: b"<plist/>".to_vec(), mode: 0o644, finder_flags: 0, type_creator: None });
        }
        let mut root = Node::folder();
        if let Node::Folder { children, .. } = &mut root {
            children.insert("Demo.app".into(), app);
            children.insert("Applications".into(), Node::Symlink { target: "/Applications".into() });
            children.insert(".background".into(), Node::Folder { children: BTreeMap::new(), mode: 0o755, finder_flags: INVISIBLE });
        }
        root
    }

    #[test]
    fn a_volume_has_its_headers_and_files() {
        let img = write(&Volume { name: "Demo".into(), root: tree(), time: 1_700_000_000, uid: 501, gid: 20, open_folder: Some(String::new()), free: 0 }).unwrap();
        assert_eq!(img.len() % BLOCK as usize, 0);
        assert_eq!(&img[1024..1026], b"H+");
        assert_eq!(&img[img.len() - 1024..img.len() - 1022], b"H+");
        // two files (Info.plist, the symlink) and two folders besides the root
        assert_eq!(u32::from_be_bytes(img[1024 + 32..1024 + 36].try_into().unwrap()), 2);
        assert_eq!(u32::from_be_bytes(img[1024 + 36..1024 + 40].try_into().unwrap()), 2);
        // the folder to open is the root
        assert_eq!(u32::from_be_bytes(img[1024 + 80 + 8..1024 + 80 + 12].try_into().unwrap()), ROOT);
        // the symlink's target is in its data fork
        assert!(img.windows(13).any(|w| w == b"/Applications"));
    }

    #[test]
    fn names_order_without_case() {
        let a = hfs_name("applications").unwrap();
        let b = hfs_name("Demo.app").unwrap();
        assert_eq!(compare_names(&a, &b), std::cmp::Ordering::Less);
        assert_eq!(compare_names(&hfs_name("A").unwrap(), &hfs_name("a").unwrap()), std::cmp::Ordering::Equal);
    }

    #[test]
    fn same_inputs_same_bytes() {
        let v = || Volume { name: "Demo".into(), root: tree(), time: 1_700_000_000, uid: 501, gid: 20, open_folder: None, free: 1 << 20 };
        assert_eq!(write(&v()).unwrap(), write(&v()).unwrap());
    }
}
