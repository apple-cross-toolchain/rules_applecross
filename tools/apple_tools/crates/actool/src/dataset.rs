//! Data sets: each file a rendition holding it as it is
//! (docs/clean-room/actool.md).

use crate::car::{self, Facet, Rendition};
use crate::read_json;

/// Compiles data sets, by name, for iOS: one universal file each, with
/// the type identifier its entry gives, or else the one Xcode looks up for
/// the file's extension (`file_type`).
pub fn compile(sets: &[(String, std::path::PathBuf)], identifier: impl Fn(&str) -> u16) -> Result<(Vec<Facet>, Vec<Rendition>), String> {
    let mut facets = Vec::new();
    let mut renditions = Vec::new();
    for (name, dir) in sets {
        let unsupported = |what: &str| Err(format!("{}: {what} is not supported by this actool", dir.display()));
        let contents = read_json(&dir.join("Contents.json"))?;
        for key in contents.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
            if !["data", "info"].contains(&key.as_str()) {
                return unsupported(&format!("the {key} key of a data set"));
            }
        }
        let entries: Vec<&serde_json::Value> = contents["data"].as_array().map(|a| a.iter().filter(|e| !e["filename"].is_null()).collect()).unwrap_or_default();
        let [entry] = entries[..] else {
            if entries.is_empty() {
                continue;
            }
            return unsupported("a data set of more than one file");
        };
        for k in entry.as_object().unwrap().keys() {
            if !["filename", "idiom", "universal-type-identifier"].contains(&k.as_str()) {
                return unsupported(&format!("the {k} key of a data set's file"));
            }
        }
        if entry["idiom"].as_str() != Some("universal") {
            return unsupported("a data set's file for an idiom other than universal");
        }
        let filename = entry["filename"].as_str().unwrap();
        let uti = match entry["universal-type-identifier"].as_str() {
            Some(uti) => uti.to_string(),
            None => file_type(filename).ok_or_else(|| format!("{}: a data set's file of an extension whose type depends on the Mac's installed apps is not supported by this actool", dir.display()))?,
        };
        let uti = uti.as_str();
        let path = dir.join(filename);
        let data = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let id = identifier(name);
        let attrs = vec![(car::ATTR_SCALE, 1), (car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE)];
        renditions.push(Rendition { attrs, csi: car::data_csi(uti, &data) });
        facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE), (car::ATTR_IDENTIFIER, id)] });
    }
    Ok((facets, renditions))
}

/// The type identifier Xcode records for a file of a data set that names
/// none, by its last extension, whatever its case (five probes): macOS's
/// and Xcode's declared types (`data_utis`), and for a file without an
/// extension the dynamic identifier for an empty one, `dyn.age8u`. Other
/// extensions are `None`: on a Mac, any app may claim one.
fn file_type(filename: &str) -> Option<String> {
    let Some((_, extension)) = filename.rsplit_once('.') else {
        return Some(dynamic_type(""));
    };
    let extension = extension.to_ascii_lowercase();
    let table = crate::data_utis::DATA_TYPES;
    table.binary_search_by(|(e, _)| (*e).cmp(extension.as_str())).ok().map(|i| table[i].1.to_string())
}

/// A dynamic type identifier for a filename extension no type declares:
/// `dyn.a` and the bytes of `1=<extension>` five bits at a time, most
/// significant first, in the alphabet below (`dyn.age8u` for none,
/// `dyn.age81u8p4` for `xyz`: two probes).
fn dynamic_type(extension: &str) -> String {
    const ALPHABET: &[u8; 32] = b"abcdefghkmnpqrstuvwxyz0123456789";
    let bits: Vec<u8> = format!("1={extension}").bytes().flat_map(|b| (0..8).rev().map(move |i| (b >> i) & 1)).collect();
    let digits = bits.chunks(5).map(|c| ALPHABET[c.iter().chain(std::iter::repeat(&0)).take(5).fold(0, |v, b| v << 1 | *b as usize)] as char);
    format!("dyn.a{}", digits.collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_of_files() {
        assert_eq!(file_type("notes"), Some("dyn.age8u".into()));
        assert_eq!(dynamic_type("xyz"), "dyn.age81u8p4");
        assert_eq!(file_type("a.tar.gz"), Some("org.gnu.gnu-zip-archive".into()));
        assert_eq!(file_type("f.PNG"), Some("public.png".into()));
        assert_eq!(file_type("color.drawing"), Some("com.apple.drawing".into()));
        assert_eq!(file_type("f.dat"), None);
        let table = crate::data_utis::DATA_TYPES;
        assert!(table.windows(2).all(|w| w[0].0 < w[1].0), "DATA_TYPES is sorted");
    }
}
