//! Icon Composer documents (docs/clean-room/actool.md#icon-composer-documents):
//! the layered renditions Xcode compiles a document to (its images as
//! Vector renditions, a stack and a group per appearance, the colours and
//! gradients they name), byte for byte, and this actool's own flat
//! renderings for the rasters Xcode writes beside them.

use crate::car::{self, Facet, Rendition};
use crate::image::Rgba;
use crate::{dmp2, name_identifier};
use serde_json::{Map, Value};
use std::path::Path;

/// The canvas of an icon, in pixels (and points: Icon Composer's documents
/// are 1024 pt square at 1x).
pub const CANVAS: u32 = 1024;

pub const PART_ICON_STACK: u16 = 245;
pub const PART_ICON_GROUP: u16 = 246;
pub const PART_GRADIENT: u16 = 247;
const LAYOUT_VECTOR: u16 = 9;
const LAYOUT_ICON_STACK: u16 = 1019;
const LAYOUT_ICON_GROUP: u16 = 1020;
const LAYOUT_GRADIENT: u16 = 1021;

fn fail<T>(msg: impl Into<String>) -> Result<T, String> {
    Err(msg.into())
}

/// The appearances a document is compiled for: the stack and groups of
/// each, keyed by its value (light is UIAppearanceLight, 4, for iOS and
/// NSAppearanceNameAqua, 8, for macOS).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Appearance {
    Light,
    Dark,
    Tinted,
}

impl Appearance {
    pub const ALL: [Appearance; 3] = [Appearance::Light, Appearance::Dark, Appearance::Tinted];

    fn name(self) -> &'static str {
        match self {
            Appearance::Light => "light",
            Appearance::Dark => "dark",
            Appearance::Tinted => "tinted",
        }
    }

    pub fn value(self, mac: bool) -> u16 {
        match self {
            Appearance::Light if mac => 8,
            Appearance::Light => 4,
            Appearance::Dark => 1,
            Appearance::Tinted => 10,
        }
    }
}

// --- Colours ---------------------------------------------------------------------

/// A colour as icon.json writes it, `<space>:<components>`: the space's code
/// in a colour rendition and the components.
#[derive(Clone, PartialEq, Debug)]
pub struct Color {
    pub space: u32,
    /// Stored as single-precision floats, widened to doubles.
    pub components: Vec<f64>,
}

/// Colour spaces by name: their code, and whether they are grey.
const SPACES: [(&str, u32, bool); 5] = [("srgb", 1, false), ("gray", 2, true), ("display-p3", 3, false), ("extended-srgb", 4, false), ("extended-gray", 6, true)];

fn color(spec: &str) -> Result<Color, String> {
    let bad = || format!("the colour {spec:?} is not supported by this actool");
    let (space, rest) = spec.split_once(':').ok_or_else(bad)?;
    let &(_, code, gray) = SPACES.iter().find(|s| s.0 == space).ok_or_else(bad)?;
    let components: Vec<f64> = rest.split(',').map(|v| v.parse::<f64>().map_err(|_| bad()).map(thousandths)).collect::<Result<_, _>>()?;
    if components.len() != if gray { 2 } else { 4 } || components.iter().any(|c| !c.is_finite()) {
        return Err(bad());
    }
    Ok(Color { space: code, components })
}

/// A component as Xcode keeps it: to three decimal places, as `%.3f` rounds
/// (an exact half to even: 0.0625 is 0.062, 0.0025 a little over half,
/// 0.003; four probes).
fn thousandths(v: f64) -> f64 {
    format!("{v:.3}").parse().unwrap()
}

impl Color {
    fn gray(white: f64, alpha: f64, space: u32) -> Color {
        Color { space, components: vec![white, alpha] }
    }

    pub fn alpha(&self) -> f64 {
        *self.components.last().unwrap()
    }

    /// Its components as a colour rendition stores them.
    fn stored(&self) -> Vec<f64> {
        self.components.iter().map(|&v| v as f32 as f64).collect()
    }

    fn with_alpha(&self, alpha: f64) -> Color {
        let mut c = self.clone();
        *c.components.last_mut().unwrap() = alpha;
        c
    }

    /// The colour in Display P3 and in sRGB (gamma-encoded, unclamped).
    fn rgb(&self) -> ([f64; 3], [f64; 3]) {
        let c = &self.components;
        match self.space {
            3 => {
                let p3 = [c[0], c[1], c[2]];
                (p3, convert(&P3_TO_SRGB, &p3))
            }
            // Gray Gamma 2.2: linear light, then sRGB's curve.
            2 => {
                let s = encode(c[0].max(0.0).powf(2.2));
                ([s; 3], [s; 3])
            }
            6 => ([c[0]; 3], [c[0]; 3]),
            _ => {
                let s = [c[0], c[1], c[2]];
                (convert(&SRGB_TO_P3, &s), s)
            }
        }
    }

    /// HSL lightness in Display P3: what Xcode tells a light colour from a
    /// dark one by (`LIGHT`).
    fn lightness(&self) -> f64 {
        let (p3, _) = self.rgb();
        let max = p3.iter().cloned().fold(f64::MIN, f64::max);
        let min = p3.iter().cloned().fold(f64::MAX, f64::min);
        (max + min) / 2.0
    }

    /// Straight sRGB components for drawing, clamped.
    fn srgb8(&self) -> [f32; 4] {
        let (_, s) = self.rgb();
        [s[0] as f32, s[1] as f32, s[2] as f32, self.alpha() as f32].map(|v| v.clamp(0.0, 1.0))
    }
}

fn decode(c: f64) -> f64 {
    let a = c.abs();
    let v = if a <= 0.04045 { a / 12.92 } else { ((a + 0.055) / 1.055).powf(2.4) };
    v.copysign(c)
}

fn encode(c: f64) -> f64 {
    let a = c.abs();
    let v = if a <= 0.0031308 { a * 12.92 } else { 1.055 * a.powf(1.0 / 2.4) - 0.055 };
    v.copysign(c)
}

/// sRGB to Display P3 (both D65, sRGB's curve), through linear light.
const SRGB_TO_P3: [[f64; 3]; 3] = [[0.822462, 0.177538, 0.0], [0.033194, 0.966806, 0.0], [0.017083, 0.072397, 0.910520]];
const P3_TO_SRGB: [[f64; 3]; 3] = [[1.224940, -0.224940, 0.0], [-0.042057, 1.042057, 0.0], [-0.019638, -0.078636, 1.098274]];

fn convert(m: &[[f64; 3]; 3], c: &[f64; 3]) -> [f64; 3] {
    let l = c.map(decode);
    [0, 1, 2].map(|i| encode(m[i][0] * l[0] + m[i][1] * l[1] + m[i][2] * l[2]))
}

/// Colours at least this light (HSL lightness in Display P3) are light:
/// D9D9D9 is, D8D8D8 isn't; FFC0C0 and B2FFB2 are, FFB3B3 and FFFF00
/// aren't (85 probes, 40 of them within 0.0015 of it).
const LIGHT: f64 = 0.85;
/// Backgrounds at least this light (the same lightness, averaged over a
/// gradient's colours) are light enough for a light layer to take their
/// colour in dark (28 probes).
const LIGHT_BACKGROUND: f64 = 0.2;
/// How close to a threshold a colour converted to Display P3 may come
/// before this actool refuses to decide: its conversion may round apart
/// from ColorSync's (the 40 probes near `LIGHT` all fell on the side this
/// conversion puts them, the nearest 0.00009 away).
const LIGHT_MARGIN: f64 = 0.0001;

/// Whether a lightness is at least `threshold`, or None when too close to
/// call for a colour that isn't Display P3's own.
fn at_least(c: &Color, l: f64, threshold: f64) -> Option<bool> {
    (c.space == 3 || (l - threshold).abs() >= LIGHT_MARGIN).then_some(l >= threshold)
}

/// Whether every one of `colors` is light; an error when that depends on
/// one too close to call.
fn all_light(colors: &[Color]) -> Result<bool, String> {
    let calls: Vec<Option<bool>> = colors.iter().map(|c| at_least(c, c.lightness(), LIGHT)).collect();
    if calls.contains(&Some(false)) {
        return Ok(false);
    }
    if calls.contains(&None) {
        return fail("a colour too close to Xcode's threshold of light to tell, which this actool doesn't");
    }
    Ok(true)
}

/// A gradient's orientation: start and stop in unit coordinates.
#[derive(Clone, PartialEq, Debug)]
struct Orientation([f32; 4]);

impl Default for Orientation {
    fn default() -> Self {
        Orientation([0.5, 0.0, 0.5, 1.0])
    }
}

/// A fill as icon.json writes it.
#[derive(Clone, PartialEq, Debug)]
enum Fill {
    Solid(Color),
    /// A gradient Xcode derives from one colour (kind 0).
    Automatic(Color, Orientation),
    /// Two colours (kind 1).
    Linear(Vec<Color>, Orientation),
    SystemLight,
    SystemDark,
}

impl Fill {
    fn stops(&self) -> Vec<Color> {
        match self {
            Fill::Solid(c) | Fill::Automatic(c, _) => vec![c.clone()],
            Fill::Linear(cs, _) => cs.clone(),
            Fill::SystemLight => system_light(),
            Fill::SystemDark => system_dark(),
        }
    }

    fn is_light(&self) -> Result<bool, String> {
        match self {
            Fill::SystemLight => Ok(true),
            Fill::SystemDark => Ok(false),
            _ => all_light(&self.stops()),
        }
    }

    /// Whether a background is light enough for light layers to take its
    /// colour in dark: its colours' lightness, averaged.
    fn is_light_background(&self) -> Result<bool, String> {
        let stops = self.stops();
        let l = stops.iter().map(Color::lightness).sum::<f64>() / stops.len() as f64;
        let exact = stops.iter().all(|c| c.space == 3) || matches!(self, Fill::SystemLight | Fill::SystemDark);
        if !exact && (l - LIGHT_BACKGROUND).abs() < LIGHT_MARGIN {
            return fail("a background too close to Xcode's threshold of light to tell, which this actool doesn't");
        }
        Ok(l >= LIGHT_BACKGROUND)
    }
}

fn system_light() -> Vec<Color> {
    vec![Color::gray(1.0, 1.0, 2), Color::gray(0.925, 1.0, 2)]
}

fn system_dark() -> Vec<Color> {
    vec![Color::gray(0.192, 1.0, 2), Color::gray(0.078, 1.0, 2)]
}

fn orientation(v: &Map<String, Value>) -> Result<Orientation, String> {
    let Some(o) = v.get("orientation") else { return Ok(Orientation::default()) };
    let point = |k: &str| -> Result<[f32; 2], String> {
        let p = o.get(k).and_then(Value::as_object).ok_or("a gradient's orientation needs a start and a stop")?;
        if p.keys().any(|k| k != "x" && k != "y") {
            return fail("a gradient orientation with other keys is not supported by this actool");
        }
        let n = |k: &str| p.get(k).and_then(Value::as_f64).map(|v| v as f32).ok_or("a gradient's orientation needs x and y");
        Ok([n("x")?, n("y")?])
    };
    if o.as_object().is_none_or(|o| o.keys().any(|k| k != "start" && k != "stop")) {
        return fail("a gradient orientation with other keys is not supported by this actool");
    }
    let (s, e) = (point("start")?, point("stop")?);
    Ok(Orientation([s[0], s[1], e[0], e[1]]))
}

/// A fill value; `None` for `automatic` (resolved where it is used).
fn fill(v: &Value) -> Result<Option<Fill>, String> {
    match v {
        Value::String(s) => match s.as_str() {
            "automatic" => Ok(None),
            "system-light" => Ok(Some(Fill::SystemLight)),
            "system-dark" => Ok(Some(Fill::SystemDark)),
            other => fail(format!("the fill {other:?} is not supported by this actool")),
        },
        Value::Object(o) => {
            let only = |keys: &[&str]| o.keys().all(|k| keys.contains(&k.as_str()));
            if let Some(s) = o.get("solid").and_then(Value::as_str) {
                if !only(&["solid"]) {
                    return fail("a solid fill with other keys is not supported by this actool");
                }
                Ok(Some(Fill::Solid(color(s)?)))
            } else if let Some(s) = o.get("automatic-gradient").and_then(Value::as_str) {
                if !only(&["automatic-gradient", "orientation"]) {
                    return fail("an automatic gradient with other keys is not supported by this actool");
                }
                Ok(Some(Fill::Automatic(color(s)?, orientation(o)?)))
            } else if let Some(list) = o.get("linear-gradient").and_then(Value::as_array) {
                if !only(&["linear-gradient", "orientation"]) {
                    return fail("a linear gradient with other keys is not supported by this actool");
                }
                let colors: Vec<Color> = list.iter().map(|c| c.as_str().ok_or("a gradient colour should be a string").map_err(String::from).and_then(color)).collect::<Result<_, _>>()?;
                if colors.len() != 2 {
                    return fail("linear gradients of other than two colours are not supported by this actool");
                }
                Ok(Some(Fill::Linear(colors, orientation(o)?)))
            } else {
                fail(format!("the fill {v} is not supported by this actool"))
            }
        }
        _ => fail(format!("the fill {v} is not supported by this actool")),
    }
}

// --- Specializations -----------------------------------------------------------

/// The platform a document is compiled for: the idioms its specializations
/// may name, least specific first.
#[derive(Clone, Copy)]
pub struct Platform {
    pub mac: bool,
}

impl Platform {
    /// How specific an idiom is for this platform: `square` (iOS and
    /// macOS) under the platform's own name; None for another platform's.
    fn idiom_rank(self, idiom: &str) -> Result<Option<u8>, String> {
        match idiom {
            "square" => Ok(Some(1)),
            "iOS" if !self.mac => Ok(Some(2)),
            "macOS" if self.mac => Ok(Some(2)),
            "iOS" | "macOS" | "watchOS" => Ok(None),
            other => fail(format!("the idiom {other:?} is not supported by this actool")),
        }
    }
}

/// What a property resolves to for an appearance.
struct Resolved<'a> {
    value: Option<&'a Value>,
    /// Whether that came from a specialization for that appearance.
    for_appearance: bool,
}

/// A property's value for an appearance: the plain key, or else the
/// specialization most specific to the platform's idiom (its own name over
/// `square` over none) and then to the appearance, the last of equals
/// (probes of each).
fn resolve<'a>(o: &'a Map<String, Value>, key: &str, appearance: Appearance, platform: Platform) -> Result<Resolved<'a>, String> {
    let specs_key = format!("{key}-specializations");
    let plain = o.get(key);
    let Some(specs) = o.get(&specs_key) else { return Ok(Resolved { value: plain, for_appearance: false }) };
    if plain.is_some() {
        // Xcode takes the plain key and ignores the specializations (one
        // probe, of opacity); not established for the others.
        return fail(format!("both {key} and {specs_key} are not supported by this actool"));
    }
    let specs = specs.as_array().ok_or_else(|| format!("{specs_key} should be a list"))?;
    let mut best: Option<((u8, bool), &Value)> = None;
    for s in specs {
        let s = s.as_object().ok_or_else(|| format!("{specs_key} should be a list of objects"))?;
        if s.keys().any(|k| !["appearance", "idiom", "value"].contains(&k.as_str())) {
            return fail(format!("a specialization with other keys than appearance, idiom and value is not supported by this actool ({specs_key})"));
        }
        let value = s.get("value").ok_or_else(|| format!("{specs_key}: a specialization without a value"))?;
        let idiom = match s.get("idiom") {
            None => Some(0),
            Some(i) => platform.idiom_rank(i.as_str().ok_or("an idiom should be a string")?)?,
        };
        let matches_appearance = match s.get("appearance") {
            None => Some(false),
            Some(a) => {
                let a = a.as_str().ok_or("an appearance should be a string")?;
                if !["light", "dark", "tinted"].contains(&a) {
                    return fail(format!("the appearance {a:?} is not supported by this actool"));
                }
                (a == appearance.name()).then_some(true)
            }
        };
        let (Some(idiom), Some(specific)) = (idiom, matches_appearance) else { continue };
        if best.is_none_or(|(rank, _)| (idiom, specific) >= rank) {
            best = Some(((idiom, specific), value));
        }
    }
    Ok(Resolved { value: best.map(|b| b.1), for_appearance: best.is_some_and(|b| b.0 .1) })
}

fn as_f32(v: &Value, what: &str) -> Result<f32, String> {
    v.as_f64().map(|v| v as f32).ok_or_else(|| format!("{what} should be a number"))
}

fn as_bool(v: &Value, what: &str) -> Result<bool, String> {
    v.as_bool().ok_or_else(|| format!("{what} should be true or false"))
}

/// A placement: a scale about the canvas centre, then a translation.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Position {
    scale: f64,
    tx: f64,
    ty: f64,
}

impl Default for Position {
    fn default() -> Self {
        Position { scale: 1.0, tx: 0.0, ty: 0.0 }
    }
}

fn position(v: Option<&Value>) -> Result<Position, String> {
    let Some(v) = v else { return Ok(Position::default()) };
    let o = v.as_object().ok_or("a position should be an object")?;
    if o.keys().any(|k| k != "scale" && k != "translation-in-points") {
        return fail("a position with other keys is not supported by this actool");
    }
    let scale = o.get("scale").map(|s| s.as_f64().ok_or("a position's scale should be a number")).transpose()?.unwrap_or(1.0);
    let (tx, ty) = match o.get("translation-in-points").and_then(Value::as_array).map(Vec::as_slice) {
        None => (0.0, 0.0),
        Some([x, y]) => (x.as_f64().ok_or("a translation should be numbers")?, y.as_f64().ok_or("a translation should be numbers")?),
        Some(_) => return fail("a position's translation-in-points should be two numbers"),
    };
    Ok(Position { scale, tx, ty })
}

/// Blend modes by name: CGBlendMode's values, the ones Xcode accepts.
const BLEND_MODES: [(&str, u32); 10] = [
    ("normal", 0),
    ("multiply", 1),
    ("screen", 2),
    ("overlay", 3),
    ("darken", 4),
    ("lighten", 5),
    ("soft-light", 8),
    ("hard-light", 9),
    ("plus-darker", 26),
    ("plus-lighter", 27),
];

fn blend_mode(o: &Map<String, Value>) -> Result<u32, String> {
    if o.contains_key("blend-mode-specializations") {
        // Xcode ignores them (one probe of a layer's).
        return fail("blend-mode-specializations are not supported by this actool");
    }
    match o.get("blend-mode") {
        None => Ok(0),
        Some(v) => {
            let name = v.as_str().ok_or("a blend mode should be a string")?;
            BLEND_MODES.iter().find(|m| m.0 == name).map(|m| m.1).ok_or_else(|| format!("the blend mode {name:?} is not supported by this actool"))
        }
    }
}

/// Keys of a document, a group and a layer, and the ones Xcode was seen to
/// ignore (those are refused anywhere else).
const DOCUMENT_KEYS: [&str; 4] = ["fill", "fill-specializations", "groups", "supported-platforms"];
const GROUP_KEYS: [&str; 22] = [
    "layers",
    "name",
    "hidden",
    "hidden-specializations",
    "opacity",
    "opacity-specializations",
    "blend-mode",
    "lighting",
    "lighting-specializations",
    "blur-material",
    "blur-material-specializations",
    "specular",
    "specular-specializations",
    "translucency",
    "translucency-specializations",
    "shadow",
    "shadow-specializations",
    "position",
    "position-specializations",
    // Ignored by Xcode (one probe each).
    "fill",
    "fill-specializations",
    "supported-platforms",
];
const LAYER_KEYS: [&str; 19] = [
    "image-name",
    "image-name-specializations",
    "name",
    "hidden",
    "hidden-specializations",
    "glass",
    "glass-specializations",
    "opacity",
    "opacity-specializations",
    "blend-mode",
    "fill",
    "fill-specializations",
    "position",
    "position-specializations",
    // Ignored by Xcode (one probe each).
    "shadow",
    "translucency",
    "specular",
    "blur-material",
    "supported-platforms",
];

fn check_keys(o: &Map<String, Value>, allowed: &[&str], what: &str) -> Result<(), String> {
    match o.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(k) => fail(format!("{what}'s {k:?} is not supported by this actool")),
        None => Ok(()),
    }
}

// --- The document ----------------------------------------------------------------

/// A layer as an appearance shows it.
#[derive(Clone, Debug)]
struct LayerLook {
    hidden: bool,
    opacity: f32,
    glass: bool,
    fill: Option<Fill>,
    /// Whether that fill was given for this appearance.
    fill_for_appearance: bool,
    /// Whether its opacity was given for this appearance, whatever the value.
    opacity_for_appearance: bool,
    position: Position,
}

struct Layer {
    image: String,
    blend: u32,
    looks: [LayerLook; 3],
}

#[derive(Clone, Debug)]
struct GroupLook {
    hidden: bool,
    opacity: f32,
    lighting: u32,
    blur: f32,
    specular: bool,
    translucency: f32,
    shadow_kind: u32,
    shadow_opacity: f32,
    position: Position,
}

struct Group {
    name: String,
    blend: u32,
    looks: [GroupLook; 3],
    layers: Vec<Layer>,
}

/// An image a layer shows: an SVG (a Vector rendition) or a PNG.
struct Image {
    /// Its name in the document's Assets folder.
    file: String,
    /// Its facet, "<name>_Assets/<file without extension>".
    facet: String,
    width: f64,
    height: f64,
    data: Vec<u8>,
    svg: bool,
    /// Whether every colour it paints is light (SVGs only).
    light: bool,
}

pub struct Document {
    name: String,
    platform: Platform,
    /// The background's fill for the light (and tinted) appearance.
    fill: Fill,
    /// For the dark appearance, and whether Xcode chose it (system-dark,
    /// unless the document names one).
    dark_fill: Fill,
    dark_automatic: bool,
    groups: Vec<Group>,
    images: Vec<Image>,
}

fn per_appearance<T>(f: impl Fn(Appearance) -> Result<T, String>) -> Result<[T; 3], String> {
    Ok([f(Appearance::Light)?, f(Appearance::Dark)?, f(Appearance::Tinted)?])
}

/// A group's name as its facet has it: letters, digits, spaces, `.` and
/// `-` kept, every other character `_` (one per character).
fn sanitized(name: &str) -> Result<String, String> {
    name.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | ' ' | '.' | '-' => Ok(c),
            c if (c as u32) < 0x10000 => Ok('_'),
            _ => fail(format!("the name {name:?}: characters outside the Basic Multilingual Plane are not supported by this actool")),
        })
        .collect()
}

pub fn read(doc: &Path, name: &str, platform: Platform) -> Result<Document, String> {
    let json_path = doc.join("icon.json");
    let bytes = std::fs::read(&json_path).map_err(|e| format!("{}: {e}", json_path.display()))?;
    let json: Value = serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", json_path.display()))?;
    let at = |e: String| format!("{}: {e}", json_path.display());
    read_json(doc, name, platform, &json).map_err(at)
}

fn read_json(doc: &Path, name: &str, platform: Platform, json: &Value) -> Result<Document, String> {
    if name.is_empty() || name.chars().any(|c| !(c.is_ascii_alphanumeric() || "_-.".contains(c))) {
        return fail(format!("the icon name {name:?} is not supported by this actool"));
    }
    let root = json.as_object().ok_or("icon.json should be an object")?;
    check_keys(root, &DOCUMENT_KEYS, "the document")?;

    // The background: for light (tinted takes light's: Xcode ignores a
    // tinted specialization), and for dark, where Xcode uses system-dark
    // unless the document names a fill for dark, or system-dark for all.
    let light = resolve(root, "fill", Appearance::Light, platform)?;
    let fill_value = match light.value {
        None => Some(Fill::Solid(Color::gray(1.0, 1.0, 6))),
        Some(v) => self::fill(v)?,
    };
    let fill_light = fill_value.unwrap_or(Fill::SystemLight);
    let dark = resolve(root, "fill", Appearance::Dark, platform)?;
    // Only a fill given for dark is the document's choice: system-dark as
    // the default value, with light given its own, is still Xcode's (one of
    // the holdout's documents).
    let (dark_fill, dark_automatic) = match (dark.for_appearance, dark.value.map(self::fill).transpose()?.flatten()) {
        (true, Some(f)) => (f, false),
        _ => (Fill::SystemDark, true),
    };

    let mut groups = Vec::new();
    let mut images: Vec<Image> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for g in root.get("groups").and_then(Value::as_array).ok_or("a document needs groups")? {
        let g = g.as_object().ok_or("a group should be an object")?;
        check_keys(g, &GROUP_KEYS, "a group")?;
        let looks = per_appearance(|a| group_look(g, a, platform))?;
        match (looks[0].hidden, looks[1].hidden, looks[2].hidden) {
            (true, true, true) => continue,
            (false, false, false) => {}
            _ => return fail("a group hidden in some appearances only is not supported by this actool"),
        }
        let mut layers = Vec::new();
        for l in g.get("layers").and_then(Value::as_array).ok_or("a group needs layers")? {
            let l = l.as_object().ok_or("a layer should be an object")?;
            check_keys(l, &LAYER_KEYS, "a layer")?;
            let looks = per_appearance(|a| layer_look(l, a, platform))?;
            // A layer that adds nothing in any appearance, hidden or fully
            // transparent without glass, is left out, image and all; fully
            // transparent with glass, it is kept for its glass (28 probes:
            // hidden and opacity by appearance and idiom, glass on and off).
            if looks.iter().all(|l| l.hidden || (l.opacity == 0.0 && !l.glass)) {
                continue;
            }
            let image_names = per_appearance(|a| {
                let r = resolve(l, "image-name", a, platform)?;
                r.value.and_then(Value::as_str).map(str::to_string).ok_or_else(|| "a layer without an image is not supported by this actool".to_string())
            })?;
            if image_names[1] != image_names[0] || image_names[2] != image_names[0] {
                return fail("a layer showing different images by appearance is not supported by this actool");
            }
            let file = image_names[0].clone();
            if !images.iter().any(|i| i.file == file) {
                images.push(image(doc, name, &file)?);
            }
            layers.push(Layer { image: file, blend: blend_mode(l)?, looks });
        }
        // A group left with no layers is left out too (one probe).
        if layers.is_empty() {
            continue;
        }
        if looks.iter().all(|l| l.opacity == 0.0) {
            return fail("a fully transparent group is not supported by this actool");
        }
        // Names, made unique in order: "Group", then "Group 2" and so on
        // (a name taken goes to the first; four probes).
        let base = match g.get("name") {
            None => "Group".to_string(),
            Some(n) => {
                let n = n.as_str().ok_or("a group's name should be a string")?;
                if n.is_empty() {
                    return fail("a group with an empty name is not supported by this actool");
                }
                sanitized(n)?
            }
        };
        let mut unique = base.clone();
        let mut k = 2;
        while names.contains(&unique) {
            unique = format!("{base} {k}");
            k += 1;
        }
        names.push(unique.clone());

        groups.push(Group { name: unique, blend: blend_mode(g)?, looks, layers });
    }
    for (i, a) in images.iter().enumerate() {
        if images[..i].iter().any(|b| b.facet == a.facet) {
            return fail(format!("two images named {} are not supported by this actool", a.facet));
        }
    }
    Ok(Document { name: name.to_string(), platform, fill: fill_light, dark_fill, dark_automatic, groups, images })
}

fn group_look(g: &Map<String, Value>, a: Appearance, p: Platform) -> Result<GroupLook, String> {
    let r = |k: &str| resolve(g, k, a, p).map(|r| r.value);
    let hidden = r("hidden")?.map(|v| as_bool(v, "hidden")).transpose()?.unwrap_or(false);
    let opacity = r("opacity")?.map(|v| as_f32(v, "opacity")).transpose()?.unwrap_or(1.0);
    let lighting = match r("lighting")?.map(|v| v.as_str().ok_or("lighting should be a string")).transpose()? {
        None | Some("individual") => 2,
        Some("combined") => 0,
        Some(other) => return fail(format!("the lighting {other:?} is not supported by this actool")),
    };
    let blur = match r("blur-material")? {
        None | Some(Value::Null) => 0.0,
        Some(v) => as_f32(v, "blur-material")?,
    };
    let specular = r("specular")?.map(|v| as_bool(v, "specular")).transpose()?.unwrap_or(true);
    // Absent, translucency is 0.3 (one probe); disabled, 0.
    let translucency = match r("translucency")? {
        None => 0.3,
        Some(v) => {
            let o = v.as_object().ok_or("translucency should be an object")?;
            if o.keys().any(|k| k != "enabled" && k != "value") {
                return fail("translucency with other keys is not supported by this actool");
            }
            let enabled = as_bool(o.get("enabled").ok_or("translucency needs enabled")?, "translucency's enabled")?;
            let value = as_f32(o.get("value").ok_or("translucency needs a value")?, "translucency's value")?;
            if enabled { value } else { 0.0 }
        }
    };
    let (shadow_kind, shadow_opacity) = match r("shadow")? {
        None => (3, 0.5),
        Some(v) => {
            let o = v.as_object().ok_or("a shadow should be an object")?;
            if o.keys().any(|k| k != "kind" && k != "opacity") {
                return fail("a shadow with other keys is not supported by this actool");
            }
            let kind = match o.get("kind").and_then(Value::as_str).ok_or("a shadow needs a kind")? {
                "none" => 0,
                "layer-color" => 2,
                "neutral" => 3,
                other => return fail(format!("the shadow {other:?} is not supported by this actool")),
            };
            (kind, as_f32(o.get("opacity").ok_or("a shadow needs an opacity")?, "a shadow's opacity")?)
        }
    };
    Ok(GroupLook { hidden, opacity, lighting, blur, specular, translucency, shadow_kind, shadow_opacity, position: position(r("position")?)? })
}

fn light_fill(l: &Map<String, Value>, p: Platform) -> Result<Option<Fill>, String> {
    resolve(l, "fill", Appearance::Light, p)?.value.map(self::fill).transpose().map(Option::flatten)
}

fn layer_look(l: &Map<String, Value>, a: Appearance, p: Platform) -> Result<LayerLook, String> {
    let r = |k: &str| resolve(l, k, a, p).map(|r| r.value);
    // "automatic" is no fill for light, and light's fill for the others
    // (three probes and two of the holdout's documents).
    let resolved = resolve(l, "fill", a, p)?;
    let (fill, fill_for_appearance) = match resolved.value {
        Some(Value::String(v)) if v == "automatic" && a != Appearance::Light => (light_fill(l, p)?, false),
        Some(v) => (self::fill(v)?, resolved.for_appearance),
        None => (None, false),
    };
    let opacity = resolve(l, "opacity", a, p)?;
    Ok(LayerLook {
        hidden: r("hidden")?.map(|v| as_bool(v, "hidden")).transpose()?.unwrap_or(false),
        opacity: opacity.value.map(|v| as_f32(v, "opacity")).transpose()?.unwrap_or(1.0),
        opacity_for_appearance: opacity.for_appearance,
        glass: r("glass")?.map(|v| as_bool(v, "glass")).transpose()?.unwrap_or(true),
        fill,
        fill_for_appearance,
        position: position(r("position")?)?,
    })
}

/// The image a layer names, from the document's Assets folder.
fn image(doc: &Path, name: &str, file: &str) -> Result<Image, String> {
    if file.contains('/') || file.starts_with('.') {
        return fail(format!("the image {file:?} is outside the document's Assets"));
    }
    let path = doc.join("Assets").join(file);
    let data = std::fs::read(&path).map_err(|_| format!("The layer image {file:?} does not exist."))?;
    let (stem, ext) = file.rsplit_once('.').unwrap_or((file, ""));
    let facet = format!("{name}_Assets/{}", sanitized(stem)?);
    match ext.to_ascii_lowercase().as_str() {
        "svg" => {
            let (width, height) = svg_size(&data).map_err(|e| format!("{file}: {e}"))?;
            let light = svg_is_light(&data).map_err(|e| format!("{file}: {e}"))?;
            Ok(Image { file: file.to_string(), facet, width, height, data, svg: true, light })
        }
        "png" => {
            if data.get(24) != Some(&8) {
                // Xcode keeps a 16-bit layer's wider colours too, beside it.
                return fail(format!("{file}: PNG layers of other than 8 bits a channel are not supported by this actool"));
            }
            crate::image::check_color_management(&data).map_err(|e| format!("{file}: {e}"))?;
            // A layer's size is its pixels', whatever resolution its PNG
            // gives (rules_apple's 90 dpi logo; 144 dpi, one probe).
            let img = crate::image::decode_png(&data)?;
            if img.width * img.height <= crate::stack::PACKING_AREA {
                return fail(format!("{file}: PNG layers small enough to pack are not supported by this actool"));
            }
            Ok(Image { file: file.to_string(), facet, width: img.width as f64, height: img.height as f64, data, svg: false, light: false })
        }
        _ => fail(format!("{file}: only SVG and PNG layers are supported by this actool")),
    }
}

// --- SVGs ---------------------------------------------------------------------------

fn parse_svg(data: &[u8]) -> Result<roxmltree::Document<'_>, String> {
    let text = std::str::from_utf8(data).map_err(|_| "an SVG that isn't UTF-8 is not supported by this actool".to_string())?;
    roxmltree::Document::parse_with_options(text, roxmltree::ParsingOptions { allow_dtd: false, ..Default::default() }).map_err(|e| format!("{e}"))
}

/// A length as Xcode reads an SVG's size: the number, whatever its unit
/// (`300pt` is 300).
fn svg_length(v: &str) -> Result<f64, String> {
    let v = v.trim();
    let end = v.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E')).unwrap_or(v.len());
    let unit = &v[end..];
    if !["", "px", "pt"].contains(&unit) {
        return fail(format!("an SVG size in {unit:?} is not supported by this actool"));
    }
    v[..end].parse::<f64>().map_err(|_| format!("the SVG size {v:?} is not supported by this actool"))
}

/// An SVG's size: its width and height, or its view box's.
fn svg_size(data: &[u8]) -> Result<(f64, f64), String> {
    let doc = parse_svg(data)?;
    let root = doc.root_element();
    if root.tag_name().name() != "svg" {
        return fail("not an SVG");
    }
    match (root.attribute("width"), root.attribute("height"), root.attribute("viewBox")) {
        (Some(w), Some(h), _) => Ok((svg_length(w)?, svg_length(h)?)),
        (None, None, Some(vb)) => {
            let n: Vec<f64> = vb.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).map(|s| s.parse::<f64>().map_err(|_| "a view box should be numbers")).collect::<Result<_, _>>()?;
            match n[..] {
                [_, _, w, h] => Ok((w, h)),
                _ => fail("a view box should be four numbers"),
            }
        }
        _ => fail("an SVG without both a width and a height, or a view box, is not supported by this actool"),
    }
}

/// A paint value's colour: None for `none` (and `transparent`, which Xcode
/// takes as none), or a colour.
enum Paint {
    None,
    Color(Color),
    Url(String),
}

fn paint(v: &str) -> Result<Paint, String> {
    let v = v.trim();
    if v == "none" || v == "transparent" {
        return Ok(Paint::None);
    }
    if let Some(rest) = v.strip_prefix("url(") {
        let (target, after) = rest.split_once(')').ok_or("a malformed url()")?;
        if !after.trim().is_empty() {
            return fail("a paint with a fallback is not supported by this actool");
        }
        let id = target.trim().trim_matches(|c| c == '"' || c == '\'').strip_prefix('#').ok_or("a url() to another file is not supported by this actool")?;
        return Ok(Paint::Url(id.to_string()));
    }
    if let Some(rest) = v.strip_prefix("color(") {
        let inner = rest.strip_suffix(')').ok_or("a malformed color()")?;
        let mut parts = inner.split_whitespace();
        if parts.next() != Some("display-p3") {
            return fail(format!("the colour {v:?} is not supported by this actool"));
        }
        let c: Vec<f32> = parts.map(|p| p.parse::<f32>().map_err(|_| format!("the colour {v:?} is not supported by this actool"))).collect::<Result<_, _>>()?;
        if c.len() != 3 {
            return fail(format!("the colour {v:?} is not supported by this actool"));
        }
        return Ok(Paint::Color(Color { space: 3, components: vec![c[0] as f64, c[1] as f64, c[2] as f64, 1.0] }));
    }
    if v == "currentColor" || v == "inherit" || v.starts_with("var(") || v.starts_with("icc-color") {
        return fail(format!("the paint {v:?} is not supported by this actool"));
    }
    let c: svgtypes::Color = v.parse().map_err(|_| format!("the colour {v:?} is not supported by this actool"))?;
    let f = |x: u8| x as f64 / 255.0;
    Ok(Paint::Color(Color { space: 1, components: vec![f(c.red), f(c.green), f(c.blue), f(c.alpha)] }))
}

/// An element's own value of a property: its style attribute's last
/// declaration of it, or else its attribute.
fn own_property<'a>(n: roxmltree::Node<'a, 'a>, name: &str) -> Option<&'a str> {
    let from_style = n.attribute("style").and_then(|s| s.split(';').filter_map(|d| d.split_once(':')).filter(|(k, _)| k.trim() == name).map(|(_, v)| v).last());
    from_style.or_else(|| n.attribute(name))
}

/// Whether every colour an SVG paints is light (`LIGHT`). Xcode reads the
/// SVG's elements, not its pixels: an element hidden, transparent or
/// covered counts; one in `defs`, a clip path or a mask doesn't; a
/// gradient's stops count, whatever their opacity; an SVG painting nothing
/// is light (47 probes).
fn svg_is_light(data: &[u8]) -> Result<bool, String> {
    let doc = parse_svg(data)?;
    const SKIPPED: [&str; 9] = ["defs", "clipPath", "mask", "linearGradient", "radialGradient", "title", "desc", "metadata", "stop"];
    // Text makes CoreSVG log an error (one probe): not established.
    const GRAPHICS: [&str; 7] = ["path", "rect", "circle", "ellipse", "line", "polyline", "polygon"];
    const CONTAINERS: [&str; 3] = ["svg", "g", "a"];
    for n in doc.descendants().filter(|n| n.is_element()) {
        let tag = n.tag_name().name();
        if !(SKIPPED.contains(&tag) || GRAPHICS.contains(&tag) || CONTAINERS.contains(&tag)) {
            return fail(format!("the SVG element <{tag}> is not supported by this actool"));
        }
    }
    let mut colors = Vec::new();
    let mut stack = vec![(doc.root_element(), "#000000".to_string(), "none".to_string())];
    while let Some((n, inherited_fill, inherited_stroke)) = stack.pop() {
        let tag = n.tag_name().name();
        if SKIPPED.contains(&tag) {
            continue;
        }
        let fill = own_property(n, "fill").map(str::to_string).unwrap_or(inherited_fill);
        let stroke = own_property(n, "stroke").map(str::to_string).unwrap_or(inherited_stroke);
        if GRAPHICS.contains(&tag) {
            for p in [&fill, &stroke] {
                colors.extend(paint_colors(&doc, p)?);
            }
        }
        for child in n.children().filter(|c| c.is_element()) {
            stack.push((child, fill.clone(), stroke.clone()));
        }
    }
    all_light(&colors)
}

fn paint_colors(doc: &roxmltree::Document, value: &str) -> Result<Vec<Color>, String> {
    match paint(value)? {
        Paint::None => Ok(Vec::new()),
        Paint::Color(c) => Ok(vec![c]),
        Paint::Url(id) => {
            let Some(target) = doc.descendants().find(|n| n.attribute("id") == Some(id.as_str())) else {
                return fail(format!("a paint referring to a missing #{id} is not supported by this actool"));
            };
            if !["linearGradient", "radialGradient"].contains(&target.tag_name().name()) {
                return fail(format!("a paint referring to a <{}> is not supported by this actool", target.tag_name().name()));
            }
            if target.attributes().any(|a| a.name() == "href") {
                return fail("a gradient inheriting another's stops is not supported by this actool");
            }
            let mut out = Vec::new();
            for stop in target.children().filter(|c| c.has_tag_name("stop")) {
                match paint(own_property(stop, "stop-color").unwrap_or("#000000"))? {
                    Paint::Color(c) => out.push(c),
                    Paint::None => {}
                    Paint::Url(_) => return fail("a stop colour of a url() is not supported by this actool"),
                }
            }
            Ok(out)
        }
    }
}

/// A Vector rendition's payload: `RAWD`, 1 and the SVG as LZFSE compresses
/// it when that is smaller, else 0 and the SVG itself.
fn vector_payload(svg: &[u8]) -> Vec<u8> {
    let compressed = lzfse::encode(svg);
    let (flag, data) = if compressed.len() < svg.len() { (1u32, compressed) } else { (0, svg.to_vec()) };
    let mut out = b"DWAR".to_vec();
    out.extend_from_slice(&flag.to_le_bytes());
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&data);
    out
}

// --- Colours and gradients, as the catalog names them ---------------------------

/// The document's colours and gradients, named in the order they are first
/// used: `Color-1` (white) first, then the background's for light and for
/// dark, then each layer's for light, dark and tinted, front to back. Equal
/// ones share a name; gradients count from 1 with system-light and
/// system-dark, which keep their own names.
struct Paints {
    prefix: String,
    colors: Vec<Color>,
    gradients: Vec<(String, u32, Orientation, Vec<usize>)>,
    next_gradient: usize,
}

/// What a stack or group entry fills with: a colour or a gradient, by
/// index.
#[derive(Clone, Copy, PartialEq, Debug)]
enum PaintRef {
    Color(usize),
    Gradient(usize),
}

impl Paints {
    fn new(name: &str) -> Paints {
        let mut p = Paints { prefix: format!("{name}_Assets/"), colors: Vec::new(), gradients: Vec::new(), next_gradient: 1 };
        p.color(&Color::gray(1.0, 1.0, 6));
        p
    }

    fn color(&mut self, c: &Color) -> usize {
        if let Some(i) = self.colors.iter().position(|x| x == c) {
            return i;
        }
        self.colors.push(c.clone());
        self.colors.len() - 1
    }

    fn fill(&mut self, f: &Fill) -> PaintRef {
        let (kind, orientation, colors, name) = match f {
            Fill::Solid(c) => return PaintRef::Color(self.color(c)),
            Fill::Automatic(c, o) => (0, o.clone(), vec![c.clone()], None),
            Fill::Linear(cs, o) => (1, o.clone(), cs.clone(), None),
            Fill::SystemLight => (1, Orientation::default(), system_light(), Some("system-light")),
            Fill::SystemDark => (1, Orientation::default(), system_dark(), Some("system-dark")),
        };
        let stops: Vec<usize> = colors.iter().map(|c| self.color(c)).collect();
        if let Some(i) = self.gradients.iter().position(|g| g.1 == kind && g.2 == orientation && g.3 == stops) {
            return PaintRef::Gradient(i);
        }
        let name = name.map_or_else(|| format!("Gradient-{}", self.next_gradient), str::to_string);
        self.next_gradient += 1;
        self.gradients.push((name, kind, orientation, stops));
        PaintRef::Gradient(self.gradients.len() - 1)
    }

    fn name(&self, r: PaintRef) -> String {
        match r {
            PaintRef::Color(i) => format!("{}Color-{}", self.prefix, i + 1),
            PaintRef::Gradient(i) => format!("{}{}", self.prefix, self.gradients[i].0),
        }
    }

    fn part(r: PaintRef) -> u16 {
        match r {
            PaintRef::Color(_) => car::PART_COLOR,
            PaintRef::Gradient(_) => PART_GRADIENT,
        }
    }
}

fn le32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn tlv(out: &mut Vec<u8>, kind: u32, data: &[u8]) {
    le32(out, kind);
    le32(out, data.len() as u32);
    out.extend_from_slice(data);
}

/// A rendition of a document's (no size, scale or pixel format): `layout`,
/// `name`, its TLVs and its payload.
fn data_csi(name: &str, layout: u16, flags: u32, size: u32, scale: u32, pixel_format: &[u8; 4], tlvs: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut out = b"ISTC".to_vec();
    for v in [1, flags, size, size, scale] {
        le32(&mut out, v);
    }
    out.extend_from_slice(pixel_format);
    le32(&mut out, 0); // colour space
    le32(&mut out, 0); // modification time
    out.extend_from_slice(&layout.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    let mut n = name.as_bytes().to_vec();
    n.truncate(127);
    n.resize(128, 0);
    out.extend_from_slice(&n);
    le32(&mut out, tlvs.len() as u32);
    le32(&mut out, 1);
    le32(&mut out, 0);
    le32(&mut out, payload.len() as u32);
    out.extend_from_slice(tlvs);
    out.extend_from_slice(payload);
    out
}

fn composition(t: &mut Vec<u8>, opacity: f32) {
    let mut c = Vec::new();
    le32(&mut c, 0);
    c.extend_from_slice(&opacity.to_le_bytes());
    tlv(t, 0x3ec, &c);
}

/// A gradient (part 247, layout 1021): `ARGG`, its stop count and kind, 0,
/// its start and stop, then each stop's location and colour by name.
fn gradient_csi(name: &str, kind: u32, o: &Orientation, stops: &[String]) -> Vec<u8> {
    let mut t = Vec::new();
    composition(&mut t, 0.0);
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    let mut p = b"ARGG".to_vec();
    le32(&mut p, stops.len() as u32);
    le32(&mut p, kind);
    for v in [0.0f32, o.0[0], o.0[1], o.0[2], o.0[3]] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    for (i, s) in stops.iter().enumerate() {
        let location = if stops.len() == 1 { 0.0f32 } else { i as f32 / (stops.len() - 1) as f32 };
        p.extend_from_slice(&location.to_le_bytes());
        le32(&mut p, s.len() as u32 + 1);
        p.extend_from_slice(s.as_bytes());
        p.push(0);
    }
    data_csi(name, LAYOUT_GRADIENT, 0, 0, 0, &[0; 4], &t, &p)
}

/// One entry of a stack's or a group's list: where it goes, how it blends
/// and how opaque it is, what it shows (by part and identifier), and its
/// properties.
struct Entry {
    rect: (i32, i32, u32, u32),
    blend: u32,
    opacity: f32,
    part: u16,
    identifier: u16,
    /// Glass for a layer, lighting for a group, 0 for the background.
    flag: u32,
    blur: f32,
    fill: String,
    /// A group's specular, translucency and shadow.
    effects: [u32; 5],
}

/// A stack (layout 1019) or a group (layout 1020): its entries, back to
/// front, in four lists (0x3f4, 0x3fc to 0x3fe).
fn layered_csi(name: &str, layout: u16, size: u32, entries: &[Entry]) -> Vec<u8> {
    let n = entries.len() as u32;
    let list = |each: &dyn Fn(&Entry, &mut Vec<u8>)| {
        let mut v = Vec::new();
        le32(&mut v, n);
        le32(&mut v, 0);
        for e in entries {
            each(e, &mut v);
        }
        v
    };
    let mut t = Vec::new();
    tlv(
        &mut t,
        0x3f4,
        &list(&|e, v| {
            le32(v, 0);
            v.extend_from_slice(&e.rect.0.to_le_bytes());
            v.extend_from_slice(&e.rect.1.to_le_bytes());
            le32(v, e.rect.2);
            le32(v, e.rect.3);
            le32(v, e.blend);
            v.extend_from_slice(&e.opacity.to_le_bytes());
            le32(v, 16);
            for (a, x) in [(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, e.part), (car::ATTR_IDENTIFIER, e.identifier)] {
                v.extend_from_slice(&a.to_le_bytes());
                v.extend_from_slice(&x.to_le_bytes());
            }
            le32(v, 0);
        }),
    );
    tlv(
        &mut t,
        0x3fc,
        &list(&|e, v| {
            le32(v, e.flag);
            v.extend_from_slice(&e.blur.to_le_bytes());
            le32(v, e.fill.len() as u32 + 1);
            v.extend_from_slice(e.fill.as_bytes());
            v.push(0);
        }),
    );
    tlv(&mut t, 0x3fd, &list(&|e, v| e.effects.iter().for_each(|x| le32(v, *x))));
    tlv(&mut t, 0x3fe, &list(&|_, v| v.extend_from_slice(&[0; 12])));
    composition(&mut t, 1.0);
    if layout == LAYOUT_ICON_STACK {
        let uti = b"public.layeredimage\0";
        let mut u = Vec::new();
        le32(&mut u, uti.len() as u32);
        le32(&mut u, 0);
        u.extend_from_slice(uti);
        tlv(&mut t, 0x3ed, &u);
    }
    tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
    data_csi(name, layout, 0, size, 100, b"ATAD", &t, &[b"DWAR".as_slice(), &[0u8; 8]].concat())
}

/// Where a layer goes on the canvas: its image's size scaled by its and its
/// group's scales (rounded half to even), centred and translated (its
/// group's scale applying to its own translation), the corner rounded
/// down, below zero too (13 probes and 30 of the holdout's layers).
fn rect(width: f64, height: f64, group: Position, layer: Position) -> Result<(i32, i32, u32, u32), String> {
    let s = group.scale * layer.scale;
    let c = CANVAS as f64 / 2.0;
    let (cx, cy) = (c + group.tx + group.scale * layer.tx, c + group.ty + group.scale * layer.ty);
    let (w, h) = (width * s, height * s);
    let (x, y) = (cx - w / 2.0, cy - h / 2.0);
    if w < 0.0 || h < 0.0 {
        return fail("a layer of negative size is not supported by this actool");
    }
    // An edge exactly on a whole point, of a layer larger than half the
    // canvas from an image other than 1024 points, at a scale binary can't
    // hold exactly (1.34, not 2.5): Xcode puts some of them one point
    // further out, by a rule not established (101 of 1,934 such edges in a
    // sweep of image sizes and scales, every one of them in this region;
    // none below 536 points, from 1024, or at a scale in 1024ths, as the
    // real documents' 2.5 and 3 are).
    let unsure = |v: f64, size: f64, dim: f64| v.fract() == 0.0 && size > CANVAS as f64 / 2.0 && dim != CANVAS as f64;
    if (s * 1024.0).fract() != 0.0 && (unsure(x, w, width) || unsure(y, h, height)) {
        return fail("a scaled layer larger than half the icon with an edge exactly on a whole point is not supported by this actool (Xcode's rounding of it isn't established)");
    }
    Ok((x.floor() as i32, y.floor() as i32, w.round_ties_even() as u32, h.round_ties_even() as u32))
}

pub struct Layered {
    pub facets: Vec<Facet>,
    pub renditions: Vec<Rendition>,
}

impl Document {
    /// Whether, in dark, light layers take the background's light fill:
    /// when Xcode chooses the background for dark and the light one isn't
    /// dark (Visor's, nearly black, keeps its layers).
    fn swaps_light_layers(&self) -> Result<bool, String> {
        Ok(self.dark_automatic && self.fill.is_light_background()?)
    }

    /// The facets and renditions of the document's layers, as Xcode writes
    /// them for every platform: its images, colours and gradients, and per
    /// appearance its stack and groups.
    pub fn layered(&self) -> Result<Layered, String> {
        let mac = self.platform.mac;
        let mut paints = Paints::new(&self.name);
        let light_fill = paints.fill(&self.fill);
        let dark_fill = paints.fill(&self.dark_fill);
        let swaps = self.swaps_light_layers()?;
        // Each layer's fill per appearance, in order.
        let mut fills: Vec<Vec<[Option<PaintRef>; 3]>> = Vec::new();
        for g in &self.groups {
            let mut group = Vec::new();
            for l in &g.layers {
                let image = self.images.iter().find(|i| i.file == l.image).unwrap();
                let mut refs = [None; 3];
                for (k, a) in Appearance::ALL.iter().enumerate() {
                    // Hidden in this appearance, a layer keeps its fill.
                    let look = &l.looks[k];
                    refs[k] = match (*a, &look.fill) {
                        // In dark, a light layer that isn't given a fill or
                        // an opacity of its own for dark takes the
                        // background's light fill, while Xcode chooses the
                        // background's (twenty probes; the opacity, any
                        // value, eight more: glass, position and other
                        // appearances' opacity don't count).
                        (Appearance::Dark, f) if !look.fill_for_appearance && !look.opacity_for_appearance && swaps => {
                            // A PNG layer without a fill keeps its own (one
                            // of the holdout's documents); with one, it goes
                            // by the fill, as an SVG's does (another).
                            let light = match f {
                                None if !image.svg => false,
                                None => image.light,
                                Some(f) => f.is_light()?,
                            };
                            if light {
                                Some(self.light_fill_with_alpha(&mut paints, light_fill, f.as_ref())?)
                            } else {
                                f.as_ref().map(|f| paints.fill(f))
                            }
                        }
                        (_, f) => f.as_ref().map(|f| paints.fill(f)),
                    };
                }
                group.push(refs);
            }
            fills.push(group);
        }

        let mut facets = Vec::new();
        let mut renditions = Vec::new();
        let facet = |name: &str, part: u16| Facet { name: name.to_string(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, part), (car::ATTR_IDENTIFIER, name_identifier(name))] };
        let attrs = |appearance: Option<u16>, id: u16, part: u16| {
            let mut a = Vec::new();
            if let Some(v) = appearance {
                a.push((car::ATTR_APPEARANCE, v));
            }
            a.extend([(car::ATTR_SCALE, 1), (car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, part)]);
            a
        };

        for image in &self.images {
            facets.push(facet(&image.facet, car::PART_IMAGE));
            let id = name_identifier(&image.facet);
            let csi = if image.svg {
                let mut t = Vec::new();
                composition(&mut t, 1.0);
                tlv(&mut t, 0x3ee, &1u32.to_le_bytes());
                data_csi("image.svg", LAYOUT_VECTOR, 4, 0, 0, b" GVS", &t, &vector_payload(&image.data))
            } else {
                // As a tvOS stack's layers: opaque when its PNG has no
                // alpha. Grey ones as GA8, opaque when every pixel is, as
                // image sets' (two probes).
                let img = crate::image::decode_png(&image.data)?;
                let bgra = img.premultiplied_bgra();
                let (w, h) = (img.width, img.height);
                if img.is_gray() {
                    let opaque = bgra.chunks_exact(4).all(|p| p[3] == 255);
                    let ga: Vec<u8> = bgra.chunks_exact(4).flat_map(|p| [p[0], p[3]]).collect();
                    let payload = dmp2::encode_gray(w, h, &ga, opaque).map_err(|e| format!("{}: {e}", image.file))?;
                    car::dmp2_pixels_csi("image.png", w as u32, h as u32, 1, 16, car::Pixels::Gray, opaque, &payload, None)
                } else {
                    let opaque = !crate::image::png_has_alpha(&image.data);
                    let payload = dmp2::encode(w, h, &bgra, opaque).map_err(|e| format!("{}: {e}", image.file))?;
                    car::dmp2_image_csi("image.png", w as u32, h as u32, 1, &payload)
                }
            };
            renditions.push(Rendition { attrs: attrs(None, id, car::PART_IMAGE), csi });
        }
        for (i, c) in paints.colors.iter().enumerate() {
            let name = paints.name(PaintRef::Color(i));
            facets.push(facet(&name, car::PART_COLOR));
            let components = c.stored();
            renditions.push(Rendition { attrs: attrs(None, name_identifier(&name), car::PART_COLOR), csi: car::color_csi(&name, c.space, &components) });
        }
        for (i, (_, kind, o, stops)) in paints.gradients.iter().enumerate() {
            let name = paints.name(PaintRef::Gradient(i));
            facets.push(facet(&name, PART_GRADIENT));
            let stops: Vec<String> = stops.iter().map(|&s| paints.name(PaintRef::Color(s))).collect();
            renditions.push(Rendition { attrs: attrs(None, name_identifier(&name), PART_GRADIENT), csi: gradient_csi(&name, *kind, o, &stops) });
        }

        let stack_id = name_identifier(&self.name);
        for g in &self.groups {
            facets.push(facet(&format!("{}/{}", self.name, g.name), PART_ICON_GROUP));
        }
        for (k, a) in Appearance::ALL.iter().enumerate() {
            let value = a.value(mac);
            let background = if *a == Appearance::Dark { dark_fill } else { light_fill };
            let mut stack = vec![Entry {
                rect: (0, 0, 0, 0),
                blend: 0,
                opacity: 1.0,
                part: Paints::part(background),
                identifier: name_identifier(&paints.name(background)),
                flag: 0,
                blur: 0.0,
                fill: String::new(),
                effects: [0; 5],
            }];
            for (gi, g) in self.groups.iter().enumerate().rev() {
                let look = &g.looks[k];
                let group_name = format!("{}/{}", self.name, g.name);
                let group_id = name_identifier(&group_name);
                stack.push(Entry {
                    rect: (0, 0, 0, 0),
                    blend: g.blend,
                    opacity: look.opacity,
                    part: PART_ICON_GROUP,
                    identifier: group_id,
                    flag: look.lighting,
                    blur: look.blur,
                    fill: String::new(),
                    effects: [look.specular as u32, look.translucency.to_bits(), look.shadow_kind, look.shadow_opacity.to_bits(), 0],
                });
                let mut layers = Vec::new();
                for (li, l) in g.layers.iter().enumerate().rev() {
                    let ll = &l.looks[k];
                    let image = self.images.iter().find(|i| i.file == l.image).unwrap();
                    layers.push(Entry {
                        rect: rect(image.width, image.height, look.position, ll.position)?,
                        blend: l.blend,
                        // Hidden in this appearance only: kept, invisible
                        // and without glass (two probes).
                        opacity: if ll.hidden { 0.0 } else { ll.opacity },
                        part: car::PART_IMAGE,
                        identifier: name_identifier(&image.facet),
                        flag: (ll.glass && !ll.hidden) as u32,
                        blur: 0.0,
                        fill: fills[gi][li][k].map(|r| paints.name(r)).unwrap_or_default(),
                        effects: [0; 5],
                    });
                }
                renditions.push(Rendition { attrs: attrs(Some(value), group_id, PART_ICON_GROUP), csi: layered_csi("IconGroup", LAYOUT_ICON_GROUP, 0, &layers) });
            }
            renditions.push(Rendition { attrs: attrs(Some(value), stack_id, PART_ICON_STACK), csi: layered_csi(&format!("{}.iconstack", self.name), LAYOUT_ICON_STACK, CANVAS, &stack) });
        }
        let mut ids: Vec<u16> = facets.iter().map(|f| f.attrs[2].1).collect();
        ids.push(stack_id);
        ids.sort();
        if ids.windows(2).any(|w| w[0] == w[1]) {
            return fail("two of the document's names share an identifier, which this actool doesn't support");
        }
        Ok(Layered { facets, renditions })
    }

    /// The background's light fill for a light layer in dark: as it is for a
    /// layer without a fill of its own or with a system one, otherwise with
    /// its colours' alpha the layer's fill's first colour's, itself when that
    /// changes nothing (eight probes and six of the holdouts' documents).
    fn light_fill_with_alpha(&self, paints: &mut Paints, light: PaintRef, layer: Option<&Fill>) -> Result<PaintRef, String> {
        // Without a fill, or with a system one, as it is (one of the
        // holdout's documents each).
        let Some(layer) = layer.filter(|f| !matches!(f, Fill::SystemLight | Fill::SystemDark)) else {
            return Ok(light);
        };
        // The alpha of the fill's first colour, whatever the others' (six
        // probes and three of the second holdout's documents).
        let alpha = layer.stops().first().map_or(1.0, Color::alpha);
        if alpha == 1.0 && matches!(self.fill, Fill::SystemLight | Fill::SystemDark) {
            return Ok(light);
        }
        let copy = match &self.fill {
            Fill::Solid(c) => Fill::Solid(c.with_alpha(alpha)),
            Fill::Automatic(c, o) => Fill::Automatic(c.with_alpha(alpha), o.clone()),
            Fill::Linear(cs, o) => Fill::Linear(cs.iter().map(|c| c.with_alpha(alpha)).collect(), o.clone()),
            // Translucent, system-light becomes a gradient of its own, from
            // white to 0.961 (not its 0.925) grey, whatever the alpha or the
            // fill's colour space (ten probes).
            Fill::SystemLight => Fill::Linear(vec![Color::gray(1.0, alpha, 2), Color::gray(0.961, alpha, 2)], Orientation::default()),
            Fill::SystemDark => return fail("a translucent light layer on a system-dark background is not supported by this actool"),
        };
        Ok(paints.fill(&copy))
    }
}

// --- Rasters ---------------------------------------------------------------------------

fn paint_shader(f: &Fill, alpha: f32) -> tiny_skia::Shader<'static> {
    let c = |c: &Color| {
        let [r, g, b, a] = c.srgb8();
        tiny_skia::Color::from_rgba(r, g, b, a * alpha).unwrap()
    };
    let (stops, o) = match f {
        Fill::Solid(x) => return tiny_skia::Shader::SolidColor(c(x)),
        // Lighter at the start, the colour at the stop.
        Fill::Automatic(x, o) => {
            let [r, g, b, a] = x.srgb8();
            let lighter = tiny_skia::Color::from_rgba(r + (1.0 - r) * 0.25, g + (1.0 - g) * 0.25, b + (1.0 - b) * 0.25, a * alpha).unwrap();
            (vec![lighter, c(x)], o.clone())
        }
        Fill::Linear(xs, o) => (xs.iter().map(c).collect(), o.clone()),
        Fill::SystemLight => (system_light().iter().map(c).collect(), Orientation::default()),
        Fill::SystemDark => (system_dark().iter().map(c).collect(), Orientation::default()),
    };
    let n = CANVAS as f32;
    let first = stops[0];
    let stops: Vec<tiny_skia::GradientStop> = stops.iter().enumerate().map(|(i, s)| tiny_skia::GradientStop::new(i as f32 / (stops.len() - 1).max(1) as f32, *s)).collect();
    let (start, end) = (tiny_skia::Point::from_xy(o.0[0] * n, o.0[1] * n), tiny_skia::Point::from_xy(o.0[2] * n, o.0[3] * n));
    tiny_skia::LinearGradient::new(start, end, stops, tiny_skia::SpreadMode::Pad, tiny_skia::Transform::identity()).unwrap_or(tiny_skia::Shader::SolidColor(first))
}

/// An image drawn at its place, alone on a canvas.
fn draw_image(image: &Image, r: (i32, i32, u32, u32)) -> Result<tiny_skia::Pixmap, String> {
    let mut layer = tiny_skia::Pixmap::new(CANVAS, CANVAS).unwrap();
    let (sx, sy) = (r.2 as f32 / image.width.max(1.0) as f32, r.3 as f32 / image.height.max(1.0) as f32);
    let placement = tiny_skia::Transform::from_row(sx, 0.0, 0.0, sy, r.0 as f32, r.1 as f32);
    if image.svg {
        let tree = usvg::Tree::from_data(&image.data, &usvg::Options::default()).map_err(|e| format!("{}: {e}", image.file))?;
        let fit = tiny_skia::Transform::from_scale(image.width as f32 / tree.size().width(), image.height as f32 / tree.size().height());
        resvg::render(&tree, placement.pre_concat(fit), &mut layer.as_mut());
    } else {
        let img = crate::image::decode_png(&image.data)?;
        let size = tiny_skia::IntSize::from_wh(img.width as u32, img.height as u32).ok_or("empty image")?;
        let mut premultiplied = Vec::with_capacity(img.data.len());
        for p in img.data.chunks_exact(4) {
            let a = p[3] as u32;
            let m = |c: u8| ((c as u32 * a + 127) / 255) as u8;
            premultiplied.extend_from_slice(&[m(p[0]), m(p[1]), m(p[2]), p[3]]);
        }
        let pix = tiny_skia::Pixmap::from_vec(premultiplied, size).ok_or("bad image")?;
        layer.draw_pixmap(0, 0, pix.as_ref(), &tiny_skia::PixmapPaint { quality: tiny_skia::FilterQuality::Bicubic, ..Default::default() }, placement, None);
    }
    Ok(layer)
}

/// The alpha channel blurred by three passes of a box blur in each
/// direction (close to a Gaussian).
fn blurred_alpha(src: &[u8], w: usize, h: usize, radius: usize) -> Vec<f32> {
    let mut a: Vec<f32> = src.chunks_exact(4).map(|p| p[3] as f32 / 255.0).collect();
    let mut tmp = vec![0f32; a.len()];
    let width = (2 * radius + 1) as f32;
    for (lines, len, step, stride) in [(h, w, 1, w), (w, h, w, 1)].repeat(3) {
        for l in 0..lines {
            let base = l * stride;
            let at = |a: &[f32], i: isize| if i < 0 || i as usize >= len { 0.0 } else { a[base + i as usize * step] };
            let r = radius as isize;
            let mut acc: f32 = (-r..=r).map(|i| at(&a, i)).sum();
            for i in 0..len as isize {
                tmp[base + i as usize * step] = acc / width;
                acc += at(&a, i + r + 1) - at(&a, i - r);
            }
        }
        std::mem::swap(&mut a, &mut tmp);
    }
    a
}

impl Document {
    /// This actool's own rendering of an appearance, 1024 px square and
    /// opaque: the background, then each group's layers back to front
    /// (filled where the document fills them) over a blurred shadow, with
    /// no glass, specular highlights or translucency. Tinted is the dark
    /// rendering's layers, in grey, on black.
    pub fn render(&self, appearance: Appearance) -> Result<Rgba, String> {
        let k = Appearance::ALL.iter().position(|a| *a == appearance).unwrap();
        let swaps = self.swaps_light_layers()?;
        let mut canvas = tiny_skia::Pixmap::new(CANVAS, CANVAS).unwrap();
        let black = Fill::Solid(Color::gray(0.0, 1.0, 6));
        let background = match appearance {
            Appearance::Light => &self.fill,
            Appearance::Dark => &self.dark_fill,
            Appearance::Tinted => &black,
        };
        let full = tiny_skia::Rect::from_xywh(0.0, 0.0, CANVAS as f32, CANVAS as f32).unwrap();
        canvas.fill_rect(full, &tiny_skia::Paint { shader: paint_shader(background, 1.0), ..Default::default() }, tiny_skia::Transform::identity(), None);
        for g in self.groups.iter().rev() {
            let look = &g.looks[k];
            let mut content = tiny_skia::Pixmap::new(CANVAS, CANVAS).unwrap();
            for l in g.layers.iter().rev() {
                let ll = &l.looks[k];
                if ll.hidden {
                    continue;
                }
                let image = self.images.iter().find(|i| i.file == l.image).unwrap();
                let mut layer = draw_image(image, rect(image.width, image.height, look.position, ll.position)?)?;
                let fill = match (appearance, &ll.fill) {
                    (Appearance::Dark, None) if image.svg && image.light && swaps && !ll.opacity_for_appearance => Some(&self.fill),
                    (_, f) => f.as_ref(),
                };
                if let Some(f) = fill {
                    let mut painted = tiny_skia::Pixmap::new(CANVAS, CANVAS).unwrap();
                    painted.fill_rect(full, &tiny_skia::Paint { shader: paint_shader(f, 1.0), ..Default::default() }, tiny_skia::Transform::identity(), None);
                    let mask = tiny_skia::Mask::from_pixmap(layer.as_ref(), tiny_skia::MaskType::Alpha);
                    layer.fill(tiny_skia::Color::TRANSPARENT);
                    layer.draw_pixmap(0, 0, painted.as_ref(), &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), Some(&mask));
                }
                content.draw_pixmap(0, 0, layer.as_ref(), &tiny_skia::PixmapPaint { opacity: ll.opacity.clamp(0.0, 1.0), ..Default::default() }, tiny_skia::Transform::identity(), None);
            }
            if look.shadow_kind != 0 && look.shadow_opacity > 0.0 {
                let (w, h) = (CANVAS as usize, CANVAS as usize);
                let a = blurred_alpha(content.data(), w, h, 12);
                let offset = 10usize;
                let opacity = look.shadow_opacity.clamp(0.0, 1.0);
                let data = canvas.data_mut();
                for y in offset..h {
                    for x in 0..w {
                        let s = a[(y - offset) * w + x] * opacity;
                        for c in &mut data[(y * w + x) * 4..][..3] {
                            *c = (*c as f32 * (1.0 - s)).round() as u8;
                        }
                    }
                }
            }
            canvas.draw_pixmap(0, 0, content.as_ref(), &tiny_skia::PixmapPaint { opacity: look.opacity.clamp(0.0, 1.0), ..Default::default() }, tiny_skia::Transform::identity(), None);
        }
        // The canvas is opaque, so its premultiplied pixels are straight ones.
        let mut data = canvas.take();
        for p in data.chunks_exact_mut(4) {
            if appearance == Appearance::Tinted {
                let y = (0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32).round() as u8;
                p[..3].fill(y);
            }
            p[3] = 255;
        }
        Ok(Rgba { width: CANVAS as usize, height: CANVAS as usize, data })
    }

    /// This actool's macOS rendering: the light one inside macOS's rounded
    /// square (824 pt on the 1024 pt canvas), transparent around it.
    pub fn render_mac(&self) -> Result<Rgba, String> {
        let flat = self.render(Appearance::Light)?;
        let size = tiny_skia::IntSize::from_wh(CANVAS, CANVAS).unwrap();
        let src = tiny_skia::Pixmap::from_vec(flat.data, size).unwrap();
        let mut out = tiny_skia::Pixmap::new(CANVAS, CANVAS).unwrap();
        let (inset, side, radius) = (100.0f32, 824.0f32, 185.0f32);
        let mut pb = tiny_skia::PathBuilder::new();
        let (x0, y0, x1, y1) = (inset, inset, inset + side, inset + side);
        pb.move_to(x0 + radius, y0);
        pb.line_to(x1 - radius, y0);
        pb.quad_to(x1, y0, x1, y0 + radius);
        pb.line_to(x1, y1 - radius);
        pb.quad_to(x1, y1, x1 - radius, y1);
        pb.line_to(x0 + radius, y1);
        pb.quad_to(x0, y1, x0, y1 - radius);
        pb.line_to(x0, y0 + radius);
        pb.quad_to(x0, y0, x0 + radius, y0);
        pb.close();
        let path = pb.finish().unwrap();
        let scale = side / CANVAS as f32;
        let pattern = tiny_skia::Pattern::new(src.as_ref(), tiny_skia::SpreadMode::Pad, tiny_skia::FilterQuality::Bicubic, 1.0, tiny_skia::Transform::from_row(scale, 0.0, 0.0, scale, inset, inset));
        out.fill_path(&path, &tiny_skia::Paint { shader: pattern, anti_alias: true, ..Default::default() }, tiny_skia::FillRule::Winding, tiny_skia::Transform::identity(), None);
        // Straight alpha from premultiplied, rounding to nearest.
        let mut data = out.take();
        for p in data.chunks_exact_mut(4) {
            let a = p[3] as u32;
            for c in &mut p[..3] {
                *c = if a == 0 { 0 } else { ((*c as u32 * 255 + a / 2) / a).min(255) as u8 };
            }
        }
        Ok(Rgba { width: CANVAS as usize, height: CANVAS as usize, data })
    }
}

/// A raster's name: Xcode's `<name><size>x<size>_<appearance>_` and, in place
/// of its random UUID, process id and time, zeros, cut to CoreUI's 127
/// bytes as Xcode's are (docs/clean-room/actool.md).
pub fn raster_name(name: &str, size: u32, appearance: &str) -> String {
    let mut n = format!("{name}{size}x{size}_{appearance}_00000000-0000-0000-0000-000000000000-0-0000000000000000.png");
    n.truncate(127);
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours() {
        assert_eq!(color("extended-srgb:0.07059,0.08235,0.10588,1.00000").unwrap(), Color { space: 4, components: vec![0.071, 0.082, 0.106, 1.0] });
        assert_eq!(color("srgb:0.0625,0.0635,0.0025,0.12345").unwrap().components, vec![0.062, 0.064, 0.003, 0.123]);
        assert_eq!(color("extended-srgb:-0.0625,1.0625,0,1").unwrap().components, vec![-0.062, 1.062, 0.0, 1.0]);
        assert_eq!(color("gray:0.5,1").unwrap().space, 2);
        assert!(color("extended-linear-srgb:1,0,0,1").is_err());
        assert!(color("srgb:1,0,0").is_err());
    }

    fn light(svg: &str) -> bool {
        svg_is_light(format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024">{svg}</svg>"#).as_bytes()).unwrap()
    }

    /// Probes of Xcode's: which SVGs it takes as light.
    #[test]
    fn light_svgs() {
        for c in ["white", "#D9D9D9", "#FFC0C0", "#B2FFB2", "#FFFFB2", "gainsboro", "#FFF", "rgb(255,255,255)"] {
            assert!(light(&format!(r#"<rect width="9" height="9" fill="{c}"/>"#)), "{c}");
        }
        for c in ["#D8D8D8", "#FFB3B3", "#FFFF00", "#00FFFF", "lightgray", "rgba(0,0,0,0)"] {
            assert!(!light(&format!(r#"<rect width="9" height="9" fill="{c}"/>"#)), "{c}");
        }
        assert!(!light(r#"<rect width="9" height="9"/>"#));
        assert!(light(r#"<g fill="black"><rect width="9" height="9" fill="white"/></g>"#));
        assert!(light(r#"<rect width="9" height="9" fill="black" style="fill:white"/>"#));
        assert!(!light(r#"<rect width="9" height="9" style="fill:white;fill:color(display-p3 0 0 0)"/>"#));
        assert!(!light(r#"<rect width="9" height="9" fill="white"/><rect width="9" height="9" fill="black" style="display:none"/>"#));
        assert!(light(r#"<defs><rect id="r" width="9" height="9" fill="black"/></defs><rect width="9" height="9" fill="white"/>"#));
        assert!(light(r#"<clipPath id="c"><rect width="9" height="9" fill="black"/></clipPath><rect width="9" height="9" fill="white" clip-path="url(#c)"/>"#));
        assert!(light(r#"<rect width="9" height="9" fill="none"/>"#));
        assert!(!light(r#"<line x2="9" y2="9" stroke="white"/>"#));
        assert!(!light(r#"<defs><linearGradient id="g"><stop offset="0"/><stop offset="1" stop-color="white"/></linearGradient></defs><rect width="9" height="9" fill="url(#g)"/>"#));
    }

    #[test]
    fn rects() {
        let p = Position::default();
        assert_eq!(rect(401.0, 301.0, p, p).unwrap(), (311, 361, 401, 301));
        assert_eq!(rect(250.5, 100.25, p, p).unwrap(), (386, 461, 250, 100));
        assert_eq!(rect(1024.0, 1024.0, p, Position { scale: 0.333, tx: 10.4, ty: -20.6 }).unwrap(), (351, 320, 341, 341));
        assert_eq!(rect(401.0, 301.0, p, Position { scale: 1.5, tx: 0.5, ty: 0.5 }).unwrap(), (211, 286, 602, 452));
        assert_eq!(rect(1024.0, 1024.0, Position { scale: 0.5, tx: 0.0, ty: 100.0 }, Position { scale: 0.5, tx: 100.0, ty: 0.0 }).unwrap(), (434, 484, 256, 256));
        assert_eq!(rect(1024.0, 1024.0, p, Position { scale: 2.0, tx: 0.0, ty: 0.0 }).unwrap(), (-512, -512, 2048, 2048));
    }

    #[test]
    fn names() {
        assert_eq!(sanitized("a-b.c(d)&e Gr\u{fc}\u{df}e").unwrap(), "a-b.c_d__e Gr__e");
        assert_eq!(raster_name("Icon", 1024, "UIAppearanceAny"), "Icon1024x1024_UIAppearanceAny_00000000-0000-0000-0000-000000000000-0-0000000000000000.png");
    }
}
