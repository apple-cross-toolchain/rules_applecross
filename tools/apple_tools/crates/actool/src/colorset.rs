//! Colour sets: each colour a rendition of its appearance
//! (docs/clean-room/actool.md).

use crate::car::{self, Facet, Rendition};
use crate::read_json;

/// The appearances colours are keyed by, and their names in the catalog's
/// table, by value.
pub const APPEARANCES: [&str; 6] =
    ["UIAppearanceAny", "UIAppearanceDark", "UIAppearanceHighContrastAny", "UIAppearanceHighContrastDark", "UIAppearanceLight", "UIAppearanceHighContrastLight"];

/// Colour spaces by name, their code in a colour rendition and their
/// components.
const SPACES: [(&str, u32, &[&str]); 6] = [
    ("srgb", 1, &["red", "green", "blue", "alpha"]),
    ("gray-gamma-22", 2, &["white", "alpha"]),
    ("display-p3", 3, &["red", "green", "blue", "alpha"]),
    ("extended-srgb", 4, &["red", "green", "blue", "alpha"]),
    ("extended-linear-srgb", 5, &["red", "green", "blue", "alpha"]),
    ("extended-gray", 6, &["white", "alpha"]),
];

/// A component as Xcode reads it: with a decimal point, a float (single
/// precision, "0.333" is 0.3330000042915344); `0x` and hex digits over
/// 16^n - 1 ("0x1FF" is 511/4095); otherwise the leading integer over 255
/// ("1" is 1/255, "1e-1" too). Leading spaces are skipped; nothing is
/// clamped.
fn component(text: &str) -> Option<f64> {
    let t = text.trim_start();
    if t.contains('.') {
        return t.parse::<f32>().ok().map(f64::from);
    }
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        let digits: String = hex.chars().take_while(char::is_ascii_hexdigit).collect();
        let n = u64::from_str_radix(&digits, 16).ok()?;
        return Some(n as f64 / (16f64.powi(digits.len() as i32) - 1.0));
    }
    let end = t.char_indices().take_while(|&(i, c)| c.is_ascii_digit() || (i == 0 && c == '-')).count();
    t[..end].parse::<i64>().ok().map(|n| n as f64 / 255.0)
}

/// The value of an entry's appearances: luminosity (light or dark) and
/// contrast (high).
pub fn appearance(entry: &serde_json::Value) -> Option<u16> {
    let mut luminosity = None;
    let mut high = false;
    for a in entry["appearances"].as_array().map(Vec::as_slice).unwrap_or_default() {
        match (a["appearance"].as_str(), a["value"].as_str()) {
            (Some("luminosity"), Some(v @ ("light" | "dark"))) if luminosity.is_none() => luminosity = Some(v),
            (Some("contrast"), Some("high")) if !high => high = true,
            _ => return None,
        }
    }
    Some(match (luminosity, high) {
        (None, false) => 0,
        (Some("dark"), false) => 1,
        (None, true) => 2,
        (Some("dark"), true) => 3,
        (Some(_), false) => 4,
        (Some(_), true) => 5,
    })
}

/// Compiles colour sets, by name, for iOS: universal colours in the colour
/// spaces above, for any appearance. Colour sets with no colours compile to
/// nothing (the scan leaves them out).
pub fn compile(sets: &[(String, std::path::PathBuf)], identifier: impl Fn(&str) -> u16) -> Result<(Vec<Facet>, Vec<Rendition>), String> {
    let mut facets = Vec::new();
    let mut renditions = Vec::new();
    for (name, dir) in sets {
        let unsupported = |what: String| Err(format!("{}: {what} is not supported by this actool", dir.display()));
        let contents = read_json(&dir.join("Contents.json"))?;
        for key in contents.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
            if !["colors", "info"].contains(&key.as_str()) {
                return unsupported(format!("the {key} key of a colour set"));
            }
        }
        let id = identifier(name);
        let mut seen = Vec::new();
        for entry in contents["colors"].as_array().map(Vec::as_slice).unwrap_or_default() {
            let Some(color) = entry["color"].as_object() else { continue };
            for k in entry.as_object().unwrap().keys() {
                if !["idiom", "appearances", "color"].contains(&k.as_str()) {
                    return unsupported(format!("the {k} key of a colour"));
                }
            }
            if entry["idiom"].as_str() != Some("universal") {
                return unsupported("a colour for an idiom other than universal".into());
            }
            let Some(appearance) = appearance(entry) else {
                return unsupported(format!("the appearances {}", entry["appearances"]));
            };
            if seen.contains(&appearance) {
                return unsupported("two colours for one appearance".into());
            }
            seen.push(appearance);
            let mut attrs = vec![(car::ATTR_SCALE, 1), (car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_COLOR)];
            if appearance != 0 {
                attrs.push((car::ATTR_APPEARANCE, appearance));
            }
            // A system colour, as Xcode resolves it for the appearance; one
            // for macOS too, building for iOS (Binder's accent colour).
            if let Some(reference) = color.get("reference") {
                for (k, v) in color {
                    match (k.as_str(), v.as_str()) {
                        ("reference", Some(_)) | ("platform", Some("universal" | "ios" | "osx")) => {}
                        _ => return unsupported(format!("a system colour's {k} {v}")),
                    }
                }
                let reference = reference.as_str().unwrap();
                let Some((_, resolved)) = SYSTEM_COLORS.iter().find(|c| c.0 == reference) else {
                    return unsupported(format!("the system colour {reference}"));
                };
                // Xcode resolves light as dark, and high-contrast light as
                // high-contrast dark (every system colour, light and
                // high-contrast light).
                let (code, components) = resolved[[0, 1, 2, 3, 1, 3][appearance as usize]];
                let components: Vec<f64> = components.iter().map(|c| f64::from(*c)).collect();
                renditions.push(Rendition { attrs, csi: car::color_reference_csi(name, code.into(), &components, reference) });
                continue;
            }
            for k in color.keys() {
                if !["color-space", "components"].contains(&k.as_str()) {
                    return unsupported(format!("a colour's {k} key"));
                }
            }
            // Display P3 when none is named.
            let space = color.get("color-space").and_then(|v| v.as_str()).unwrap_or("display-p3");
            let Some(&(_, code, names)) = SPACES.iter().find(|s| s.0 == space) else {
                return unsupported(format!("the colour space {space:?}"));
            };
            let given = color.get("components").and_then(|v| v.as_object()).cloned().unwrap_or_default();
            if given.len() != names.len() {
                return unsupported(format!("a {space} colour's components {}", serde_json::Value::Object(given)));
            }
            let mut components = Vec::new();
            for n in names {
                match given.get(*n).and_then(|v| v.as_str()).and_then(component) {
                    Some(v) => components.push(v),
                    None => return unsupported(format!("the {n} component {:?}", given.get(*n))),
                }
            }
            renditions.push(Rendition { attrs, csi: car::color_csi(name, code, &components) });
        }
        if !seen.is_empty() {
            facets.push(Facet { name: name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_COLOR), (car::ATTR_IDENTIFIER, id)] });
        }
    }
    Ok((facets, renditions))
}

/// A system colour's space code and components per appearance.
type Resolved = [(u8, &'static [f32]); 4];

/// System colours a colour set may refer to, as Xcode 27.0 resolves them for
/// the any, dark, high-contrast and high-contrast dark appearances: the
/// colour space's code and the components, in single precision (one probe
/// of each; Xcode resolves names it doesn't know to a placeholder).
#[rustfmt::skip]
pub const SYSTEM_COLORS: [(&str, Resolved); 39] = [
    ("darkTextColor", [(6, &[0.0, 1.0]), (6, &[0.0, 1.0]), (6, &[0.0, 1.0]), (6, &[0.0, 1.0])]),
    ("labelColor", [(2, &[0.0, 1.0]), (2, &[1.0, 1.0]), (2, &[0.0, 1.0]), (2, &[1.0, 1.0])]),
    ("lightTextColor", [(6, &[1.0, 0.6]), (6, &[1.0, 0.6]), (6, &[1.0, 0.6]), (6, &[1.0, 0.6])]),
    ("linkColor", [(4, &[0.0, 0.478, 1.0, 1.0]), (4, &[0.035, 0.518, 1.0, 1.0]), (4, &[0.0, 0.478, 1.0, 1.0]), (4, &[0.035, 0.518, 1.0, 1.0])]),
    ("opaqueSeparatorColor", [(4, &[0.776, 0.776, 0.784, 1.0]), (4, &[0.22, 0.22, 0.227, 1.0]), (4, &[0.776, 0.776, 0.784, 1.0]), (4, &[0.22, 0.22, 0.227, 1.0])]),
    ("placeholderTextColor", [(1, &[0.235, 0.235, 0.263, 0.298]), (1, &[0.922, 0.922, 0.961, 0.298]), (1, &[0.235, 0.235, 0.263, 0.698]), (1, &[0.922, 0.922, 0.961, 0.549])]),
    ("quaternaryLabelColor", [(1, &[0.235, 0.235, 0.263, 0.176]), (1, &[0.922, 0.922, 0.961, 0.157]), (1, &[0.235, 0.235, 0.263, 0.549]), (1, &[0.922, 0.922, 0.961, 0.4])]),
    ("quaternarySystemFillColor", [(4, &[0.455, 0.455, 0.502, 0.08]), (4, &[0.463, 0.463, 0.502, 0.18]), (4, &[0.455, 0.455, 0.502, 0.16]), (4, &[0.463, 0.463, 0.502, 0.26])]),
    ("secondaryLabelColor", [(1, &[0.235, 0.235, 0.263, 0.6]), (1, &[0.922, 0.922, 0.961, 0.6]), (1, &[0.235, 0.235, 0.263, 0.8]), (1, &[0.922, 0.922, 0.961, 0.698])]),
    ("secondarySystemBackgroundColor", [(4, &[0.949, 0.949, 0.969, 1.0]), (4, &[0.11, 0.11, 0.118, 1.0]), (4, &[0.922, 0.922, 0.941, 1.0]), (4, &[0.141, 0.141, 0.149, 1.0])]),
    ("secondarySystemFillColor", [(4, &[0.471, 0.471, 0.502, 0.16]), (4, &[0.471, 0.471, 0.502, 0.32]), (4, &[0.471, 0.471, 0.502, 0.24]), (4, &[0.471, 0.471, 0.502, 0.4])]),
    ("secondarySystemGroupedBackgroundColor", [(6, &[1.0, 1.0]), (4, &[0.11, 0.11, 0.118, 1.0]), (6, &[1.0, 1.0]), (4, &[0.141, 0.141, 0.149, 1.0])]),
    ("separatorColor", [(4, &[0.235, 0.235, 0.263, 0.12]), (4, &[0.329, 0.329, 0.345, 0.5]), (4, &[0.235, 0.235, 0.263, 0.12]), (4, &[0.329, 0.329, 0.345, 0.5])]),
    ("systemBackgroundColor", [(6, &[1.0, 1.0]), (6, &[0.0, 1.0]), (6, &[1.0, 1.0]), (6, &[0.0, 1.0])]),
    ("systemBlueColor", [(1, &[0.0, 0.533, 1.0, 1.0]), (1, &[0.0, 0.569, 1.0, 1.0]), (1, &[0.118, 0.431, 0.957, 1.0]), (1, &[0.361, 0.722, 1.0, 1.0])]),
    ("systemBrownColor", [(1, &[0.675, 0.498, 0.369, 1.0]), (1, &[0.718, 0.541, 0.4, 1.0]), (1, &[0.584, 0.427, 0.318, 1.0]), (1, &[0.859, 0.651, 0.475, 1.0])]),
    ("systemCyanColor", [(1, &[0.0, 0.753, 0.91, 1.0]), (1, &[0.235, 0.827, 0.996, 1.0]), (1, &[0.0, 0.494, 0.682, 1.0]), (1, &[0.427, 0.851, 1.0, 1.0])]),
    ("systemFillColor", [(4, &[0.471, 0.471, 0.502, 0.2]), (4, &[0.471, 0.471, 0.502, 0.36]), (4, &[0.471, 0.471, 0.502, 0.28]), (4, &[0.471, 0.471, 0.502, 0.44])]),
    ("systemGray2Color", [(4, &[0.682, 0.682, 0.698, 1.0]), (4, &[0.388, 0.388, 0.4, 1.0]), (4, &[0.557, 0.557, 0.576, 1.0]), (4, &[0.486, 0.486, 0.502, 1.0])]),
    ("systemGray3Color", [(4, &[0.78, 0.78, 0.8, 1.0]), (4, &[0.282, 0.282, 0.29, 1.0]), (4, &[0.682, 0.682, 0.698, 1.0]), (4, &[0.329, 0.329, 0.337, 1.0])]),
    ("systemGray4Color", [(4, &[0.82, 0.82, 0.839, 1.0]), (4, &[0.227, 0.227, 0.235, 1.0]), (4, &[0.737, 0.737, 0.753, 1.0]), (4, &[0.267, 0.267, 0.275, 1.0])]),
    ("systemGray5Color", [(4, &[0.898, 0.898, 0.918, 1.0]), (4, &[0.173, 0.173, 0.18, 1.0]), (4, &[0.847, 0.847, 0.863, 1.0]), (4, &[0.212, 0.212, 0.22, 1.0])]),
    ("systemGray6Color", [(4, &[0.949, 0.949, 0.969, 1.0]), (4, &[0.11, 0.11, 0.118, 1.0]), (4, &[0.922, 0.922, 0.941, 1.0]), (4, &[0.141, 0.141, 0.149, 1.0])]),
    ("systemGrayColor", [(1, &[0.557, 0.557, 0.576, 1.0]), (1, &[0.557, 0.557, 0.576, 1.0]), (1, &[0.424, 0.424, 0.439, 1.0]), (1, &[0.682, 0.682, 0.698, 1.0])]),
    ("systemGreenColor", [(1, &[0.204, 0.78, 0.349, 1.0]), (1, &[0.188, 0.82, 0.345, 1.0]), (1, &[0.0, 0.537, 0.196, 1.0]), (1, &[0.29, 0.851, 0.408, 1.0])]),
    ("systemGroupedBackgroundColor", [(4, &[0.949, 0.949, 0.969, 1.0]), (6, &[0.0, 1.0]), (4, &[0.922, 0.922, 0.941, 1.0]), (6, &[0.0, 1.0])]),
    ("systemIndigoColor", [(1, &[0.38, 0.333, 0.961, 1.0]), (1, &[0.42, 0.365, 1.0, 1.0]), (1, &[0.337, 0.29, 0.871, 1.0]), (1, &[0.655, 0.667, 1.0, 1.0])]),
    ("systemMintColor", [(1, &[0.0, 0.784, 0.702, 1.0]), (1, &[0.0, 0.855, 0.765, 1.0]), (1, &[0.0, 0.522, 0.459, 1.0]), (1, &[0.329, 0.875, 0.796, 1.0])]),
    ("systemOrangeColor", [(1, &[1.0, 0.553, 0.157, 1.0]), (1, &[1.0, 0.573, 0.188, 1.0]), (1, &[0.773, 0.325, 0.0, 1.0]), (1, &[1.0, 0.627, 0.337, 1.0])]),
    ("systemPinkColor", [(1, &[1.0, 0.176, 0.333, 1.0]), (1, &[1.0, 0.216, 0.373, 1.0]), (1, &[0.906, 0.071, 0.302, 1.0]), (1, &[1.0, 0.541, 0.769, 1.0])]),
    ("systemPurpleColor", [(1, &[0.796, 0.188, 0.878, 1.0]), (1, &[0.859, 0.204, 0.949, 1.0]), (1, &[0.69, 0.184, 0.761, 1.0]), (1, &[0.918, 0.553, 1.0, 1.0])]),
    ("systemRedColor", [(1, &[1.0, 0.22, 0.235, 1.0]), (1, &[1.0, 0.259, 0.271, 1.0]), (1, &[0.914, 0.082, 0.176, 1.0]), (1, &[1.0, 0.38, 0.396, 1.0])]),
    ("systemTealColor", [(1, &[0.0, 0.765, 0.816, 1.0]), (1, &[0.0, 0.824, 0.878, 1.0]), (1, &[0.0, 0.506, 0.596, 1.0]), (1, &[0.231, 0.867, 0.925, 1.0])]),
    ("systemYellowColor", [(1, &[1.0, 0.8, 0.0, 1.0]), (1, &[1.0, 0.839, 0.0, 1.0]), (1, &[0.631, 0.416, 0.0, 1.0]), (1, &[0.996, 0.875, 0.263, 1.0])]),
    ("tertiaryLabelColor", [(1, &[0.235, 0.235, 0.263, 0.298]), (1, &[0.922, 0.922, 0.961, 0.298]), (1, &[0.235, 0.235, 0.263, 0.698]), (1, &[0.922, 0.922, 0.961, 0.549])]),
    ("tertiarySystemBackgroundColor", [(6, &[1.0, 1.0]), (4, &[0.173, 0.173, 0.18, 1.0]), (6, &[1.0, 1.0]), (4, &[0.212, 0.212, 0.22, 1.0])]),
    ("tertiarySystemFillColor", [(4, &[0.463, 0.463, 0.502, 0.12]), (4, &[0.463, 0.463, 0.502, 0.24]), (4, &[0.463, 0.463, 0.502, 0.2]), (4, &[0.463, 0.463, 0.502, 0.32])]),
    ("tertiarySystemGroupedBackgroundColor", [(4, &[0.949, 0.949, 0.969, 1.0]), (4, &[0.173, 0.173, 0.18, 1.0]), (4, &[0.922, 0.922, 0.941, 1.0]), (4, &[0.212, 0.212, 0.22, 1.0])]),
    ("tintColor", [(1, &[0.0, 0.533, 1.0, 1.0]), (1, &[0.0, 0.569, 1.0, 1.0]), (1, &[0.118, 0.431, 0.957, 1.0]), (1, &[0.361, 0.722, 1.0, 1.0])]),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Components as Xcode 27.0 stored them.
    #[test]
    fn components() {
        let cases: [(&str, f64); 10] = [
            ("0.123456789012345678", 0.12345679104328156),
            ("0.333", 0.3330000042915344),
            (".5", 0.5),
            (" 0.5", 0.5),
            ("1e-1", 1.0 / 255.0),
            ("0x1FF", 511.0 / 4095.0),
            ("0xff", 1.0),
            ("256", 256.0 / 255.0),
            ("-0.25", -0.25),
            ("1.5", 1.5),
        ];
        for (text, value) in cases {
            assert_eq!(component(text), Some(value), "{text}");
        }
    }
}
