//! Data sets: each file a rendition holding it as it is
//! (docs/clean-room/actool.md).

use crate::car::{self, Facet, Rendition};
use crate::read_json;

/// Compiles data sets, by name, for iOS: one universal file each, with
/// the type identifier its entry gives (Xcode otherwise looks one up for
/// the file's extension).
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
        let Some(uti) = entry["universal-type-identifier"].as_str() else {
            return unsupported("a data set's file without a universal-type-identifier");
        };
        let path = dir.join(entry["filename"].as_str().unwrap());
        let data = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let id = identifier(name);
        let attrs = vec![(car::ATTR_SCALE, 1), (car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE)];
        renditions.push(Rendition { attrs, csi: car::data_csi(uti, &data) });
        facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_IMAGE), (car::ATTR_IDENTIFIER, id)] });
    }
    Ok((facets, renditions))
}
