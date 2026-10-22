use byteorder::{LittleEndian, ByteOrder};
use std::collections::HashMap;
use std::fs;
use xcbuild_bom::Bom;

/// CAR header variable names.
const CAR_HEADER_VAR: &str = "CARHEADER";
const CAR_KEY_FORMAT_VAR: &str = "KEYFORMAT";
const CAR_FACET_KEYS_VAR: &str = "FACETKEYS";
const CAR_RENDITIONS_VAR: &str = "RENDITIONS";

/// CAR magic: "RATC"
const CAR_MAGIC: &[u8; 4] = b"RATC";

/// Key format magic: "tmfk"
const KEY_FORMAT_MAGIC: &[u8; 4] = b"tmfk";

/// Known attribute identifiers.
pub const ATTR_ELEMENT: u16 = 1;
pub const ATTR_PART: u16 = 2;
pub const ATTR_SIZE: u16 = 3;
pub const ATTR_DIRECTION: u16 = 4;
pub const ATTR_VALUE: u16 = 6;
pub const ATTR_DIMENSION1: u16 = 8;
pub const ATTR_DIMENSION2: u16 = 9;
pub const ATTR_STATE: u16 = 10;
pub const ATTR_LAYER: u16 = 11;
pub const ATTR_SCALE: u16 = 12;
pub const ATTR_PRESENTATION_STATE: u16 = 14;
pub const ATTR_IDIOM: u16 = 15;
pub const ATTR_SUBTYPE: u16 = 16;
pub const ATTR_IDENTIFIER: u16 = 17;
pub const ATTR_PREVIOUS_VALUE: u16 = 18;
pub const ATTR_PREVIOUS_STATE: u16 = 19;
pub const ATTR_SIZE_CLASS_H: u16 = 20;
pub const ATTR_SIZE_CLASS_V: u16 = 21;
pub const ATTR_MEMORY_CLASS: u16 = 22;
pub const ATTR_GRAPHICS_CLASS: u16 = 23;
pub const ATTR_DISPLAY_GAMUT: u16 = 24;
pub const ATTR_DEPLOYMENT_TARGET: u16 = 25;

/// Attribute identifier names.
pub fn attribute_name(id: u16) -> &'static str {
    match id {
        ATTR_ELEMENT => "element",
        ATTR_PART => "part",
        ATTR_SIZE => "size",
        ATTR_DIRECTION => "direction",
        ATTR_VALUE => "value",
        ATTR_DIMENSION1 => "dimension1",
        ATTR_DIMENSION2 => "dimension2",
        ATTR_STATE => "state",
        ATTR_LAYER => "layer",
        ATTR_SCALE => "scale",
        ATTR_PRESENTATION_STATE => "presentation_state",
        ATTR_IDIOM => "idiom",
        ATTR_SUBTYPE => "subtype",
        ATTR_IDENTIFIER => "identifier",
        ATTR_PREVIOUS_VALUE => "previous_value",
        ATTR_PREVIOUS_STATE => "previous_state",
        ATTR_SIZE_CLASS_H => "size_class_horizontal",
        ATTR_SIZE_CLASS_V => "size_class_vertical",
        ATTR_MEMORY_CLASS => "memory_class",
        ATTR_GRAPHICS_CLASS => "graphics_class",
        ATTR_DISPLAY_GAMUT => "display_gamut",
        ATTR_DEPLOYMENT_TARGET => "deployment_target",
        _ => "unknown",
    }
}

/// A set of attribute key-value pairs.
#[derive(Debug, Clone)]
pub struct AttributeList {
    pub attrs: HashMap<u16, u16>,
}

impl AttributeList {
    /// Get an attribute value by identifier.
    pub fn get(&self, id: u16) -> Option<u16> {
        self.attrs.get(&id).copied()
    }

    /// Load from a list of identifiers and corresponding key values.
    pub fn from_key_format(identifiers: &[u32], key_values: &[u16]) -> Self {
        let mut attrs = HashMap::new();
        for (i, &id) in identifiers.iter().enumerate() {
            if i < key_values.len() {
                attrs.insert(id as u16, key_values[i]);
            }
        }
        AttributeList { attrs }
    }

    /// Load from attribute pairs in a facet value.
    pub fn from_pairs(pairs: &[(u16, u16)]) -> Self {
        let mut attrs = HashMap::new();
        for &(id, val) in pairs {
            attrs.insert(id, val);
        }
        AttributeList { attrs }
    }
}

/// Parsed CAR header.
#[derive(Debug, Clone)]
pub struct CarHeader {
    pub ui_version: u32,
    pub storage_version: u32,
    pub storage_timestamp: u32,
    pub rendition_count: u32,
    pub file_creator: String,
    pub other_creator: String,
    pub uuid: [u8; 16],
    pub associated_checksum: u32,
    pub schema_version: u32,
    pub color_space_id: u32,
    pub key_semantics: u32,
}

impl CarHeader {
    fn parse(data: &[u8]) -> Option<Self> {
        // car_header is: 4 magic + 4 ui_version + 4 storage_version + 4 storage_timestamp
        //   + 4 rendition_count + 128 file_creator + 256 other_creator + 16 uuid
        //   + 4 associated_checksum + 4 schema_version + 4 color_space_id + 4 key_semantics
        // Total: 432 bytes
        if data.len() < 432 {
            return None;
        }
        if &data[0..4] != CAR_MAGIC {
            return None;
        }
        let file_creator = {
            let end = data[20..148]
                .iter()
                .position(|&b| b == 0 || b == 0x0A)
                .unwrap_or(128);
            String::from_utf8_lossy(&data[20..20 + end]).to_string()
        };
        let other_creator = {
            let end = data[148..404]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(256);
            String::from_utf8_lossy(&data[148..148 + end]).to_string()
        };
        let mut uuid = [0u8; 16];
        uuid.copy_from_slice(&data[404..420]);

        Some(CarHeader {
            ui_version: LittleEndian::read_u32(&data[4..8]),
            storage_version: LittleEndian::read_u32(&data[8..12]),
            storage_timestamp: LittleEndian::read_u32(&data[12..16]),
            rendition_count: LittleEndian::read_u32(&data[16..20]),
            file_creator,
            other_creator,
            uuid,
            associated_checksum: LittleEndian::read_u32(&data[420..424]),
            schema_version: LittleEndian::read_u32(&data[424..428]),
            color_space_id: LittleEndian::read_u32(&data[428..432]),
            key_semantics: LittleEndian::read_u32(&data[432..436]),
        })
    }
}

/// A facet (named asset) in the CAR archive.
#[derive(Debug, Clone)]
pub struct Facet {
    pub name: String,
    pub attributes: AttributeList,
}

impl Facet {
    /// Get the facet's identifier attribute.
    pub fn identifier(&self) -> Option<u16> {
        self.attributes.get(ATTR_IDENTIFIER)
    }
}

/// A rendition (specific variant of an asset).
#[derive(Debug, Clone)]
pub struct Rendition {
    pub attributes: AttributeList,
    pub width: u32,
    pub height: u32,
    pub scale_factor: u32,
    pub pixel_format: [u8; 4],
    pub name: String,
    pub layout: u16,
    pub data_length: u32,
}

impl Rendition {
    fn parse(key_data: &[u8], value_data: &[u8], identifiers: &[u32]) -> Option<Self> {
        // Key is a list of u16 values corresponding to identifiers
        let num_keys = key_data.len() / 2;
        let mut key_values = Vec::with_capacity(num_keys);
        for i in 0..num_keys {
            key_values.push(LittleEndian::read_u16(&key_data[i * 2..i * 2 + 2]));
        }
        let attributes = AttributeList::from_key_format(identifiers, &key_values);

        // Value starts with "CTSI" magic
        // car_rendition_value layout:
        // 0: magic[4] "CTSI"
        // 4: version u32
        // 8: flags u32
        // 12: width u32
        // 16: height u32
        // 20: scale_factor u32
        // 24: pixel_format u32
        // 28: color_space_id:4 + reserved:28 (u32)
        // 32: metadata.modification_date u32
        // 36: metadata.layout u16
        // 38: metadata.reserved u16
        // 40: metadata.name[128]
        // 168: info_len u32
        // 172: bitmaps.bitmap_count u32
        // 176: bitmaps.reserved u32
        // 180: bitmaps.payload_size u32
        // 184: info[0]...
        if value_data.len() < 184 {
            return None;
        }
        if &value_data[0..4] != b"CTSI" {
            return None;
        }

        let width = LittleEndian::read_u32(&value_data[12..16]);
        let height = LittleEndian::read_u32(&value_data[16..20]);
        let scale_factor = LittleEndian::read_u32(&value_data[20..24]);
        let mut pixel_format = [0u8; 4];
        pixel_format.copy_from_slice(&value_data[24..28]);
        let layout = LittleEndian::read_u16(&value_data[36..38]);

        let name_end = value_data[40..168]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(128);
        let name = String::from_utf8_lossy(&value_data[40..40 + name_end]).to_string();

        let data_length = LittleEndian::read_u32(&value_data[180..184]);

        Some(Rendition {
            attributes,
            width,
            height,
            scale_factor,
            pixel_format,
            name,
            layout,
            data_length,
        })
    }

    pub fn scale(&self) -> f32 {
        self.scale_factor as f32 / 100.0
    }

    pub fn pixel_format_string(&self) -> String {
        let pf = &self.pixel_format;
        if pf == b"ARGB" {
            "ARGB".to_string()
        } else if pf == b"GA8 " {
            "GA8".to_string()
        } else if pf == b"PDF " {
            "PDF".to_string()
        } else if pf == b"DATA" {
            "DATA".to_string()
        } else if pf == b"JPEG" {
            "JPEG".to_string()
        } else if pf == b"WEBP" {
            "WEBP".to_string()
        } else {
            format!(
                "{:02x}{:02x}{:02x}{:02x}",
                pf[0], pf[1], pf[2], pf[3]
            )
        }
    }

    pub fn file_name(&self) -> String {
        if self.name.is_empty() {
            format!(
                "rendition_{}",
                self.attributes.get(ATTR_IDENTIFIER).unwrap_or(0)
            )
        } else {
            self.name.clone()
        }
    }
}

/// A parsed CAR archive reader.
#[derive(Debug)]
pub struct CarReader {
    pub header: CarHeader,
    pub key_format_identifiers: Vec<u32>,
    pub facets: Vec<Facet>,
    pub renditions: Vec<Rendition>,
    bom: Bom,
}

impl CarReader {
    /// Load a CAR archive from a file path.
    pub fn open(path: &str) -> Option<CarReader> {
        let data = fs::read(path).ok()?;
        Self::load(data)
    }

    /// Load a CAR archive from raw data.
    pub fn load(data: Vec<u8>) -> Option<CarReader> {
        let bom = Bom::load(data).ok()?;

        // Read CARHEADER
        let header_index = bom.variable_get(CAR_HEADER_VAR)?;
        let header_data = bom.index_get(header_index)?;
        let header = CarHeader::parse(header_data)?;

        if header.storage_version < 8 {
            return None;
        }

        // Read KEYFORMAT
        let keyfmt_index = bom.variable_get(CAR_KEY_FORMAT_VAR)?;
        let keyfmt_data = bom.index_get(keyfmt_index)?;
        let identifiers = parse_key_format(keyfmt_data)?;

        // Read FACETKEYS
        let facets = if let Ok(entries) = bom.tree_entries(CAR_FACET_KEYS_VAR) {
            entries
                .iter()
                .filter_map(|entry| {
                    let name_end = entry
                        .key
                        .iter()
                        .position(|&b| b == 0)
                        .unwrap_or(entry.key.len());
                    let name =
                        String::from_utf8_lossy(&entry.key[..name_end]).to_string();
                    let attrs = parse_facet_value(&entry.value);
                    Some(Facet {
                        name,
                        attributes: attrs,
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        // Read RENDITIONS
        let renditions = if let Ok(entries) = bom.tree_entries(CAR_RENDITIONS_VAR) {
            entries
                .iter()
                .filter_map(|entry| {
                    Rendition::parse(&entry.key, &entry.value, &identifiers)
                })
                .collect()
        } else {
            Vec::new()
        };

        Some(CarReader {
            header,
            key_format_identifiers: identifiers,
            facets,
            renditions,
            bom,
        })
    }

    /// Look up renditions for a given facet.
    pub fn lookup_renditions(&self, facet: &Facet) -> Vec<&Rendition> {
        let facet_id = match facet.identifier() {
            Some(id) => id,
            None => return Vec::new(),
        };

        self.renditions
            .iter()
            .filter(|r| r.attributes.get(ATTR_IDENTIFIER) == Some(facet_id))
            .collect()
    }

    /// Get the list of BOM variables in the archive.
    pub fn variables(&self) -> Vec<(String, usize)> {
        self.bom
            .variables()
            .iter()
            .map(|v| {
                let size = self
                    .bom
                    .index_get(v.index)
                    .map(|d| d.len())
                    .unwrap_or(0);
                (v.name.clone(), size)
            })
            .collect()
    }

    /// Dump CAR header info.
    pub fn dump_header(&self) {
        let h = &self.header;
        println!("Magic: RATC");
        println!("UI version: {:x}", h.ui_version);
        println!("Storage version: {:x}", h.storage_version);
        println!("Storage Timestamp: {:x}", h.storage_timestamp);
        println!("Rendition Count: {:x}", h.rendition_count);
        println!("Creator: {}", h.file_creator);
        println!("Other Creator: {}", h.other_creator);
        println!(
            "UUID: {:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            h.uuid[0], h.uuid[1], h.uuid[2], h.uuid[3],
            h.uuid[4], h.uuid[5],
            h.uuid[6], h.uuid[7],
            h.uuid[8], h.uuid[9],
            h.uuid[10], h.uuid[11], h.uuid[12], h.uuid[13], h.uuid[14], h.uuid[15]
        );
        println!("Associated Checksum: {:x}", h.associated_checksum);
        println!("Schema Version: {:x}", h.schema_version);
        println!("Color space ID: {:x}", h.color_space_id);
        println!("Key Semantics: {:x}", h.key_semantics);
    }

    /// Dump key format info.
    pub fn dump_key_format(&self) {
        println!("Identifier Count: {}", self.key_format_identifiers.len());
        for &id in &self.key_format_identifiers {
            println!("  Identifier: {} ({})", attribute_name(id as u16), id);
        }
    }
}

fn parse_key_format(data: &[u8]) -> Option<Vec<u32>> {
    if data.len() < 12 {
        return None;
    }
    if &data[0..4] != KEY_FORMAT_MAGIC {
        return None;
    }
    let num = LittleEndian::read_u32(&data[8..12]) as usize;
    let mut identifiers = Vec::with_capacity(num);
    for i in 0..num {
        let offset = 12 + i * 4;
        if offset + 4 > data.len() {
            break;
        }
        identifiers.push(LittleEndian::read_u32(&data[offset..offset + 4]));
    }
    Some(identifiers)
}

fn parse_facet_value(data: &[u8]) -> AttributeList {
    // Facet value: 4 bytes hotspot + 2 bytes count + N attribute pairs (4 bytes each)
    if data.len() < 6 {
        return AttributeList {
            attrs: HashMap::new(),
        };
    }
    let count = LittleEndian::read_u16(&data[4..6]) as usize;
    let mut pairs = Vec::new();
    for i in 0..count {
        let offset = 6 + i * 4;
        if offset + 4 > data.len() {
            break;
        }
        let id = LittleEndian::read_u16(&data[offset..offset + 2]);
        let val = LittleEndian::read_u16(&data[offset + 2..offset + 4]);
        pairs.push((id, val));
    }
    AttributeList::from_pairs(&pairs)
}

/// Dump a facet for display.
pub fn dump_facet(facet: &Facet) {
    print!("Facet: {}", facet.name);
    if let Some(id) = facet.identifier() {
        print!(" (id={})", id);
    }
    println!();
    for (&attr_id, &attr_val) in &facet.attributes.attrs {
        if attr_id != ATTR_IDENTIFIER {
            println!("  {} = {}", attribute_name(attr_id), attr_val);
        }
    }
}

/// Dump a rendition for display.
pub fn dump_rendition(rendition: &Rendition) {
    println!(
        "  Rendition: {}x{} @{:.0}x {} layout={} name={}",
        rendition.width,
        rendition.height,
        rendition.scale(),
        rendition.pixel_format_string(),
        rendition.layout,
        rendition.name,
    );
}
