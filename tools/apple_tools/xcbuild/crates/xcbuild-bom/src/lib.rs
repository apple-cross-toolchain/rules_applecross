use byteorder::{BigEndian, ByteOrder};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BomError {
    #[error("file too small")]
    FileTooSmall,
    #[error("invalid magic (expected 'BOMStore')")]
    InvalidMagic,
    #[error("invalid version (expected 1, got {0})")]
    InvalidVersion(u32),
    #[error("index offset out of bounds")]
    IndexOutOfBounds,
    #[error("variables offset out of bounds")]
    VariablesOutOfBounds,
    #[error("index {0} out of range")]
    IndexOutOfRange(u32),
    #[error("tree not found for variable '{0}'")]
    TreeNotFound(String),
    #[error("invalid tree magic")]
    InvalidTreeMagic,
    #[error("invalid tree version")]
    InvalidTreeVersion,
    #[error("data extends beyond buffer")]
    DataOutOfBounds,
}

const BOM_MAGIC: &[u8; 8] = b"BOMStore";
const TREE_MAGIC: &[u8; 4] = b"tree";

const HEADER_SIZE: usize = 32;
const INDEX_ENTRY_SIZE: usize = 8;
const TREE_HEADER_SIZE: usize = 21;
const TREE_ENTRY_HEADER_SIZE: usize = 12;
const TREE_ENTRY_INDEX_SIZE: usize = 8;

/// A read-only BOM archive context.
#[derive(Debug, Clone)]
pub struct Bom {
    data: Vec<u8>,
    index_offset: usize,
    index_count: u32,
    variables_offset: usize,
}

#[derive(Debug, Clone)]
pub struct BomIndex {
    pub address: u32,
    pub length: u32,
}

#[derive(Debug, Clone)]
pub struct BomVariable {
    pub name: String,
    pub index: u32,
}

#[derive(Debug, Clone)]
pub struct BomTreeEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl Bom {
    pub fn load(data: Vec<u8>) -> Result<Self, BomError> {
        if data.len() < HEADER_SIZE {
            return Err(BomError::FileTooSmall);
        }

        if &data[0..8] != BOM_MAGIC {
            return Err(BomError::InvalidMagic);
        }

        let version = BigEndian::read_u32(&data[8..12]);
        if version != 1 {
            return Err(BomError::InvalidVersion(version));
        }

        let index_offset = BigEndian::read_u32(&data[16..20]) as usize;
        let index_length = BigEndian::read_u32(&data[20..24]) as usize;

        if index_offset + 4 > data.len() {
            return Err(BomError::IndexOutOfBounds);
        }
        if index_offset + index_length > data.len() {
            return Err(BomError::IndexOutOfBounds);
        }

        let variables_offset = BigEndian::read_u32(&data[24..28]) as usize;
        if variables_offset + 4 > data.len() {
            return Err(BomError::VariablesOutOfBounds);
        }

        let index_count = BigEndian::read_u32(&data[index_offset..index_offset + 4]);

        Ok(Bom {
            data,
            index_offset,
            index_count,
            variables_offset,
        })
    }

    pub fn block_count(&self) -> u32 {
        BigEndian::read_u32(&self.data[12..16])
    }

    pub fn index_length(&self) -> u32 {
        BigEndian::read_u32(&self.data[20..24])
    }

    pub fn trailer_len(&self) -> u32 {
        BigEndian::read_u32(&self.data[28..32])
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn index_get(&self, index: u32) -> Option<&[u8]> {
        if index >= self.index_count {
            return None;
        }

        let entry_offset = self.index_offset + 4 + (index as usize) * INDEX_ENTRY_SIZE;
        if entry_offset + INDEX_ENTRY_SIZE > self.data.len() {
            return None;
        }

        let address = BigEndian::read_u32(&self.data[entry_offset..entry_offset + 4]) as usize;
        let length = BigEndian::read_u32(&self.data[entry_offset + 4..entry_offset + 8]) as usize;

        if address + length > self.data.len() {
            return None;
        }

        Some(&self.data[address..address + length])
    }

    pub fn indices(&self) -> Vec<(u32, BomIndex)> {
        let mut result = Vec::new();
        for i in 0..self.index_count {
            let entry_offset = self.index_offset + 4 + (i as usize) * INDEX_ENTRY_SIZE;
            if entry_offset + INDEX_ENTRY_SIZE > self.data.len() {
                break;
            }
            let address = BigEndian::read_u32(&self.data[entry_offset..entry_offset + 4]);
            let length = BigEndian::read_u32(&self.data[entry_offset + 4..entry_offset + 8]);
            result.push((i, BomIndex { address, length }));
        }
        result
    }

    pub fn variables(&self) -> Vec<BomVariable> {
        let mut result = Vec::new();
        let vars_offset = self.variables_offset;

        if vars_offset + 4 > self.data.len() {
            return result;
        }

        let count = BigEndian::read_u32(&self.data[vars_offset..vars_offset + 4]);
        let mut offset = vars_offset + 4;

        for _ in 0..count {
            if offset + 5 > self.data.len() {
                break;
            }

            let index = BigEndian::read_u32(&self.data[offset..offset + 4]);
            let name_len = self.data[offset + 4] as usize;
            offset += 5;

            if offset + name_len > self.data.len() {
                break;
            }

            let name = String::from_utf8_lossy(&self.data[offset..offset + name_len]).to_string();
            offset += name_len;

            result.push(BomVariable { name, index });
        }

        result
    }

    pub fn variable_get(&self, name: &str) -> Option<u32> {
        self.variables()
            .iter()
            .find(|v| v.name == name)
            .map(|v| v.index)
    }

    pub fn tree_entries(&self, variable_name: &str) -> Result<Vec<BomTreeEntry>, BomError> {
        let tree_index = self
            .variable_get(variable_name)
            .ok_or_else(|| BomError::TreeNotFound(variable_name.to_string()))?;

        let tree_data = self
            .index_get(tree_index)
            .ok_or(BomError::IndexOutOfRange(tree_index))?;

        if tree_data.len() < TREE_HEADER_SIZE {
            return Err(BomError::InvalidTreeMagic);
        }

        if &tree_data[0..4] != TREE_MAGIC {
            return Err(BomError::InvalidTreeMagic);
        }

        let tree_version = BigEndian::read_u32(&tree_data[4..8]);
        if tree_version != 1 {
            return Err(BomError::InvalidTreeVersion);
        }

        let child_index = BigEndian::read_u32(&tree_data[8..12]);

        let mut entries = Vec::new();
        self.collect_tree_entries(child_index, &mut entries)?;
        Ok(entries)
    }

    /// Check if a variable holds a valid tree.
    pub fn is_tree(&self, variable_index: u32) -> bool {
        if let Some(data) = self.index_get(variable_index) {
            if data.len() >= TREE_HEADER_SIZE && &data[0..4] == TREE_MAGIC {
                let version = BigEndian::read_u32(&data[4..8]);
                return version == 1;
            }
        }
        false
    }

    fn collect_tree_entries(
        &self,
        entry_index: u32,
        entries: &mut Vec<BomTreeEntry>,
    ) -> Result<(), BomError> {
        let entry_data = self
            .index_get(entry_index)
            .ok_or(BomError::IndexOutOfRange(entry_index))?;

        if entry_data.len() < TREE_ENTRY_HEADER_SIZE {
            return Err(BomError::DataOutOfBounds);
        }

        let is_leaf = BigEndian::read_u16(&entry_data[0..2]);
        let count = BigEndian::read_u16(&entry_data[2..4]) as usize;
        let forward = BigEndian::read_u32(&entry_data[4..8]);

        if is_leaf == 0 {
            // Non-leaf: follow the first child's value_index
            if count > 0 && entry_data.len() >= TREE_ENTRY_HEADER_SIZE + TREE_ENTRY_INDEX_SIZE {
                let value_index = BigEndian::read_u32(
                    &entry_data[TREE_ENTRY_HEADER_SIZE..TREE_ENTRY_HEADER_SIZE + 4],
                );
                self.collect_tree_entries(value_index, entries)?;
            }
        } else {
            // Leaf: collect all key/value entries
            for i in 0..count {
                let idx_offset = TREE_ENTRY_HEADER_SIZE + i * TREE_ENTRY_INDEX_SIZE;
                if idx_offset + TREE_ENTRY_INDEX_SIZE > entry_data.len() {
                    break;
                }

                let value_index =
                    BigEndian::read_u32(&entry_data[idx_offset..idx_offset + 4]);
                let key_index =
                    BigEndian::read_u32(&entry_data[idx_offset + 4..idx_offset + 8]);

                let key_data = self.index_get(key_index).unwrap_or(&[]);
                let value_data = self.index_get(value_index).unwrap_or(&[]);

                entries.push(BomTreeEntry {
                    key: key_data.to_vec(),
                    value: value_data.to_vec(),
                });
            }

            // Follow forward pointer to next leaf
            if forward != 0 {
                self.collect_tree_entries(forward, entries)?;
            }
        }

        Ok(())
    }
}

/// BOM path structures for lsbom
pub mod paths {
    use byteorder::{BigEndian, ByteOrder};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PathType {
        File,
        Directory,
        Link,
        Device,
    }

    #[derive(Debug, Clone)]
    pub struct PathInfo1 {
        pub id: u32,
        pub index: u32,
    }

    #[derive(Debug, Clone)]
    pub struct PathInfo2 {
        pub path_type: u8,
        pub architecture: u16,
        pub mode: u16,
        pub user: u32,
        pub group: u32,
        pub modtime: u32,
        pub size: u32,
        pub checksum: u32,
        pub link_name: String,
    }

    #[derive(Debug, Clone)]
    pub struct FileKey {
        pub parent: u32,
        pub name: String,
    }

    impl PathInfo1 {
        pub fn from_bytes(data: &[u8]) -> Option<Self> {
            if data.len() < 8 {
                return None;
            }
            Some(PathInfo1 {
                id: BigEndian::read_u32(&data[0..4]),
                index: BigEndian::read_u32(&data[4..8]),
            })
        }
    }

    impl PathInfo2 {
        pub fn from_bytes(data: &[u8]) -> Option<Self> {
            if data.len() < 22 {
                return None;
            }
            let path_type = data[0];
            let architecture = BigEndian::read_u16(&data[2..4]);
            let mode = BigEndian::read_u16(&data[4..6]);
            let user = BigEndian::read_u32(&data[6..10]);
            let group = BigEndian::read_u32(&data[10..14]);
            let modtime = BigEndian::read_u32(&data[14..18]);
            let size = BigEndian::read_u32(&data[18..22]);
            let checksum = if data.len() >= 27 {
                BigEndian::read_u32(&data[23..27])
            } else {
                0
            };
            let link_name = if data.len() >= 31 {
                let link_len = BigEndian::read_u32(&data[27..31]) as usize;
                if data.len() >= 31 + link_len {
                    String::from_utf8_lossy(&data[31..31 + link_len]).to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            Some(PathInfo2 {
                path_type,
                architecture,
                mode,
                user,
                group,
                modtime,
                size,
                checksum,
                link_name,
            })
        }

        pub fn path_type(&self) -> PathType {
            match self.path_type {
                1 => PathType::File,
                2 => PathType::Directory,
                3 => PathType::Link,
                4 => PathType::Device,
                _ => PathType::File,
            }
        }
    }

    impl FileKey {
        pub fn from_bytes(data: &[u8]) -> Option<Self> {
            if data.len() < 5 {
                return None;
            }
            let parent = BigEndian::read_u32(&data[0..4]);
            let name_bytes = &data[4..];
            let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len());
            let name = String::from_utf8_lossy(&name_bytes[..name_end]).to_string();
            Some(FileKey { parent, name })
        }
    }

    /// Resolve the full path for a file by walking up the parent chain.
    pub fn resolve_path(
        file_key: &FileKey,
        files: &HashMap<u32, (u32, String)>,
    ) -> String {
        let mut path = file_key.name.clone();
        let mut next = file_key.parent;
        while let Some((parent, name)) = files.get(&next) {
            path = format!("{name}/{path}");
            next = *parent;
        }
        path
    }
}

/// A BOM writer for creating new BOM archives.
pub struct BomWriter {
    /// Stored data blocks (index 0 is unused/null).
    blocks: Vec<Vec<u8>>,
    /// Named variables mapping to block indices.
    variables: Vec<(String, u32)>,
}

impl BomWriter {
    pub fn new() -> Self {
        // Block 0 is reserved (null block)
        BomWriter {
            blocks: vec![vec![]],
            variables: Vec::new(),
        }
    }

    /// Add a data block and return its index.
    pub fn add_block(&mut self, data: Vec<u8>) -> u32 {
        let idx = self.blocks.len() as u32;
        self.blocks.push(data);
        idx
    }

    /// Add a named variable pointing to a block index.
    pub fn add_variable(&mut self, name: &str, index: u32) {
        self.variables.push((name.to_string(), index));
    }

    /// Build a BOM tree from key-value pairs and return the block index of the tree root.
    pub fn build_tree(&mut self, entries: &[(Vec<u8>, Vec<u8>)]) -> u32 {
        // Store each key and value as separate blocks
        // Build leaf nodes containing entry indices

        const ENTRIES_PER_LEAF: usize = 256;

        let mut leaf_indices: Vec<u32> = Vec::new();

        for chunk in entries.chunks(ENTRIES_PER_LEAF) {
            let count = chunk.len() as u16;
            let is_leaf: u16 = 1;
            let forward: u32 = 0; // will be patched below

            let mut leaf_data = Vec::new();
            leaf_data.extend_from_slice(&is_leaf.to_be_bytes());
            leaf_data.extend_from_slice(&count.to_be_bytes());
            leaf_data.extend_from_slice(&forward.to_be_bytes());
            // 4 bytes padding (backward pointer)
            leaf_data.extend_from_slice(&0u32.to_be_bytes());

            for (key, value) in chunk {
                let value_idx = self.add_block(value.clone());
                let key_idx = self.add_block(key.clone());
                leaf_data.extend_from_slice(&value_idx.to_be_bytes());
                leaf_data.extend_from_slice(&key_idx.to_be_bytes());
            }

            let leaf_idx = self.add_block(leaf_data);
            leaf_indices.push(leaf_idx);
        }

        // Patch forward pointers in leaf nodes
        for i in 0..leaf_indices.len() - 1 {
            let next_idx = leaf_indices[i + 1];
            let leaf = &mut self.blocks[leaf_indices[i] as usize];
            // Forward pointer is at offset 4..8
            leaf[4..8].copy_from_slice(&next_idx.to_be_bytes());
        }

        let child_index = if leaf_indices.is_empty() {
            // Empty tree: create an empty leaf
            let mut leaf_data = Vec::new();
            leaf_data.extend_from_slice(&1u16.to_be_bytes()); // is_leaf
            leaf_data.extend_from_slice(&0u16.to_be_bytes()); // count
            leaf_data.extend_from_slice(&0u32.to_be_bytes()); // forward
            leaf_data.extend_from_slice(&0u32.to_be_bytes()); // backward
            self.add_block(leaf_data)
        } else {
            leaf_indices[0]
        };

        let entry_count = entries.len() as u32;

        // Build tree header
        let mut tree_data = Vec::new();
        tree_data.extend_from_slice(TREE_MAGIC);
        tree_data.extend_from_slice(&1u32.to_be_bytes()); // version
        tree_data.extend_from_slice(&child_index.to_be_bytes());
        // block size (unused, set to 4096)
        tree_data.extend_from_slice(&4096u32.to_be_bytes());
        // path count
        tree_data.extend_from_slice(&entry_count.to_be_bytes());
        // unknown byte
        tree_data.push(0);

        self.add_block(tree_data)
    }

    /// Serialize the BOM to bytes.
    pub fn serialize(&self) -> Vec<u8> {
        // Calculate layout
        // Header: 32 bytes
        // Then data blocks
        // Then index table
        // Then variables table

        let num_blocks = self.blocks.len() as u32;

        // Calculate data block positions (starting after header)
        let mut block_offsets: Vec<(u32, u32)> = Vec::new(); // (offset, length)
        let mut data_offset: u32 = HEADER_SIZE as u32;

        for block in &self.blocks {
            block_offsets.push((data_offset, block.len() as u32));
            data_offset += block.len() as u32;
        }

        // Index table
        let index_offset = data_offset;
        // Index table: 4 bytes count + num_blocks * 8 bytes
        let index_size = 4 + num_blocks * INDEX_ENTRY_SIZE as u32;

        // Variables table
        let vars_offset = index_offset + index_size;
        let mut vars_data = Vec::new();
        vars_data.extend_from_slice(&(self.variables.len() as u32).to_be_bytes());
        for (name, index) in &self.variables {
            vars_data.extend_from_slice(&index.to_be_bytes());
            vars_data.push(name.len() as u8);
            vars_data.extend_from_slice(name.as_bytes());
        }

        let total_size = vars_offset + vars_data.len() as u32;

        // Build the output
        let mut output = Vec::with_capacity(total_size as usize);

        // Header (32 bytes)
        output.extend_from_slice(BOM_MAGIC); // magic
        output.extend_from_slice(&1u32.to_be_bytes()); // version
        output.extend_from_slice(&num_blocks.to_be_bytes()); // block count
        output.extend_from_slice(&index_offset.to_be_bytes()); // index offset
        output.extend_from_slice(&index_size.to_be_bytes()); // index length
        output.extend_from_slice(&vars_offset.to_be_bytes()); // variables offset
        output.extend_from_slice(&(vars_data.len() as u32).to_be_bytes()); // variables length

        // Data blocks
        for block in &self.blocks {
            output.extend_from_slice(block);
        }

        // Index table
        output.extend_from_slice(&num_blocks.to_be_bytes());
        for (offset, length) in &block_offsets {
            output.extend_from_slice(&offset.to_be_bytes());
            output.extend_from_slice(&length.to_be_bytes());
        }

        // Variables table
        output.extend_from_slice(&vars_data);

        output
    }
}

impl Default for BomWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl paths::PathInfo1 {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(8);
        data.extend_from_slice(&self.id.to_be_bytes());
        data.extend_from_slice(&self.index.to_be_bytes());
        data
    }
}

impl paths::PathInfo2 {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(31);
        data.push(self.path_type);
        data.push(0); // padding
        data.extend_from_slice(&self.architecture.to_be_bytes());
        data.extend_from_slice(&self.mode.to_be_bytes());
        data.extend_from_slice(&self.user.to_be_bytes());
        data.extend_from_slice(&self.group.to_be_bytes());
        data.extend_from_slice(&self.modtime.to_be_bytes());
        data.extend_from_slice(&self.size.to_be_bytes());
        data.push(1); // device type (regular)
        data.extend_from_slice(&self.checksum.to_be_bytes());
        let link_bytes = self.link_name.as_bytes();
        data.extend_from_slice(&(link_bytes.len() as u32).to_be_bytes());
        data.extend_from_slice(link_bytes);
        data
    }
}

impl paths::FileKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(5 + self.name.len());
        data.extend_from_slice(&self.parent.to_be_bytes());
        data.extend_from_slice(self.name.as_bytes());
        data.push(0); // null terminator
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_magic() {
        let data = vec![0u8; 64];
        assert!(Bom::load(data).is_err());
    }

    #[test]
    fn test_too_small() {
        let data = vec![0u8; 4];
        assert!(Bom::load(data).is_err());
    }

    #[test]
    fn test_bom_writer_roundtrip() {
        let mut writer = BomWriter::new();

        // Create a simple tree with one entry
        let key = paths::FileKey {
            parent: 0,
            name: "test.txt".to_string(),
        };
        let info1 = paths::PathInfo1 { id: 1, index: 0 };
        let info2 = paths::PathInfo2 {
            path_type: 1,
            architecture: 0,
            mode: 0o100644,
            user: 501,
            group: 20,
            modtime: 0,
            size: 100,
            checksum: 0,
            link_name: String::new(),
        };

        let info2_idx = writer.add_block(info2.to_bytes());
        let info1_with_index = paths::PathInfo1 {
            id: info1.id,
            index: info2_idx,
        };

        let entries = vec![(key.to_bytes(), info1_with_index.to_bytes())];
        let tree_idx = writer.build_tree(&entries);
        writer.add_variable("Paths", tree_idx);

        let data = writer.serialize();
        let bom = Bom::load(data).expect("should load written BOM");

        let read_entries = bom.tree_entries("Paths").expect("should read Paths tree");
        assert_eq!(read_entries.len(), 1);
    }
}
