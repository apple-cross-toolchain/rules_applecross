//! The supported subset of iOS and tvOS storyboards and the nibs it compiles
//! to; see docs/clean-room/ibtool.md.

use crate::fonts;
use crate::nib::{self, reference, Object, Value};
use roxmltree::{Document, Node};
use std::collections::{HashMap, HashSet};

mod collection;
mod parent;
mod segue;
mod split;
mod tab;
mod table;

pub struct Compiled {
    /// (file name, contents) inside the .storyboardc directory.
    pub files: Vec<(String, Vec<u8>)>,
}

fn fail<T>(msg: impl Into<String>) -> Result<T, String> {
    Err(msg.into())
}

fn attr<'a>(n: Node<'a, '_>, name: &str) -> Result<&'a str, String> {
    n.attribute(name).ok_or_else(|| format!("<{}> has no {name} attribute", n.tag_name().name()))
}

fn children<'a, 'i>(n: Node<'a, 'i>, tag: &'a str) -> impl Iterator<Item = Node<'a, 'i>> + 'a {
    n.children().filter(move |c| c.is_element() && c.tag_name().name() == tag)
}

fn child<'a, 'i>(n: Node<'a, 'i>, tag: &'a str, key: &str) -> Option<Node<'a, 'i>> {
    children(n, tag).find(|c| c.attribute("key") == Some(key))
}

/// Interface Builder's own settings for how a controller is drawn while
/// editing, which compiled storyboards don't record (five probes); a
/// freeform size is another matter (see `compile`).
const EDITOR_ONLY: [&str; 8] = [
    "simulatedMetricsContainer", "simulatedNavigationBarMetrics", "simulatedStatusBarMetrics", "simulatedTabBarMetrics",
    "simulatedToolbarMetrics", "simulatedOrientationMetrics", "simulatedScreenMetrics", "freeformSimulatedSizeMetrics",
];

fn only_children(n: Node, allowed: &[&str]) -> Result<(), String> {
    for c in n.children().filter(|c| c.is_element()) {
        let editor_only = matches!(n.tag_name().name(), "viewController" | "tableViewController" | "collectionViewController" | "navigationController")
            && (EDITOR_ONLY.contains(&c.tag_name().name()) || c.has_tag_name("nil") && c.attribute("key").is_some_and(|k| k.starts_with("simulated")));
        // An empty list of gesture recognizers, or of animations (older
        // documents), is Interface Builder's alone (one probe).
        let empty_gestures = (c.has_tag_name("gestureRecognizers") || c.has_tag_name("animations")) && !c.children().any(|g| g.is_element());
        if !allowed.contains(&c.tag_name().name()) && !editor_only && !empty_gestures {
            return fail(format!("<{}> inside <{}> is not supported by this ibtool", c.tag_name().name(), n.tag_name().name()));
        }
    }
    Ok(())
}

/// Refuses attributes other than `allowed`, and values other than those
/// given (a `None` value accepts any).
fn only_attributes(n: Node, allowed: &[(&str, Option<&str>)]) -> Result<(), String> {
    // An object's name in Interface Builder's outline, and its warnings that
    // a view is misplaced or ambiguous; not compiled.
    for a in n.attributes().filter(|a| !matches!(a.name(), "userLabel" | "misplaced" | "ambiguous")) {
        match allowed.iter().find(|(name, _)| *name == a.name()) {
            Some((_, Some(v))) if *v != a.value() => {
                return fail(format!("{}=\"{}\" on <{}> is not supported by this ibtool", a.name(), a.value(), n.tag_name().name()))
            }
            Some(_) => {}
            None => return fail(format!("the {} attribute of <{}> is not supported by this ibtool", a.name(), n.tag_name().name())),
        }
    }
    Ok(())
}

fn number(s: &str) -> Result<f64, String> {
    s.parse::<f64>().map_err(|_| format!("not a number: {s}"))
}

/// A CGRect, CGPoint or UIEdgeInsets archived as NIB data: a 7 (doubles)
/// followed by the components in little-endian order.
fn doubles(v: &[f64]) -> Value {
    let mut d = vec![7u8];
    for x in v {
        d.extend_from_slice(&x.to_le_bytes());
    }
    Value::Data(d)
}

/// The shortest decimal that round-trips, as NSColor archives components.
/// `x` rounded to 10 significant digits, as `%.10g` would print it.
fn significant_10(x: f64) -> f64 {
    format!("{x:.9e}").parse().unwrap()
}

/// `x` as C's `%.3g` prints it: 3 significant digits, trailing zeros
/// dropped, in exponent form (two exponent digits at least) below 1e-4.
fn general_3(x: f64) -> String {
    if x == 0.0 {
        return "0".into();
    }
    let sci = format!("{x:.2e}");
    let (mantissa, exponent) = sci.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    let trim = |s: String| if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    if exponent < -4 || exponent >= 3 {
        let sign = if exponent < 0 { '-' } else { '+' };
        format!("{}e{sign}{:02}", trim(mantissa.to_string()), exponent.abs())
    } else {
        trim(format!("{x:.*}", (2 - exponent) as usize))
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Platform {
    Ios,
    Tvos,
}

/// A system colour's components as Xcode archives them, whatever the
/// document's copy says: a white and alpha, or sRGB and alpha.
enum SystemColor {
    Gray([f64; 2]),
    Rgb([f64; 4]),
}

/// iOS's system colours, as Xcode 27.0's ibtool archives them (measured).
const SYSTEM_COLORS: &[(&str, SystemColor)] = &[
    ("darkTextColor", SystemColor::Gray([0.0, 1.0])),
    ("groupTableViewBackgroundColor", SystemColor::Rgb([0.9490196078, 0.9490196078, 0.968627451, 1.0])),
    ("labelColor", SystemColor::Gray([0.0, 1.0])),
    ("lightTextColor", SystemColor::Gray([1.0, 0.6])),
    ("linkColor", SystemColor::Rgb([0.0, 0.4784313725, 1.0, 1.0])),
    ("opaqueSeparatorColor", SystemColor::Rgb([0.7764705882, 0.7764705882, 0.7843137255, 1.0])),
    ("placeholderTextColor", SystemColor::Rgb([0.2352941176, 0.2352941176, 0.262745098, 0.2980392157])),
    ("quaternaryLabelColor", SystemColor::Rgb([0.2352941176, 0.2352941176, 0.262745098, 0.1764705882])),
    ("quaternarySystemFillColor", SystemColor::Rgb([0.4549019608, 0.4549019608, 0.5019607843, 0.08])),
    ("secondaryLabelColor", SystemColor::Rgb([0.2352941176, 0.2352941176, 0.262745098, 0.6])),
    ("secondarySystemBackgroundColor", SystemColor::Rgb([0.9490196078, 0.9490196078, 0.968627451, 1.0])),
    ("secondarySystemFillColor", SystemColor::Rgb([0.4705882353, 0.4705882353, 0.5019607843, 0.16])),
    ("secondarySystemGroupedBackgroundColor", SystemColor::Gray([1.0, 1.0])),
    ("separatorColor", SystemColor::Rgb([0.2352941176, 0.2352941176, 0.262745098, 0.12])),
    ("systemBackgroundColor", SystemColor::Gray([1.0, 1.0])),
    ("systemBlueColor", SystemColor::Rgb([0.0, 0.5333333333, 1.0, 1.0])),
    ("systemBrownColor", SystemColor::Rgb([0.6745098039, 0.4980392157, 0.368627451, 1.0])),
    ("systemCyanColor", SystemColor::Rgb([0.0, 0.7529411765, 0.9098039216, 1.0])),
    ("systemFillColor", SystemColor::Rgb([0.4705882353, 0.4705882353, 0.5019607843, 0.2])),
    ("systemGray2Color", SystemColor::Rgb([0.6823529412, 0.6823529412, 0.6980392157, 1.0])),
    ("systemGray3Color", SystemColor::Rgb([0.7803921569, 0.7803921569, 0.8, 1.0])),
    ("systemGray4Color", SystemColor::Rgb([0.8196078431, 0.8196078431, 0.8392156863, 1.0])),
    ("systemGray5Color", SystemColor::Rgb([0.8980392157, 0.8980392157, 0.9176470588, 1.0])),
    ("systemGray6Color", SystemColor::Rgb([0.9490196078, 0.9490196078, 0.968627451, 1.0])),
    ("systemGrayColor", SystemColor::Rgb([0.5568627451, 0.5568627451, 0.5764705882, 1.0])),
    ("systemGreenColor", SystemColor::Rgb([0.2039215686, 0.7803921569, 0.3490196078, 1.0])),
    ("systemGroupedBackgroundColor", SystemColor::Rgb([0.9490196078, 0.9490196078, 0.968627451, 1.0])),
    ("systemIndigoColor", SystemColor::Rgb([0.3803921569, 0.3333333333, 0.9607843137, 1.0])),
    ("systemMintColor", SystemColor::Rgb([0.0, 0.7843137255, 0.7019607843, 1.0])),
    ("systemOrangeColor", SystemColor::Rgb([1.0, 0.5529411765, 0.1568627451, 1.0])),
    ("systemPinkColor", SystemColor::Rgb([1.0, 0.1764705882, 0.3333333333, 1.0])),
    ("systemPurpleColor", SystemColor::Rgb([0.7960784314, 0.1882352941, 0.8784313725, 1.0])),
    ("systemRedColor", SystemColor::Rgb([1.0, 0.2196078431, 0.2352941176, 1.0])),
    ("systemTealColor", SystemColor::Rgb([0.0, 0.7647058824, 0.8156862745, 1.0])),
    ("systemYellowColor", SystemColor::Rgb([1.0, 0.8, 0.0, 1.0])),
    ("tertiaryLabelColor", SystemColor::Rgb([0.2352941176, 0.2352941176, 0.262745098, 0.2980392157])),
    ("tertiarySystemBackgroundColor", SystemColor::Gray([1.0, 1.0])),
    ("tertiarySystemFillColor", SystemColor::Rgb([0.462745098, 0.462745098, 0.5019607843, 0.12])),
    ("tertiarySystemGroupedBackgroundColor", SystemColor::Rgb([0.9490196078, 0.9490196078, 0.968627451, 1.0])),
    ("tintColor", SystemColor::Rgb([0.0, 0.5333333333, 1.0, 1.0])),
];

/// Colour components as Interface Builder archives them: each rounded to 10
/// significant digits, as a float followed by the double when the float
/// isn't exact, then the components as text (`%.3g`, alpha only when not 1).
fn color_components(names: &[(&'static str, &'static str)], values: &[f64]) -> (Vec<(&'static str, Value)>, Vec<u8>) {
    let values: Vec<f64> = values.iter().map(|&x| significant_10(x)).collect();
    exact_color_components(names, &values)
}

/// As `color_components`, without rounding: a named colour's components
/// are kept as the document gives them.
fn exact_color_components(names: &[(&'static str, &'static str)], values: &[f64]) -> (Vec<(&'static str, Value)>, Vec<u8>) {
    let mut keys = Vec::new();
    for (&(key, double_key), &x) in names.iter().zip(values) {
        keys.push((key, Value::Float(x as f32)));
        if (x as f32) as f64 != x {
            keys.push((double_key, Value::Double(x)));
        }
    }
    let (alpha, rest) = values.split_last().unwrap();
    let mut text: Vec<String> = rest.iter().map(|&x| general_3(x)).collect();
    if *alpha != 1.0 {
        text.push(general_3(*alpha));
    }
    (keys, text.join(" ").into_bytes())
}

const RGB_KEYS: [(&str, &str); 4] = [("UIRed", "UIRed-Double"), ("UIGreen", "UIGreen-Double"), ("UIBlue", "UIBlue-Double"), ("UIAlpha", "UIAlpha-Double")];
const GRAY_KEYS: [(&str, &str); 2] = [("UIWhite", "UIWhite-Double"), ("UIAlpha", "UIAlpha-Double")];

fn autoresizing_mask(n: Node) -> Result<i64, String> {
    let mut m = 0;
    for (name, bit) in [("flexibleMinX", 1), ("widthSizable", 2), ("flexibleMaxX", 4), ("flexibleMinY", 8), ("heightSizable", 16), ("flexibleMaxY", 32)] {
        if n.attribute(name) == Some("YES") {
            m |= bit;
        }
    }
    Ok(m)
}

/// NSLayoutAttribute.
fn layout_attribute(name: &str) -> Result<i64, String> {
    Ok(match name {
        "left" => 1,
        "right" => 2,
        "top" => 3,
        "bottom" => 4,
        "leading" => 5,
        "trailing" => 6,
        "width" => 7,
        "height" => 8,
        "centerX" => 9,
        "centerY" => 10,
        other => return fail(format!("constraint attribute {other} is not supported by this ibtool")),
    })
}

/// A nib being built: object names, and the strings and numbers it shares.
/// Xcode's nibs hold one object per distinct string or number, encoded
/// where it is first used.
struct Nib {
    platform: Platform,
    /// Whether the storyboard is in `Base.lproj`, where Xcode makes the
    /// strings a translation replaces localizable.
    localizable: bool,
    /// The document's `colorMatched`: without it, colours' components are
    /// taken as they are rather than converted from their colour space.
    color_matched: bool,
    shared: HashMap<String, String>,
    next: usize,
}

impl Nib {
    fn new(platform: Platform, color_matched: bool, localizable: bool) -> Nib {
        Nib { platform, color_matched, localizable, shared: HashMap::new(), next: 0 }
    }

    fn fresh(&mut self) -> String {
        self.next += 1;
        format!("#{}", self.next)
    }

    fn shared(&mut self, key: String, make: impl FnOnce() -> Object) -> Value {
        if let Some(name) = self.shared.get(&key) {
            return reference(name.clone());
        }
        let name = self.fresh();
        self.shared.insert(key, name.clone());
        Value::New(make().named(name))
    }

    /// The object shared under `key`, if there is one yet.
    fn existing(&self, key: &str) -> Option<Value> {
        self.shared.get(key).map(|name| reference(name.clone()))
    }

    fn string(&mut self, s: &str) -> Value {
        self.shared(format!("s:{s}"), || Object::new("NSString", vec![("NS.bytes", Value::Data(s.as_bytes().to_vec()))]))
    }

    fn double_number(&mut self, d: f64) -> Value {
        self.shared(format!("d:{d}"), || Object::new("NSNumber", vec![("NS.dblval", Value::Double(d))]))
    }

    /// A named image, as the nib refers to it: one placeholder per name.
    fn image(&mut self, name: &str) -> Value {
        if let Some(v) = self.existing(&format!("image:{name}")) {
            return v;
        }
        let resource = self.string(name);
        self.shared(format!("image:{name}"), || Object::new("UIImageNibPlaceholder", vec![
            ("UIImageWidth", Value::Float(1.0)),
            ("UIImageHeight", Value::Float(1.0)),
            ("UIResourceName", resource),
            ("UISystemSymbolResourceName", Value::Nil),
            ("UISymbolImageConfiguration", Value::Nil),
            ("UIResourceCatalogName", Value::Nil),
        ]))
    }

    /// A string a translation replaces, the `property` of the object `id`:
    /// in `Base.lproj` an `NSLocalizableString` keyed `<id>.<property>`
    /// with the string as its development value. An empty string is never
    /// localized.
    fn text(&mut self, id: &str, property: &str, s: &str) -> Value {
        if !self.localizable || s.is_empty() {
            return self.string(s);
        }
        let key = self.string(&format!("{id}.{property}"));
        let dev = self.string(s);
        Value::New(Object::new("NSLocalizableString", vec![
            ("NS.bytes", Value::Data(s.as_bytes().to_vec())),
            ("NSKey", key),
            ("NSDev", dev),
        ]))
    }

    /// The nib's one empty array, encoded where first used.
    fn empty_array(&mut self) -> Value {
        self.shared("empty array".into(), || Object::new("NSArray", vec![("NSInlinedValue", Value::Bool(true))]))
    }

    fn int_number(&mut self, i: i64) -> Value {
        self.shared(format!("i:{i}"), || Object::new("NSNumber", vec![("NS.intval", Value::Int(i))]))
    }

    fn array(&mut self, class: &'static str, items: Vec<Value>) -> Value {
        let mut values = vec![("NSInlinedValue", Value::Bool(true))];
        values.extend(items.into_iter().map(|v| ("UINibEncoderEmptyKey", v)));
        Value::New(Object::new(class, values))
    }

    fn system_color(&mut self, name: &str, color: &SystemColor) -> Value {
        let name_value = self.string(name);
        let (count, components, text_key, space) = match color {
            SystemColor::Gray(c) => (2, color_components(&GRAY_KEYS, c), "NSWhite", 4),
            SystemColor::Rgb(c) => (4, color_components(&RGB_KEYS, c), "NSRGB", 2),
        };
        self.shared(format!("color:{name}"), || {
            let mut keys = vec![("UISystemColorName", name_value), ("UIColorComponentCount", Value::Int(count))];
            keys.extend(components.0);
            keys.push((text_key, Value::Data(components.1)));
            keys.push(("NSColorSpace", Value::Int(space)));
            Object::new("UIColor", keys).with_fallback("NSColor")
        })
    }

    /// UIViewLargeContentStoredProperties, which tvOS's views don't have.
    fn large_content(&self, values: &mut Vec<(&'static str, Value)>) {
        if self.platform == Platform::Ios {
            values.push(("UIViewLargeContentStoredProperties", Value::Nil));
        }
    }
}

/// The document's tint colour (`<color key="tintColor">` after its
/// resources), which Xcode gives each controller's root view and each
/// navigation bar without one of its own (five probes). Bars and
/// controllers whose tint isn't established are refused beside it.
fn document_tint<'a, 'i>(n: Node<'a, 'i>) -> Result<Option<Node<'a, 'i>>, String> {
    let root = n.document().root_element();
    let Some(tint) = root.children().find(|c| c.has_tag_name("color") && c.attribute("key") == Some("tintColor")) else { return Ok(None) };
    const UNKNOWN: [&str; 5] = ["tabBar", "toolbar", "splitViewController", "pageViewController", "searchBar"];
    if let Some(d) = root.descendants().find(|d| UNKNOWN.contains(&d.tag_name().name())) {
        return fail(format!("a document tint colour beside a <{}> is not supported by this ibtool", d.tag_name().name()));
    }
    Ok(Some(tint))
}

/// Whether a colour is an opaque white in a grey space, which some views'
/// backgrounds take as a system colour.
fn opaque_grey_white(n: Node) -> Result<bool, String> {
    let space = n.attribute("customColorSpace").or_else(|| n.attribute("colorSpace"));
    let grey = matches!(space, Some("genericGamma22GrayColorSpace" | "calibratedWhite" | "deviceWhite"));
    Ok(grey && n.attribute("white").map(number).transpose()? == Some(1.0) && n.attribute("alpha").map(number).transpose()? == Some(1.0))
}

fn color(nib: &mut Nib, n: Node) -> Result<Value, String> {
    // A system colour is Xcode's own whatever the document says: its
    // definition in the resources, or Xcode 11's fallback name and
    // components beside it.
    if let Some(name) = n.attribute("systemColor") {
        only_attributes(n, &[
            ("key", None), ("systemColor", None), ("cocoaTouchSystemColor", None), ("red", None), ("green", None),
            ("blue", None), ("white", None), ("alpha", None), ("colorSpace", None), ("customColorSpace", None),
        ])?;
        if nib.platform != Platform::Ios {
            return fail(format!("system color {name}: system colors outside iOS are not supported by this ibtool"));
        }
        let sc = &SYSTEM_COLORS.iter().find(|(c, _)| *c == name).ok_or_else(|| format!("system color {name} is not supported by this ibtool"))?.1;
        return Ok(nib.system_color(name, sc));
    }
    // Xcode 11's form names the system colour (and an older fallback), the
    // older form only a UIKit class colour; neither is declared in the
    // document. Both are archived as that system colour.
    if let Some(name) = n.attribute("xcode11CocoaTouchSystemColor").or_else(|| n.attribute("cocoaTouchSystemColor")) {
        only_attributes(n, &[("key", None), ("xcode11CocoaTouchSystemColor", None), ("cocoaTouchSystemColor", None)])?;
        if nib.platform != Platform::Ios {
            return fail(format!("system color {name}: system colors outside iOS are not supported by this ibtool"));
        }
        let sc = &SYSTEM_COLORS.iter().find(|(c, _)| *c == name).ok_or_else(|| format!("system color {name} is not supported by this ibtool"))?.1;
        return Ok(nib.system_color(name, sc));
    }
    // A colour of the asset catalog, by name: its definition in the
    // document's resources, marked as the catalog's.
    if let Some(name) = n.attribute("name") {
        only_attributes(n, &[("key", None), ("name", None)])?;
        let key = format!("color:named:{name}");
        if let Some(v) = nib.existing(&key) {
            return Ok(v);
        }
        let definition = n
            .document()
            .descendants()
            .find(|d| d.has_tag_name("namedColor") && d.attribute("name") == Some(name))
            .and_then(|d| d.children().find(|c| c.has_tag_name("color")))
            .ok_or_else(|| format!("named colour {name} without its definition in the document is not supported by this ibtool"))?;
        let space = definition.attribute("customColorSpace").or_else(|| definition.attribute("colorSpace"));
        let (count, components, text_key, text, space_code) = match space {
            Some("sRGB") => {
                only_attributes(definition, &[("red", None), ("green", None), ("blue", None), ("alpha", None), ("colorSpace", Some("custom")), ("customColorSpace", None)])?;
                let c = [number(attr(definition, "red")?)?, number(attr(definition, "green")?)?, number(attr(definition, "blue")?)?, number(attr(definition, "alpha")?)?];
                let (keys, text) = exact_color_components(&RGB_KEYS, &c);
                (4, keys, "NSRGB", text, 2)
            }
            Some("genericGamma22GrayColorSpace") => {
                only_attributes(definition, &[("white", None), ("alpha", None), ("colorSpace", Some("custom")), ("customColorSpace", None)])?;
                let c = [number(attr(definition, "white")?)?, number(attr(definition, "alpha")?)?];
                let (keys, text) = exact_color_components(&GRAY_KEYS, &c);
                (2, keys, "NSWhite", text, 4)
            }
            _ => return fail(format!("named colour {name} in this colour space is not supported by this ibtool")),
        };
        let name_value = nib.string(name);
        let mut keys = vec![("UIDynamicCatalogName", name_value), ("UIDynamicCatalogUseNibBundle", Value::Bool(true)), ("UIColorComponentCount", Value::Int(count))];
        keys.extend(components);
        keys.push((text_key, Value::Data(text)));
        keys.push(("NSColorSpace", Value::Int(space_code)));
        return Ok(nib.shared(key, || Object::new("UIColor", keys).with_fallback("NSColor")));
    }
    if !nib.color_matched {
        return unmatched_color(nib, n);
    }
    // Equal colours are one object, compared at 10 significant digits. A
    // grey is archived as given (no conversion), named when it's exactly
    // black, white or clear.
    // Calibrated white is converted to the gamma 2.2 grey: only black and
    // white are established, white coming out a float's step above 1
    // unless it's opaque (and so `whiteColor`).
    let calibrated = n.attribute("colorSpace") == Some("calibratedWhite") || n.attribute("customColorSpace") == Some("calibratedWhite");
    if calibrated {
        only_attributes(n, &[("key", None), ("white", None), ("alpha", None), ("colorSpace", None), ("customColorSpace", Some("calibratedWhite"))])?;
        let white = number(attr(n, "white")?)?;
        let alpha = significant_10(number(attr(n, "alpha")?)?);
        if white != 0.0 && white != 1.0 {
            return fail(format!("calibrated white {white} is converted by Xcode in a way not established for this ibtool"));
        }
        if white == 1.0 && alpha != 1.0 {
            let key = format!("color:calibrated-white:{alpha:?}");
            if let Some(v) = nib.existing(&key) {
                return Ok(v);
            }
            let (alpha_keys, alpha_text) = color_components(&GRAY_KEYS[1..], &[alpha]);
            let mut keys = vec![("UIColorComponentCount", Value::Int(2)), ("UIWhite", Value::Float(1.0000001))];
            keys.extend(alpha_keys);
            let mut text = b"1 ".to_vec();
            text.extend(alpha_text);
            keys.push(("NSWhite", Value::Data(text)));
            keys.push(("NSColorSpace", Value::Int(4)));
            return Ok(nib.shared(key, || Object::new("UIColor", keys).with_fallback("NSColor")));
        }
    }
    if calibrated || n.attribute("customColorSpace") == Some("genericGamma22GrayColorSpace") {
        if !calibrated {
            only_attributes(n, &[("key", None), ("white", None), ("alpha", None), ("colorSpace", Some("custom")), ("customColorSpace", None)])?;
        }
        let c = [number(attr(n, "white")?)?, number(attr(n, "alpha")?)?].map(significant_10);
        let name = match c {
            [0.0, 1.0] => Some("blackColor"),
            [1.0, 1.0] => Some("whiteColor"),
            [0.0, 0.0] => Some("clearColor"),
            _ => None,
        };
        if let Some(name) = name {
            return Ok(nib.system_color(name, &SystemColor::Gray(c)));
        }
        if let Some(v) = nib.existing(&format!("color:gray:{c:?}")) {
            return Ok(v);
        }
        let (components, text) = color_components(&GRAY_KEYS, &c);
        let mut keys = vec![("UIColorComponentCount", Value::Int(2))];
        keys.extend(components);
        keys.push(("NSWhite", Value::Data(text)));
        keys.push(("NSColorSpace", Value::Int(4)));
        return Ok(nib.shared(format!("color:gray:{c:?}"), || Object::new("UIColor", keys).with_fallback("NSColor")));
    }
    only_attributes(n, &[("key", None), ("red", None), ("green", None), ("blue", None), ("alpha", None), ("colorSpace", Some("custom")), ("customColorSpace", Some("sRGB"))])?;
    let c = [number(attr(n, "red")?)?, number(attr(n, "green")?)?, number(attr(n, "blue")?)?, number(attr(n, "alpha")?)?].map(significant_10);
    let key = format!("color:rgb:{c:?}");
    if let Some(v) = nib.existing(&key) {
        return Ok(v);
    }
    let (components, text) = color_components(&RGB_KEYS, &c);
    let mut keys = vec![("UIColorComponentCount", Value::Int(4))];
    keys.extend(components);
    keys.push(("NSRGB", Value::Data(text)));
    keys.push(("NSColorSpace", Value::Int(2)));
    Ok(nib.shared(key, || Object::new("UIColor", keys).with_fallback("NSColor")))
}

/// A colour in a document without `colorMatched`: archived as sRGB with
/// the document's components when they are calibrated RGB or white, which
/// aren't converted; colours in other spaces are, so only their neutral
/// black and white (which convert to themselves) are established.
fn unmatched_color(nib: &mut Nib, n: Node) -> Result<Value, String> {
    let space = match (n.attribute("colorSpace"), n.attribute("customColorSpace")) {
        (Some("custom"), Some(s)) => s,
        (Some(s), None) => s,
        _ => return fail("<color> without a colour space is not supported by this ibtool"),
    };
    let c = match space {
        "calibratedRGB" | "sRGB" => {
            only_attributes(n, &[("key", None), ("red", None), ("green", None), ("blue", None), ("alpha", None), ("colorSpace", None), ("customColorSpace", None)])?;
            [number(attr(n, "red")?)?, number(attr(n, "green")?)?, number(attr(n, "blue")?)?, number(attr(n, "alpha")?)?]
        }
        "calibratedWhite" | "genericGamma22GrayColorSpace" => {
            only_attributes(n, &[("key", None), ("white", None), ("alpha", None), ("colorSpace", None), ("customColorSpace", None)])?;
            // The grey becomes red, green and blue as a float.
            let w = number(attr(n, "white")?)? as f32 as f64;
            [w, w, w, number(attr(n, "alpha")?)?]
        }
        other => return fail(format!("colour space {other} without colorMatched is not supported by this ibtool")),
    };
    let calibrated = space.starts_with("calibrated");
    if !calibrated && !(c[0] == c[1] && c[1] == c[2] && (c[0] == 0.0 || c[0] == 1.0)) {
        return fail(format!("{space} colours other than black and white are converted without colorMatched, which is not supported by this ibtool"));
    }
    let c = c.map(significant_10);
    let key = format!("color:rgb:{c:?}");
    if let Some(v) = nib.existing(&key) {
        return Ok(v);
    }
    let (components, text) = color_components(&RGB_KEYS, &c);
    let mut keys = vec![("UIColorComponentCount", Value::Int(4))];
    keys.extend(components);
    keys.push(("NSRGB", Value::Data(text)));
    keys.push(("NSColorSpace", Value::Int(2)));
    Ok(nib.shared(key, || Object::new("UIColor", keys).with_fallback("NSColor")))
}

/// A view controller's navigation item: its title, also as an attributed
/// string without attributes, no items, and the large title mode.
/// A navigation item (`name`) of controller `vc_id`: the object, its bar
/// button items' names, their actions (selector, item) on the controller
/// and their segues (with the item).
struct NavigationItem<'a, 'i> {
    value: Value,
    bar_items: Vec<String>,
    actions: Vec<(String, String)>,
    segues: Vec<(Node<'a, 'i>, String)>,
}

fn navigation_item<'a, 'i>(nib: &mut Nib, n: Node<'a, 'i>, name: &str, vc_id: &str) -> Result<NavigationItem<'a, 'i>, String> {
    only_attributes(n, &[("key", Some("navigationItem")), ("title", None), ("largeTitleDisplayMode", None), ("id", None)])?;
    for c in n.children().filter(|c| c.is_element()) {
        let single = c.has_tag_name("barButtonItem") && matches!(c.attribute("key"), Some("leftBarButtonItem" | "rightBarButtonItem"));
        let list = matches!(c.tag_name().name(), "leftBarButtonItems" | "rightBarButtonItems");
        if !single && !list {
            return fail("navigation items with children other than left and right bar button items are not supported by this ibtool");
        }
    }
    let mut values = Vec::new();
    // An empty title is left out.
    if let Some(title) = n.attribute("title").filter(|t| !t.is_empty()) {
        // The attributed title holds the title's own object.
        let (first, string) = if nib.localizable {
            let Value::New(o) = nib.text(attr(n, "id")?, "title", title) else { unreachable!() };
            let name = nib.fresh();
            (Value::New(o.named(name.clone())), reference(name))
        } else {
            (nib.string(title), nib.string(title))
        };
        values.push(("UITitle", first));
        // One empty attributes dictionary serves every navigation item.
        let attributes = nib.shared("empty attributes".into(), || Object::new("NSDictionary", vec![("NSInlinedValue", Value::Bool(true))]));
        values.push(("UIAttributedTitle", Value::New(Object::new("NSAttributedString", vec![("NSString", string), ("NSAttributes", attributes)]))));
    }
    // Each side's first bar button item, then the arrays of each side's
    // items around the center item groups, the first referred to and the
    // rest defined there (a single item or a list: two probes).
    let mut bar_items = Vec::new();
    let mut actions = Vec::new();
    let mut segues = Vec::new();
    let mut arrays: Vec<(&str, Vec<Value>)> = Vec::new();
    for (key, list, single, array) in [
        ("leftBarButtonItem", "leftBarButtonItems", "UILeftBarButtonItem", "UILeftBarButtonItems"),
        ("rightBarButtonItem", "rightBarButtonItems", "UIRightBarButtonItem", "UIRightBarButtonItems"),
    ] {
        let mut items: Vec<Node> = child(n, "barButtonItem", key).into_iter().collect();
        for l in children(n, list) {
            only_children(l, &["barButtonItem"])?;
            items.extend(children(l, "barButtonItem"));
        }
        if items.len() > 1 && child(n, "barButtonItem", key).is_some() {
            return fail("a navigation item with both a bar button item and a list on one side is not supported by this ibtool");
        }
        let mut entries = Vec::new();
        for (i, b) in items.iter().enumerate() {
            let item_name = if i == 0 { format!("{name} {key}") } else { format!("{name} {key} {i}") };
            let object = Value::New(bar_button_item(nib, *b, &item_name, vc_id, &mut actions, &mut segues)?.named(item_name.clone()));
            if i == 0 {
                values.push((single, object));
                entries.push(reference(item_name.clone()));
            } else {
                entries.push(object);
            }
            bar_items.push(item_name);
        }
        if !entries.is_empty() {
            arrays.push((array, entries));
        }
    }
    if bar_items.len() > 2 {
        let mut selectors: Vec<&String> = actions.iter().map(|a| &a.0).collect();
        selectors.sort();
        if selectors.windows(2).any(|w| w[0] == w[1]) {
            return fail("bar button items in lists sharing a selector are not supported by this ibtool");
        }
    }
    let mut take = |wanted: &str| arrays.iter().position(|a| a.0 == wanted).map(|i| arrays.remove(i).1);
    if let Some(entries) = take("UILeftBarButtonItems") {
        values.push(("UILeftBarButtonItems", nib.array("NSArray", entries)));
    }
    values.push(("UICenterItemGroups", nib.empty_array()));
    if let Some(entries) = take("UIRightBarButtonItems") {
        values.push(("UIRightBarButtonItems", nib.array("NSArray", entries)));
    }
    let mode = match n.attribute("largeTitleDisplayMode") {
        None | Some("automatic") => 0,
        Some("always") => 1,
        Some("never") => 2,
        Some(other) => return fail(format!("largeTitleDisplayMode {other} is not supported by this ibtool")),
    };
    values.push(("UILargeTitleDisplayMode", Value::Int(mode)));
    Ok(NavigationItem { value: Value::New(Object::new("UINavigationItem", values).named(name)), bar_items, actions, segues })
}

/// A bar button item with a title or a system item, and its actions on the
/// controller (selector, `name`) and its segue (with `name`) pushed to
/// `actions` and `segues`.
fn bar_button_item<'a, 'i>(
    nib: &mut Nib, b: Node<'a, 'i>, name: &str, vc_id: &str, actions: &mut Vec<(String, String)>, segues: &mut Vec<(Node<'a, 'i>, String)>,
) -> Result<Object, String> {
    only_attributes(b, &[("key", None), ("title", None), ("image", None), ("style", None), ("systemItem", None), ("id", None)])?;
    only_children(b, &["connections", "color"])?;
    if children(b, "color").any(|c| c.attribute("key") != Some("tintColor")) {
        return fail("bar button item colours other than a tint colour are not supported by this ibtool");
    }
    for c in children(b, "connections") {
        only_children(c, &["action", "segue"])?;
        if children(c, "segue").count() + children(c, "action").count() > 1 {
            return fail("a bar button item with more than one action or segue is not supported by this ibtool");
        }
        segues.extend(children(c, "segue").map(|sg| (sg, name.to_string())));
        for a in children(c, "action") {
            only_attributes(a, &[("selector", None), ("destination", None), ("id", None)])?;
            if attr(a, "destination")? != vc_id {
                return fail("a bar button item's actions on anything but its controller are not supported by this ibtool");
            }
            actions.push((attr(a, "selector")?.to_string(), name.to_string()));
        }
    }
    let mut values = vec![("UIEnabled", Value::Bool(true))];
    // UIBarButtonSystemItem, as probed.
    if let Some(item) = b.attribute("systemItem") {
        // Xcode archives close as done.
        const ITEMS: [&str; 24] = [
            "done", "cancel", "edit", "save", "add", "flexibleSpace", "fixedSpace", "compose", "reply", "action", "organize", "bookmarks", "search",
            "refresh", "stop", "camera", "trash", "play", "pause", "rewind", "fastForward", "undo", "redo", "pageCurl",
        ];
        let code = match item {
            "close" => 0,
            _ => ITEMS.iter().position(|i| *i == item).ok_or_else(|| format!("bar button system item {item} is not supported by this ibtool"))? as i64,
        };
        values.push(("UIIsSystemItem", Value::Bool(true)));
        values.push(("UISystemItem", Value::Int(code)));
        // A space hides the bar's shared background.
        if code == 5 || code == 6 {
            if b.attribute("style") != Some("plain") {
                return fail("a bar button space with a style is not supported by this ibtool");
            }
            values.push(("UIHidesSharedBG", Value::Bool(true)));
        }
    }
    // No style is the legacy bordered one; plain is left out.
    match b.attribute("style") {
        None => values.push(("UIStyle", Value::Int(1))),
        Some("plain") => {}
        Some("done") => values.push(("UIStyle", Value::Int(2))),
        Some(other) => return fail(format!("bar button style {other} is not supported by this ibtool")),
    }
    // A title, an image or both, after the style; or a system item.
    match (b.attribute("title"), b.attribute("image"), b.attribute("systemItem")) {
        (title, image, None) if title.is_some() || image.is_some() => {
            if let Some(title) = title {
                values.push(("UITitle", nib.text(attr(b, "id")?, "title", title)));
            }
            if let Some(image) = image {
                values.push(("UIImage", nib.image(image)));
            }
        }
        (None, None, _) => {}
        _ => return fail("a bar button item with a system item and a title or an image is not supported by this ibtool"),
    }
    // Its tint, last (two probes).
    if let Some(c) = child(b, "color", "tintColor") {
        values.push(("UITintColor", color(nib, c)?));
    }
    Ok(Object::new("UIBarButtonItem", values))
}

/// A text style's default point size, as a minimum font size scales it
/// (ten probes).
fn text_style_size(style: &str) -> Result<f64, String> {
    Ok(match style {
        "UICTFontTextStyleBody" | "UICTFontTextStyleHeadline" => 17.0,
        "UICTFontTextStyleCallout" => 16.0,
        "UICTFontTextStyleSubhead" => 15.0,
        "UICTFontTextStyleFootnote" => 13.0,
        "UICTFontTextStyleCaption1" => 12.0,
        "UICTFontTextStyleCaption2" => 11.0,
        "UICTFontTextStyleTitle1" => 28.0,
        "UICTFontTextStyleTitle2" => 22.0,
        "UICTFontTextStyleTitle3" => 20.0,
        other => return fail(format!("a minimum font size with the {other} text style is not supported by this ibtool")),
    })
}

/// A text style's font as Xcode archives it: name, traits and the size
/// recorded for NSFont, per platform.
fn text_style(platform: Platform, style: &str) -> Result<(&'static str, i64, f64), String> {
    Ok(match (platform, style) {
        (Platform::Ios, "UICTFontTextStyleHeadline") => (".SFUI-Semibold", 2, 14.0),
        (Platform::Ios, "UICTFontTextStyleCallout") => (".SFUI-Regular", 0, 13.0),
        (Platform::Ios, "UICTFontTextStyleBody") => (".SFUI-Regular", 0, 14.0),
        (Platform::Ios, "UICTFontTextStyleSubhead") => (".SFUI-Regular", 0, 12.0),
        (Platform::Ios, "UICTFontTextStyleFootnote") => (".SFUI-Regular", 0, 12.0),
        (Platform::Ios, "UICTFontTextStyleCaption1" | "UICTFontTextStyleCaption2") => (".SFUI-Regular", 0, 11.0),
        (Platform::Ios, "UICTFontTextStyleTitle0") => (".SFUI-Regular", 0, 31.0),
        (Platform::Ios, "UICTFontTextStyleTitle1") => (".SFUI-Regular", 0, 25.0),
        (Platform::Ios, "UICTFontTextStyleTitle2") => (".SFUI-Regular", 0, 19.0),
        (Platform::Ios, "UICTFontTextStyleTitle3") => (".SFUI-Regular", 0, 17.0),
        (Platform::Tvos, "UICTFontTextStyleTitle1") => (".SFUI-Medium", 0, 76.0),
        _ => return fail(format!("the {style} text style is not supported by this ibtool here")),
    })
}

/// A controller's `<extendedEdge key="edgesForExtendedLayout">` as a
/// UIRectEdge (top 1, left 2, bottom 4, right 8), unless it is all of them,
/// the default.
fn extended_edges(vc: Node) -> Result<Option<i64>, String> {
    let Some(e) = child(vc, "extendedEdge", "edgesForExtendedLayout") else { return Ok(None) };
    only_attributes(e, &[("key", None), ("top", Some("YES")), ("left", Some("YES")), ("bottom", Some("YES")), ("right", Some("YES"))])?;
    let mask = [("top", 1), ("left", 2), ("bottom", 4), ("right", 8)].iter().filter(|(k, _)| e.attribute(*k).is_some()).map(|(_, b)| b).sum();
    Ok((mask != 15).then_some(mask))
}

/// A view's `gestureRecognizers` outlet collection to its scene's
/// recognizer, from its `<connections>`.
/// A view's connections: its outlets, to the controller or another view
/// of the nib, and its gesture recognizer outlet collection.
fn view_connections(scene: &mut Scene, n: Node, name: &str) -> Result<(), String> {
    for c in children(n, "connections") {
        only_children(c, &["outlet", "outletCollection"])?;
        for o in children(c, "outlet") {
            only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
            let (label, destination) = (attr(o, "property")?.to_string(), attr(o, "destination")?);
            if destination == scene.vc_id {
                scene.control_outlets.push((label, name.to_string()));
            } else {
                scene.view_outlets.push((label, name.to_string(), view_name(destination)));
            }
        }
        for o in children(c, "outletCollection") {
            only_attributes(o, &[("property", Some("gestureRecognizers")), ("destination", None), ("appends", Some("YES")), ("id", None)])?;
            let destination = attr(o, "destination")?;
            if !scene.gestures.iter().any(|g| g == destination) {
                return fail("an outlet collection to anything but the scene's gesture recognizers is not supported by this ibtool");
            }
            if scene.gesture_views.iter().any(|(_, gs)| gs.iter().any(|g| g == destination)) {
                return fail("a gesture recognizer of two views, or listed twice, is not supported by this ibtool");
            }
            match scene.gesture_views.iter_mut().find(|(v, _)| v == name) {
                Some((_, gs)) => gs.push(destination.to_string()),
                None => scene.gesture_views.push((name.to_string(), vec![destination.to_string()])),
            }
        }
    }
    Ok(())
}

/// The scene's gesture recognizers in their placeholders' order: the views
/// that have them from the last in the document, each view's in its order
/// (seven probes).
fn gesture_order(views: &[(String, Vec<String>)]) -> Vec<&String> {
    views.iter().rev().flat_map(|(_, gs)| gs).collect()
}

/// A scene's gesture recognizer (tap, pan or swipe), with the selectors of
/// its actions on the controller.
fn gesture_recognizer(nib: &mut Nib, r: Node, vc_id: &str) -> Result<(Object, Vec<String>), String> {
    let count = |key: &str| -> Result<Option<i64>, String> {
        match r.attribute(key) {
            None | Some("1") => Ok(None),
            Some(v) => v.parse().map(Some).map_err(|_| format!("{key}=\"{v}\" is not supported by this ibtool")),
        }
    };
    let mut values = vec![("UIGestureRecognizer.allowedTouchTypes", Value::Int(15)), ("UIGestureRecognizer.requiresExclusiveTouchType", Value::Bool(true))];
    let class = match r.tag_name().name() {
        "tapGestureRecognizer" => {
            only_attributes(r, &[("numberOfTapsRequired", None), ("numberOfTouchesRequired", None), ("id", None)])?;
            let (taps, touches) = (count("numberOfTapsRequired")?, count("numberOfTouchesRequired")?);
            if taps.is_some() && touches.is_some() {
                return fail("a tap gesture recognizer with both its taps and touches set is not supported by this ibtool");
            }
            let mut imp = Vec::new();
            if let Some(t) = taps {
                imp.push(("UITapRecognizer.numberOfTapsRequired", Value::Int(t)));
            }
            if let Some(t) = touches {
                imp.push(("UITapRecognizer.numberOfTouchesRequired", Value::Int(t)));
            }
            values.push(("UITapGestureRecognizer._imp", Value::New(Object::new("UITapRecognizer", imp))));
            "UITapGestureRecognizer"
        }
        "panGestureRecognizer" => {
            only_attributes(r, &[("minimumNumberOfTouches", None), ("maximumNumberOfTouches", None), ("id", None)])?;
            if let Some(t) = count("minimumNumberOfTouches")? {
                values.push(("UIPanGestureRecognizer.minimumNumberOfTouches", Value::Int(t)));
            }
            if let Some(t) = r.attribute("maximumNumberOfTouches") {
                let t = t.parse().map_err(|_| format!("maximumNumberOfTouches=\"{t}\" is not supported by this ibtool"))?;
                values.push(("UIPanGestureRecognizer.maximumNumberOfTouches", Value::Int(t)));
            }
            values.push(("UIPanGestureRecognizer.allowedScrollTypesMask", Value::Int(0)));
            "UIPanGestureRecognizer"
        }
        "swipeGestureRecognizer" => {
            only_attributes(r, &[("direction", None), ("id", None)])?;
            // UISwipeGestureRecognizerDirection; right, the default, is
            // left out.
            match r.attribute("direction") {
                None | Some("right") => {}
                Some("left") => values.push(("UISwipeGestureRecognizer.direction", Value::Int(2))),
                Some("up") => values.push(("UISwipeGestureRecognizer.direction", Value::Int(4))),
                Some("down") => values.push(("UISwipeGestureRecognizer.direction", Value::Int(8))),
                Some(other) => return fail(format!("swipe direction {other} is not supported by this ibtool")),
            }
            "UISwipeGestureRecognizer"
        }
        other => return fail(format!("<{other}> is not supported by this ibtool")),
    };
    let mut selectors = Vec::new();
    for c in children(r, "connections") {
        only_children(c, &["action"])?;
        for a in children(c, "action") {
            only_attributes(a, &[("selector", None), ("destination", None), ("id", None)])?;
            if attr(a, "destination")? != vc_id {
                return fail("a gesture recognizer's actions on anything but the scene's view controller are not supported by this ibtool");
            }
            selectors.push(attr(a, "selector")?.to_string());
        }
    }
    if selectors.len() > 1 {
        return fail("a gesture recognizer with more than one action is not supported by this ibtool");
    }
    let _ = nib;
    Ok((Object::new(class, values), selectors))
}

/// A view's `contentMode`: UIViewContentMode (scaleToFill, 0, is left out).
fn content_mode(name: &str) -> Result<i64, String> {
    const MODES: [&str; 13] =
        ["scaleToFill", "scaleAspectFit", "scaleAspectFill", "redraw", "center", "top", "bottom", "left", "right", "", "topRight", "bottomLeft", "bottomRight"];
    // Xcode compiles topLeft as scaleToFill: refused.
    MODES.iter().position(|m| !m.is_empty() && *m == name).map(|p| p as i64).ok_or_else(|| format!("contentMode=\"{name}\" is not supported by this ibtool"))
}

/// A system colour by name, as the table above has it.
fn system_color_named(nib: &mut Nib, name: &str) -> Result<Value, String> {
    let (_, c) = SYSTEM_COLORS.iter().find(|(n, _)| *n == name).ok_or_else(|| format!("system colour {name} is not supported by this ibtool"))?;
    Ok(nib.system_color(name, c))
}

/// A text field's or text view's `<textInputTraits>` as its values, in Xcode's order. The
/// enumerations are UIKit's; a default (0) is left out. Secure entry turns
/// autocorrection and inline prediction off.
fn text_input_traits(nib: &mut Nib, t: Node, text_view: bool) -> Result<Vec<(&'static str, Value)>, String> {
    only_attributes(t, &[
        ("key", Some("textInputTraits")), ("autocapitalizationType", None), ("autocorrectionType", None), ("spellCheckingType", None),
        ("keyboardType", None), ("keyboardAppearance", None), ("returnKeyType", None), ("enablesReturnKeyAutomatically", Some("YES")),
        ("secureTextEntry", Some("YES")), ("textContentType", None), ("smartDashesType", None), ("smartInsertDeleteType", None),
        ("smartQuotesType", None),
    ])?;
    if t.has_children() {
        return fail("text input traits with children are not supported by this ibtool");
    }
    let lookup = |key: &str, names: &[&str]| -> Result<i64, String> {
        match t.attribute(key) {
            None => Ok(0),
            Some(v) => names.iter().position(|n| *n == v).map(|i| i as i64).ok_or_else(|| format!("{key}=\"{v}\" is not supported by this ibtool")),
        }
    };
    let no_yes = ["default", "no", "yes"];
    let secure = t.attribute("secureTextEntry").is_some();
    let mut values = Vec::new();
    // A text view's default is sentences: it leaves that out and writes
    // none.
    let capitalization = lookup("autocapitalizationType", &["none", "words", "sentences", "allCharacters"])?;
    if (text_view && capitalization != 2) || (!text_view && capitalization != 0) {
        values.push(("UIAutocapitalizationType", Value::Int(capitalization)));
    }
    let mut push = |key: &'static str, v: i64| {
        if v != 0 {
            values.push((key, Value::Int(v)));
        }
    };
    push("UIAutocorrectionType", if secure { 1 } else { lookup("autocorrectionType", &no_yes)? });
    push("UISpellCheckingType", lookup("spellCheckingType", &no_yes)?);
    push("UIKeyboardAppearance", match t.attribute("keyboardAppearance") {
        Some("alert") => 1,
        _ => lookup("keyboardAppearance", &["default", "dark", "light"])?,
    });
    push("UIKeyboardType", lookup("keyboardType", &[
        "default", "alphabet", "numbersAndPunctuation", "URL", "numberPad", "phonePad", "namePhonePad", "emailAddress", "decimalPad", "twitter",
        "webSearch",
    ])?);
    push("UIReturnKeyType", lookup("returnKeyType", &[
        "default", "go", "google", "join", "next", "route", "search", "send", "yahoo", "done", "emergencyCall", "continue",
    ])?);
    if t.attribute("enablesReturnKeyAutomatically").is_some() {
        values.push(("UIEnablesReturnKeyAutomatically", Value::Bool(true)));
    }
    if secure {
        values.push(("UISecureTextEntry", Value::Bool(true)));
    }
    if let Some(content) = t.attribute("textContentType") {
        values.push(("UITextContentType", nib.string(content)));
    }
    let mut push = |key: &'static str, v: i64| {
        if v != 0 {
            values.push((key, Value::Int(v)));
        }
    };
    push("UITextSmartInsertDeleteType", lookup("smartInsertDeleteType", &no_yes)?);
    push("UITextSmartQuotesType", lookup("smartQuotesType", &no_yes)?);
    push("UITextSmartDashesType", lookup("smartDashesType", &no_yes)?);
    if secure {
        values.push(("UITextInlinePredictionType", Value::Int(1)));
    }
    Ok(values)
}

/// The class a custom class names at run time: a Swift class in a module is
/// named as the Swift runtime names it (`_TtC`, then the module and class,
/// each after its length in bytes; the standard library is `s`). A module
/// the target provides is ibtool's `--module`, when there is one.
fn runtime_class_name(n: Node, module: Option<&str>, default: &str) -> Result<String, String> {
    let Some(class) = n.attribute("customClass") else {
        if n.attribute("customModule").is_some() || n.attribute("customModuleProvider").is_some_and(|p| !p.is_empty()) {
            return fail("a custom module without a custom class is not supported by this ibtool");
        }
        return Ok(default.into());
    };
    let module = match (n.attribute("customModuleProvider"), module) {
        (Some("target"), Some(m)) => Some(m),
        (None | Some("" | "target"), _) => n.attribute("customModule"),
        (Some(p), _) => return fail(format!("customModuleProvider={p:?} is not supported by this ibtool")),
    };
    Ok(match module {
        None | Some("") => class.to_string(),
        Some("Swift") => format!("_TtCs{}{class}", class.len()),
        Some(m) if m.starts_with("__") => return fail(format!("custom module {m} is not supported by this ibtool")),
        Some(m) => format!("_TtC{}{m}{}{class}", m.len(), class.len()),
    })
}

/// A label's minimum font size and scale factor, when it adjusts its font
/// size to fit (the default) and has either. A scale factor's size is
/// rounded to the nearest quarter point, halves up; a size's factor is the
/// size over the font's.
fn minimum_font_size(n: Node) -> Result<Option<(f32, f32)>, String> {
    if n.attribute("adjustsFontSizeToFit").is_some() {
        return Ok(None);
    }
    let (factor, size) = (n.attribute("minimumScaleFactor"), n.attribute("minimumFontSize"));
    if factor.is_none() && size.map_or(true, |s| number(s) == Ok(0.0)) {
        return Ok(None);
    }
    // A text style's size is its default Dynamic Type size.
    let description = child(n, "fontDescription", "fontDescription");
    let font_size = match (description.and_then(|f| f.attribute("pointSize")), description.and_then(|f| f.attribute("style"))) {
        (Some(size), _) => number(size)?,
        (None, Some(style)) => text_style_size(style)?,
        (None, None) => return fail("a minimum font size without the font's point size is not supported by this ibtool"),
    };
    Ok(Some(match (factor, size) {
        (Some(factor), _) => {
            let factor = number(factor)? as f32;
            (((factor as f64 * font_size * 4.0).round() / 4.0) as f32, factor)
        }
        (None, Some(size)) => {
            let size = number(size)? as f32;
            (size, (size as f64 / font_size) as f32)
        }
        (None, None) => unreachable!(),
    }))
}

/// A font: `<fontDescription>`, or a control's default system font. Equal
/// fonts are one object.
fn font(nib: &mut Nib, n: Option<Node>, default_size: Option<f64>) -> Result<Value, String> {
    let (style, name, size, face) = match n {
        Some(n) => {
            if let Some(style) = n.attribute("style") {
                only_attributes(n, &[("key", None), ("style", None)])?;
                (Some(style), None, 0.0, "regular")
            } else if let Some(name) = n.attribute("name") {
                only_attributes(n, &[("key", None), ("name", None), ("family", None), ("pointSize", None)])?;
                (None, Some(name), number(attr(n, "pointSize")?)?, "regular")
            } else {
                only_attributes(n, &[("key", None), ("type", None), ("weight", None), ("pointSize", None), ("size", Some("button"))])?;
                // The bold system font is the bold weight.
                let face = match (attr(n, "type")?, n.attribute("weight")) {
                    ("system", weight) => weight.unwrap_or("regular"),
                    ("boldSystem", None) => "bold",
                    ("italicSystem", None) => "italic",
                    (kind, _) => return fail(format!("a {kind} font with a weight is not supported by this ibtool")),
                };
                // No size is 12 points (two probes); the button size is 18
                // (one probe).
                let point_size = match (n.attribute("size"), n.attribute("pointSize")) {
                    (Some(_), None) => 18.0,
                    (None, _) => number(attr(n, "pointSize")?)?,
                    (Some(_), Some(_)) => return fail("a font with both a size and a point size is not supported by this ibtool"),
                };
                let size = match (point_size, face) {
                    (0.0, "regular" | "bold") => 12.0,
                    (0.0, _) => return fail(format!("a {face} system font without a size is not supported by this ibtool")),
                    (size, _) => size,
                };
                (None, None, size, face)
            }
        }
        None => (None, None, default_size.ok_or("a font is required")?, "regular"),
    };
    // Xcode archives a font it finds under its own name.
    let name = name.map(|name| fonts::ALIASES.iter().find(|(a, _)| *a == name).map_or(name, |(_, to)| to));
    let key = format!("font:{style:?}:{name:?}:{size}:{face}");
    if let Some(v) = nib.existing(&key) {
        return Ok(v);
    }
    let mut values = Vec::new();
    let (font_name, traits, ns_size, system) = match (style, name) {
        (Some(style), _) => {
            let (name, traits, ns_size) = text_style(nib.platform, style)?;
            let name_value = nib.string(name);
            values.push(("UIFontName", name_value));
            values.push(("UIIBTextStyle", nib.string(style)));
            (name, traits, ns_size, true)
        }
        (None, Some(name)) => {
            // One Xcode doesn't find is archived differently (see ibtool.md).
            let traits = match fonts::FONTS.binary_search_by(|(n, _)| n.cmp(&name)) {
                Ok(i) => fonts::FONTS[i].1,
                Err(_) => return fail(format!("font {name} is not one Xcode's ibtool finds for iOS, which is not supported by this ibtool")),
            };
            values.push(("UIFontName", nib.string(name)));
            let name_key = nib.string("NSFontNameAttribute");
            let name_value = nib.string(name);
            let size_key = nib.string("NSFontSizeAttribute");
            let size_value = nib.double_number(size);
            let attributes = nib.array("NSDictionary", vec![name_key, name_value, size_key, size_value]);
            values.push(("UIFontDescriptor", Value::New(Object::new("UIFontDescriptor", vec![
                ("UIFontDescriptorAttributes", attributes),
                ("UIFontDescriptorOptions", Value::Int(0x8000_0000)),
            ]))));
            (name, traits, size, false)
        }
        (None, None) => {
            // The system font's faces: its name, CoreText usage and traits.
            let (name, usage, traits) = match face {
                "ultraLight" => (".SFUI-Ultralight", "CTFontUltraLightUsage", 16),
                "thin" => (".SFUI-Thin", "CTFontThinUsage", 4),
                "light" => (".SFUI-Light", "CTFontLightUsage", 8),
                "regular" => (".SFUI-Regular", "CTFontRegularUsage", 0),
                "medium" => (".SFUI-Medium", "CTFontMediumUsage", 0),
                "semibold" => (".SFUI-Semibold", "CTFontDemiUsage", 2),
                "bold" => (".SFUI-Bold", "CTFontEmphasizedUsage", 2),
                "heavy" => (".SFUI-Heavy", "CTFontHeavyUsage", 2),
                "black" => (".SFUI-Black", "CTFontBlackUsage", 2),
                "italic" => (".SFUI-RegularItalic", "CTFontObliqueUsage", 1),
                other => return fail(format!("system font weight {other} is not supported by this ibtool")),
            };
            values.push(("UIFontName", nib.string(name)));
            let mut attributes = Vec::new();
            // Italic is a symbolic trait too.
            if face == "italic" {
                let traits_key = nib.string("NSCTFontTraitsAttribute");
                let symbolic_key = nib.string("NSCTFontSymbolicTrait");
                let symbolic = nib.int_number(1);
                attributes.extend([traits_key, nib.array("NSDictionary", vec![symbolic_key, symbolic])]);
            }
            attributes.extend([nib.string("NSCTFontUIUsageAttribute"), nib.string(usage)]);
            // Bold overrides the emphasized usage, in a mutable dictionary.
            if face == "bold" {
                attributes.extend([nib.string("NSCTFontUIUsageOverrideAttribute"), nib.string("CTFontBoldUsage")]);
            }
            let size_key = nib.string("NSFontSizeAttribute");
            let size_value = nib.double_number(size);
            attributes.extend([size_key, size_value]);
            let class = if face == "bold" { "NSMutableDictionary" } else { "NSDictionary" };
            let attributes = nib.array(class, attributes);
            values.push(("UIFontDescriptor", Value::New(Object::new("UIFontDescriptor", vec![
                ("UIFontDescriptorAttributes", attributes),
                ("UIFontDescriptorOptions", Value::Int(2147517444)),
            ]))));
            (name, traits, size, true)
        }
    };
    values.push(("UIFontPointSize", Value::Double(size)));
    values.push(("UIFontTextStyleForScaling", Value::Nil));
    values.push(("UIFontPointSizeForScaling", Value::Double(0.0)));
    values.push(("UIFontMaximumPointSizeAfterScaling", Value::Double(0.0)));
    values.push(("UIFontTraits", Value::Int(traits)));
    values.push(("UISystemFont", Value::Bool(system)));
    values.push(("NSName", nib.string(font_name)));
    values.push(("NSSize", Value::Double(ns_size)));
    Ok(nib.shared(key, || Object::new("UIFont", values).with_fallback("NSFont")))
}

struct Frame {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn frame(n: Node) -> Result<Frame, String> {
    let r = child(n, "rect", "frame").ok_or_else(|| format!("<{}> without a frame", n.tag_name().name()))?;
    Ok(Frame { x: number(attr(r, "x")?)?, y: number(attr(r, "y")?)?, w: number(attr(r, "width")?)?, h: number(attr(r, "height")?)? })
}

/// A subview's frame. Without one, or with an all-zero one, a label, button,
/// image view, segmented control or plain view is 1000x1000 at the origin,
/// or has no size if its frame is fixed (image views and segmented controls:
/// four probes).
fn subview_frame(n: Node) -> Result<Frame, String> {
    if let Some(f) = child(n, "rect", "frame").map(|_| frame(n)).transpose()? {
        if f.x != 0.0 || f.y != 0.0 || f.w != 0.0 || f.h != 0.0 {
            return Ok(f);
        }
    }
    match n.tag_name().name() {
        "label" | "view" | "button" | "imageView" | "segmentedControl" if n.attribute("fixedFrame").is_some() => Ok(Frame { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }),
        "label" | "view" | "button" | "imageView" | "segmentedControl" => Ok(Frame { x: 0.0, y: 0.0, w: 1000.0, h: 1000.0 }),
        other => fail(format!("<{other}> without a frame")),
    }
}

/// An object's user-defined runtime attributes (`name` the object's), as
/// key-value pairs set when the nib loads: numbers, booleans (as 0 or 1),
/// strings, colours and sizes.
fn runtime_attributes(nib: &mut Nib, n: Node, name: &str) -> Result<Vec<Value>, String> {
    let mut pairs = Vec::new();
    for list in children(n, "userDefinedRuntimeAttributes") {
        only_children(list, &["userDefinedRuntimeAttribute"])?;
        for a in children(list, "userDefinedRuntimeAttribute") {
            let key_path = attr(a, "keyPath")?;
            let value = match attr(a, "type")? {
                "number" => {
                    only_attributes(a, &[("type", None), ("keyPath", None)])?;
                    match (child(a, "integer", "value"), child(a, "real", "value")) {
                        (Some(i), None) => {
                            let v = attr(i, "value")?;
                            nib.int_number(v.parse().map_err(|_| format!("not a number: {v}"))?)
                        }
                        (None, Some(r)) => nib.double_number(number(attr(r, "value")?)?),
                        _ => return fail("a runtime attribute's number without one integer or real is not supported by this ibtool"),
                    }
                }
                "boolean" => {
                    only_attributes(a, &[("type", None), ("keyPath", None), ("value", None)])?;
                    match attr(a, "value")? {
                        "YES" => nib.int_number(1),
                        "NO" => nib.int_number(0),
                        other => return fail(format!("a runtime attribute's boolean {other} is not supported by this ibtool")),
                    }
                }
                "string" => {
                    only_attributes(a, &[("type", None), ("keyPath", None), ("value", None)])?;
                    nib.string(attr(a, "value")?)
                }
                "color" => {
                    only_attributes(a, &[("type", None), ("keyPath", None)])?;
                    color(nib, child(a, "color", "value").ok_or("a runtime attribute's colour without a value")?)?
                }
                "size" => {
                    only_attributes(a, &[("type", None), ("keyPath", None)])?;
                    let s = child(a, "size", "value").ok_or("a runtime attribute's size without a value")?;
                    let size = nib.string(&format!("{{{}, {}}}", number(attr(s, "width")?)?, number(attr(s, "height")?)?));
                    Value::New(Object::new("NSValue", vec![("NS.special", Value::Int(2)), ("NS.sizeval", size)]))
                }
                other => return fail(format!("a runtime attribute of type {other} is not supported by this ibtool")),
            };
            let key_path = nib.string(key_path);
            pairs.push(Value::New(Object::new("UINibKeyValuePair", vec![("UIObject", reference(name)), ("UIValue", value), ("UIKeyPath", key_path)])));
        }
    }
    Ok(pairs)
}

/// A switch, slider or activity indicator: a non-opaque view laid out as
/// usual, with its own values after its large content properties, and
/// its value-changed actions on the controller.
fn simple_control(nib: &mut Nib, scene: &mut Scene, n: Node, name: &str, f: &Frame, mut values: Vec<(&'static str, Value)>) -> Result<Object, String> {
    let tag = n.tag_name().name();
    let mut allowed = vec![
        ("opaque", Some("NO")), ("contentMode", Some("scaleToFill")), ("hidden", Some("YES")), ("tag", None), ("alpha", None),
        ("fixedFrame", Some("YES")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
        ("horizontalHuggingPriority", None), ("verticalHuggingPriority", None), ("horizontalCompressionResistancePriority", None),
        ("verticalCompressionResistancePriority", None),
    ];
    let mut children_allowed = vec!["rect", "autoresizingMask", "constraints", "connections", "accessibility"];
    match tag {
        "switch" => {
            allowed.extend([
                ("contentHorizontalAlignment", Some("center")),
                ("contentVerticalAlignment", Some("center")), ("on", Some("YES")), ("enabled", Some("NO")),
            ]);
            children_allowed.push("color");
        }
        "slider" => {
            allowed.extend([
                ("contentHorizontalAlignment", Some("center")), ("contentVerticalAlignment", Some("center")), ("value", None), ("minValue", None),
                ("maxValue", None), ("continuous", Some("NO")), ("enabled", Some("NO")),
            ]);
            children_allowed.push("color");
        }
        "stepper" => {
            allowed.extend([
                ("contentHorizontalAlignment", Some("center")), ("contentVerticalAlignment", Some("center")), ("value", None),
                ("minimumValue", None), ("maximumValue", None), ("stepValue", None), ("continuous", Some("NO")), ("autorepeat", Some("NO")),
                ("wraps", Some("YES")), ("enabled", Some("NO")),
            ]);
            children_allowed.push("color");
        }
        _ => {
            allowed.extend([
                ("hidesWhenStopped", Some("YES")),
                ("animating", Some("YES")), ("style", None),
            ]);
            children_allowed.push("color");
        }
    }
    only_attributes(n, &allowed)?;
    only_children(n, &children_allowed)?;
    // An accessibility configuration as any view's (two probes).
    accessibility(nib, scene, n, name)?;
    for c in children(n, "connections") {
        only_children(c, &["action"])?;
        if tag == "activityIndicatorView" && c.has_children() {
            return fail("an activity indicator's actions are not supported by this ibtool");
        }
        // Any of UIControlEvents (one probe for a slider's).
        for a in children(c, "action") {
            only_attributes(a, &[("selector", None), ("destination", None), ("eventType", None), ("id", None)])?;
            let target = action_target(scene, a)?;
            scene.actions.push((attr(a, "selector")?.to_string(), name.to_string(), control_event(attr(a, "eventType")?)?, target.into()));
        }
    }
    let colors: Vec<&str> = children(n, "color").map(|c| c.attribute("key").unwrap_or("")).collect();
    let known: &[&str] = match tag {
        "switch" => &["onTintColor"],
        "slider" => &["minimumTrackTintColor", "maximumTrackTintColor", "thumbTintColor"],
        "stepper" => &["backgroundColor", "tintColor"],
        _ => &["color"],
    };
    if let Some(other) = colors.iter().find(|c| !known.contains(c)) {
        return fail(format!("a {tag}'s {other} colour is not supported by this ibtool"));
    }
    // An activity indicator hidden when stopped and not animating is hidden.
    let hidden_when_stopped = tag == "activityIndicatorView" && n.attribute("hidesWhenStopped").is_some() && n.attribute("animating").is_none();
    if hidden_when_stopped && n.attribute("hidden").is_some() {
        return fail("a hidden activity indicator hidden when stopped is not supported by this ibtool");
    }

    if tag == "stepper" {
        if let Some(c) = child(n, "color", "backgroundColor") {
            values.push(("UIBackgroundColor", color(nib, c)?));
        }
    }
    values.push(("UIDeepDrawRect", Value::Bool(true)));
    if hidden_when_stopped {
        values.push(("UIHidden", Value::Bool(true)));
    }
    hidden_and_tag(&mut values, n)?;
    values.push(("UIAutoresizeSubviews", Value::Bool(true)));
    values.push(("UIAutoresizingMask", Value::Int(36)));
    alpha(&mut values, n)?;
    if tag == "stepper" {
        if let Some(c) = child(n, "color", "tintColor") {
            values.push(("UITintColor", color(nib, c)?));
        }
    }
    if does_not_translate(scene, n) {
        values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
    }
    if let Some(own) = own_constraints(nib, scene, n, name)? {
        values.push(("UIViewAutolayoutConstraints", own));
    }
    // Hugging (each 250 unless given) is written, but for a switch, stepper
    // or activity indicator not when it is 750 both ways. Then compression
    // resistance, each 750 unless given, if either is.
    let priority = |k: &str| n.attribute(k).unwrap_or("250");
    let (h, v) = (priority("horizontalHuggingPriority"), priority("verticalHuggingPriority"));
    if tag == "slider" || (h, v) != ("750", "750") {
        values.push(("UIViewContentHuggingPriority", nib.string(&format!("{{{h}, {v}}}"))));
    }
    let resistance = (n.attribute("horizontalCompressionResistancePriority"), n.attribute("verticalCompressionResistancePriority"));
    if resistance != (None, None) {
        let (h, v) = (resistance.0.unwrap_or("750"), resistance.1.unwrap_or("750"));
        values.push(("UIViewContentCompressionResistancePriority", nib.string(&format!("{{{h}, {v}}}"))));
    }
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    let class = match tag {
        "switch" => {
            if n.attribute("enabled").is_some() {
                values.push(("UIDisabled", Value::Bool(true)));
            } else {
                values.push(("UISwitchEnabled", Value::Bool(true)));
            }
            if n.attribute("on").is_some() {
                values.push(("UISwitchOn", Value::Bool(true)));
            }
            if let Some(c) = child(n, "color", "onTintColor") {
                values.push(("UISwitchOnTintColor", color(nib, c)?));
            }
            // Its fixed size is set back to its frame as the nib loads.
            let rect = nib.string(&format!("{{{{{}, {}}}, {{{}, {}}}}}", f.x, f.y, f.w, f.h));
            let value = Value::New(Object::new("NSValue", vec![("NS.special", Value::Int(3)), ("NS.rectval", rect)]));
            let key_path = nib.string("frame");
            scene.pairs.push((name.to_string(), Value::New(Object::new("UINibKeyValuePair", vec![("UIObject", reference(name)), ("UIValue", value), ("UIKeyPath", key_path)]))));
            scene.switch_frames.push(name.to_string());
            "UISwitch"
        }
        "slider" => {
            if n.attribute("enabled").is_some() {
                values.push(("UIDisabled", Value::Bool(true)));
            }
            let value = |k: &str, default: f64| n.attribute(k).map(number).transpose().map(|v| v.unwrap_or(default));
            // A value of 0, or none, is left out (two probes).
            let current = value("value", 0.0)?;
            if current != 0.0 {
                values.push(("UIValue", Value::Float(current as f32)));
            }
            let (min, max) = (value("minValue", 0.0)?, value("maxValue", 1.0)?);
            if min != 0.0 {
                values.push(("UIMinValue", Value::Float(min as f32)));
            }
            if max != 1.0 {
                values.push(("UIMaxValue", Value::Float(max as f32)));
            }
            if n.attribute("continuous").is_some() {
                values.push(("UIContinuous", Value::Bool(false)));
            }
            for (key, value_key) in [("minimumTrackTintColor", "UIMinimumTintColor"), ("maximumTrackTintColor", "UIMaximumTintColor"), ("thumbTintColor", "UIThumbTintColor")] {
                if let Some(c) = child(n, "color", key) {
                    values.push((value_key, color(nib, c)?));
                }
            }
            values.push(("UIBehavioralStyle", Value::Int(0)));
            "UISlider"
        }
        "stepper" => {
            if n.attribute("enabled").is_some() {
                values.push(("UIDisabled", Value::Bool(true)));
            }
            // Each written when it isn't UIStepper's default; a maximum the
            // document leaves out is 0, not UIStepper's 100.
            for (k, key, default) in [("value", "UIValue", 0.0), ("minimumValue", "UIMinimumValue", 0.0), ("maximumValue", "UIMaximumValue", 100.0), ("stepValue", "UIStepValue", 1.0)] {
                let missing = if k == "stepValue" { 1.0 } else { 0.0 };
                let v = n.attribute(k).map(number).transpose()?.unwrap_or(missing);
                if v != default {
                    values.push((key, Value::Double(v)));
                }
            }
            for (k, key) in [("continuous", "UIContinuous"), ("autorepeat", "UIAutorepeat")] {
                if n.attribute(k).is_some() {
                    values.push((key, Value::Bool(false)));
                }
            }
            if n.attribute("wraps").is_some() {
                values.push(("UIWraps", Value::Bool(true)));
            }
            "UIStepper"
        }
        _ => {
            // The modern style and the legacy one.
            let (modern, legacy) = match attr(n, "style")? {
                "medium" => (100, 1),
                "large" => (101, 0),
                "whiteLarge" => (0, 0),
                "white" => (1, 1),
                "gray" => (2, 2),
                other => return fail(format!("activity indicator style {other} is not supported by this ibtool")),
            };
            values.push(("UIActivityIndicatorViewStyle-Modern", Value::Int(modern)));
            values.push(("UIActivityIndicatorViewStyle", Value::Int(legacy)));
            if n.attribute("animating").is_some() {
                values.push(("UIAnimating", Value::Bool(true)));
            }
            if n.attribute("hidesWhenStopped").is_none() {
                values.push(("UIHidesWhenStopped", Value::Bool(false)));
            }
            // A colour is its tint too; a modern style's tint is otherwise
            // the secondary label colour.
            match child(n, "color", "color") {
                Some(c) => {
                    let c = color(nib, c)?;
                    let Value::New(o) = c else { return fail("a shared activity indicator colour is not supported by this ibtool") };
                    let colour_name = o.name.clone().unwrap_or_else(|| nib.fresh());
                    values.push(("UIColor", Value::New(o.named(colour_name.clone()))));
                    values.push(("UITintColor", reference(colour_name)));
                    values.push(("UIDisableUpdateColorOnTraitCollectionChange", Value::Bool(true)));
                }
                None => {
                    values.push(("UIColor", Value::Nil));
                    if modern >= 100 {
                        values.push(("UITintColor", system_color_named(nib, "secondaryLabelColor")?));
                    }
                }
            }
            "UIActivityIndicatorView"
        }
    };
    Ok(Object::new(class, values))
}

/// The layout margins a view's document gives it, top, left (or leading),
/// bottom and right (or trailing), and whether they are directional.
fn layout_margins(n: Node) -> Result<Option<([f64; 4], bool)>, String> {
    Ok(match (child(n, "edgeInsets", "layoutMargins"), child(n, "directionalEdgeInsets", "directionalLayoutMargins")) {
        (None, None) => None,
        (Some(m), None) => Some((insets(m, "left", "right")?, false)),
        (None, Some(m)) => Some((insets(m, "leading", "trailing")?, true)),
        _ => return fail("a view with both layout margins and directional ones is not supported by this ibtool"),
    })
}

/// A view's layout margins (top, left, bottom, right, or leading and
/// trailing), whether they follow the readable width, whether it preserves
/// its superview's and whether it insets them from the safe area, after its
/// constraints (and a root view's layout guides).
fn margins(values: &mut Vec<(&'static str, Value)>, n: Node) -> Result<(), String> {
    // The margins, whether they follow the readable width, then whether
    // they are directional (two probes and a corpus storyboard).
    let margins = layout_margins(n)?;
    if let Some((m, _)) = margins {
        values.push(("UIViewLayoutMargins", doubles(&m)));
    }
    if children(n, "edgeInsets").chain(children(n, "directionalEdgeInsets")).count() > 1 {
        return fail("a view's other edge insets are not supported by this ibtool");
    }
    if n.attribute("layoutMarginsFollowReadableWidth").is_some() {
        values.push(("UIViewLayoutMarginsFollowReadableWidth", Value::Bool(true)));
    }
    if margins.is_some_and(|(_, directional)| directional) {
        values.push(("UIViewLayoutMarginsAreDirectional", Value::Bool(true)));
    }
    if n.attribute("preservesSuperviewLayoutMargins").is_some() {
        values.push(("UIViewPreservesSuperviewMargins", Value::Bool(true)));
        values.push(("UIViewPreservesSpecificSuperviewMargins", Value::Int(15)));
    }
    // Turned on, the default, it changes nothing (one probe).
    match n.attribute("insetsLayoutMarginsFromSafeArea") {
        None | Some("YES") => {}
        Some("NO") => values.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0))),
        Some(other) => return fail(format!("insetsLayoutMarginsFromSafeArea=\"{other}\" is not supported by this ibtool")),
    }
    Ok(())
}

/// A view's `UIRestorationIdentifier`, just before
/// `UIViewSemanticContentAttribute`.
fn restoration_identifier(nib: &mut Nib, values: &mut Vec<(&'static str, Value)>, n: Node) {
    if let Some(r) = n.attribute("restorationIdentifier") {
        values.push(("UIRestorationIdentifier", nib.string(r)));
    }
}

/// A view's `UIHidden` and `UITag`, after `UIOpaque`.
fn hidden_and_tag(values: &mut Vec<(&'static str, Value)>, n: Node) -> Result<(), String> {
    if n.attribute("hidden").is_some() {
        values.push(("UIHidden", Value::Bool(true)));
    }
    if let Some(tag) = n.attribute("tag") {
        values.push(("UITag", Value::Int(tag.parse().map_err(|_| format!("not a number: {tag}"))?)));
    }
    Ok(())
}

/// A view's `UIAlpha`, after its autoresizing mask.
fn alpha(values: &mut Vec<(&'static str, Value)>, n: Node) -> Result<(), String> {
    if let Some(alpha) = n.attribute("alpha") {
        values.push(("UIAlpha", Value::Float(number(alpha)? as f32)));
    }
    Ok(())
}

/// A view's `<accessibility>`, as its nib's configuration of the view
/// (`name`): hint, identifier, label, traits (UIAccessibilityTraits) and
/// whether it is an element, those given.
fn accessibility(nib: &mut Nib, scene: &mut Scene, n: Node, name: &str) -> Result<(), String> {
    let Some(a) = child(n, "accessibility", "accessibilityConfiguration") else { return Ok(()) };
    if children(n, "accessibility").count() > 1 {
        return fail("two accessibility configurations are not supported by this ibtool");
    }
    only_attributes(a, &[("key", None), ("hint", None), ("identifier", None), ("label", None)])?;
    only_children(a, &["accessibilityTraits", "bool"])?;
    // A hint and label are localizable (one probe); the identifier isn't.
    let id = attr(n, "id")?;
    let mut values = vec![("UIAccessibileObject", reference(name))];
    for (attribute, key) in [("hint", "UIAccessibilityHint"), ("identifier", "UIAccessibilityIdentifier"), ("label", "UIAccessibilityLabel")] {
        if let Some(v) = a.attribute(attribute) {
            let value = match attribute {
                "identifier" => nib.string(v),
                _ => nib.text(id, &format!("accessibility{}", if attribute == "hint" { "Hint" } else { "Label" }), v),
            };
            values.push((key, value));
        }
    }
    if let Some(t) = child(a, "accessibilityTraits", "traits") {
        let mut traits = 0;
        for at in t.attributes().filter(|at| at.name() != "key") {
            if at.value() != "YES" {
                return fail(format!("accessibility trait {}=\"{}\" is not supported by this ibtool", at.name(), at.value()));
            }
            traits |= match at.name() {
                "none" => 0,
                "button" => 1,
                "link" => 2,
                "image" => 4,
                "staticText" => 64,
                "notEnabled" => 256,
                "header" => 65536,
                other => return fail(format!("the accessibility trait {other} is not supported by this ibtool")),
            };
        }
        values.push(("UIAccessibilityTraits", nib.int_number(traits)));
    }
    for b in children(a, "bool") {
        only_attributes(b, &[("key", Some("isElement")), ("value", None)])?;
        let value = match attr(b, "value")? {
            "YES" => 1,
            "NO" => 0,
            other => return fail(format!("isElement=\"{other}\" is not supported by this ibtool")),
        };
        values.push(("UIIsAccessibilityElement", nib.int_number(value)));
    }
    scene.accessibility.push((name.to_string(), Value::New(Object::new("UIRuntimeAccessibilityConfiguration", values))));
    Ok(())
}

/// An `<edgeInsets>` or `<directionalEdgeInsets>` as top, `left`, bottom,
/// `right`.
fn insets(n: Node, left: &str, right: &str) -> Result<[f64; 4], String> {
    only_attributes(n, &[("key", None), ("top", None), (left, None), ("bottom", None), (right, None)])?;
    Ok([number(attr(n, "top")?)?, number(attr(n, left)?)?, number(attr(n, "bottom")?)?, number(attr(n, right)?)?])
}

/// Bounds and centre, each left out when zero.
fn geometry(values: &mut Vec<(&'static str, Value)>, f: &Frame) {
    if f.w != 0.0 || f.h != 0.0 {
        values.push(("UIBounds", doubles(&[0.0, 0.0, f.w, f.h])));
    }
    let center = [f.x + f.w / 2.0, f.y + f.h / 2.0];
    if center != [0.0, 0.0] {
        values.push(("UICenter", doubles(&center)));
    }
}

fn view_name(id: &str) -> String {
    format!("view {id}")
}

/// What compiling a scene's view collects besides the view itself.
struct Scene<'a> {
    vc_id: &'a str,
    /// ibtool's `--module`.
    module: Option<&'a str>,
    /// Subviews in depth-first preorder, with their superview's name (None
    /// for the root view's), for the nib's objects.
    subviews: Vec<(Option<String>, String)>,
    /// The superview of the subviews being compiled.
    parent: Option<String>,
    /// Control actions: (selector, source, event mask, the placeholder they
    /// target), in depth-first order.
    actions: Vec<(String, String, i64, String)>,
    /// The ids of the segues controls trigger, in depth-first order; each is
    /// a `perform:` action on a placeholder of its own.
    segues: Vec<String>,
    /// The order the controller's nib lists them in, as indices into
    /// `segues`, when not theirs: a static table's cells' segues.
    segue_listing: Option<Vec<usize>>,
    /// Container views' `containerView` outlets: (the placeholder of the
    /// view's embed segue, the view).
    containers: Vec<(String, String)>,
    /// Controls' outlets to the controller: (label, the control).
    control_outlets: Vec<(String, String)>,
    /// The scene's gesture recognizers' ids, and the views whose
    /// `gestureRecognizers` they are, with each view's recognizers.
    gestures: Vec<String>,
    gesture_views: Vec<(String, Vec<String>)>,
    /// Views' outlets to other views: (label, the view, the destination).
    view_outlets: Vec<(String, String, String)>,
    /// Views' user-defined runtime attributes, as the nib's key-value
    /// pairs, and their accessibility configurations, each with the view's
    /// name: the nib lists them in the order of its objects.
    pairs: Vec<(String, Value)>,
    /// Visual effect views and their content views, whose subviews the
    /// objects list before their constraints.
    effect_contents: Vec<String>,
    accessibility: Vec<(String, Value)>,
    /// The switches whose frames are among the pairs.
    switch_frames: Vec<String>,
    /// The constraints subviews hold, with the holder's name, for the nib's
    /// objects.
    own_constraints: Vec<(String, String)>,
    /// Ids of the views some constraint is on.
    constrained: Vec<&'a str>,
    /// Ids of the legacy top and bottom layout guides.
    guides: Vec<(&'a str, &'static str)>,
}

impl<'a> Scene<'a> {
    /// A scene compiling `view`'s subviews into `parent` (None for a root
    /// view's).
    fn new(vc_id: &'a str, module: Option<&'a str>, view: Node<'a, '_>, parent: Option<String>) -> Self {
        Scene {
            vc_id,
            module,
            own_constraints: Vec::new(),
            constrained: constrained_views(view),
            subviews: Vec::new(),
            parent,
            actions: Vec::new(),
            segues: Vec::new(),
            segue_listing: None,
            containers: Vec::new(),
            control_outlets: Vec::new(),
            gestures: Vec::new(),
            gesture_views: Vec::new(),
            view_outlets: Vec::new(),
            pairs: Vec::new(),
            effect_contents: Vec::new(),
            accessibility: Vec::new(),
            switch_frames: Vec::new(),
            guides: Vec::new(),
        }
    }
}

/// A scene's segues as its controller's nib lists them, each with its
/// placeholder's index.
fn listed_segues<'a, 'i>(scene: &Scene, vc: Node<'a, 'i>) -> Vec<(usize, Node<'a, 'i>)> {
    let order: Vec<usize> = scene.segue_listing.clone().unwrap_or_else(|| (0..scene.segues.len()).collect());
    order.into_iter().filter_map(|i| vc.descendants().find(|d| d.attribute("id") == Some(scene.segues[i].as_str())).map(|n| (i, n))).collect()
}

/// A view's content hugging priorities, each 250 unless given.
fn hugging(nib: &mut Nib, n: Node) -> Result<Value, String> {
    let h = n.attribute("horizontalHuggingPriority").unwrap_or("250");
    let v = n.attribute("verticalHuggingPriority").unwrap_or("250");
    Ok(nib.string(&format!("{{{h}, {v}}}")))
}

/// A label's or text field's `textAlignment`: NSTextAlignment, left when
/// the document gives none.
fn text_alignment(s: Option<&str>) -> Result<i64, String> {
    Ok(match s {
        None | Some("left") => 0,
        Some("center") => 1,
        Some("right") => 2,
        Some("justified") => 3,
        Some("natural") => 4,
        Some(other) => return fail(format!("textAlignment {other} is not supported by this ibtool")),
    })
}

/// A button's autoresizing mask, which Xcode leaves out when the view
/// doesn't translate it into constraints.
fn ignored_autoresizing_mask(n: Node) -> Result<(), String> {
    if children(n, "autoresizingMask").next().is_some() && n.attribute("translatesAutoresizingMaskIntoConstraints").is_none() {
        return fail(format!("<autoresizingMask> inside a <{}> that translates it is not supported by this ibtool", n.tag_name().name()));
    }
    Ok(())
}

/// Whether a view is archived as not translating its autoresizing mask:
/// when the document says so and a constraint is on it.
fn does_not_translate(scene: &Scene, n: Node) -> bool {
    n.attribute("translatesAutoresizingMaskIntoConstraints").is_some() && n.attribute("id").is_some_and(|id| scene.constrained.contains(&id))
}

/// The ids of the views the constraints in `view` are on: their items, the
/// views whose own constraints name no first item, and a stack view's
/// arranged subviews, which it constrains.
fn constrained_views<'a>(view: Node<'a, '_>) -> Vec<&'a str> {
    let mut ids = Vec::new();
    for stack in view.descendants().filter(|d| d.has_tag_name("stackView")) {
        for s in children(stack, "subviews") {
            ids.extend(s.children().filter(|c| c.is_element()).filter_map(|c| c.attribute("id")));
        }
    }
    for c in view.descendants().filter(|d| d.has_tag_name("constraint")) {
        let owner = c.parent().and_then(|p| p.parent()).and_then(|v| v.attribute("id"));
        ids.extend(c.attribute("firstItem").or(owner));
        ids.extend(c.attribute("secondItem"));
    }
    ids
}

/// The constraints a subview holds: on its own size, width before height
/// whatever the document's order and each dimension's constant before its
/// aspect ratio; then, for a container, those between it
/// and its subviews or among them, ordered as a root view's are.
fn own_constraints(nib: &mut Nib, scene: &mut Scene, n: Node, name: &str) -> Result<Option<Value>, String> {
    let Some(cs) = children(n, "constraints").next() else { return Ok(None) };
    only_children(cs, &["constraint"])?;
    let id = attr(n, "id")?;
    let direct: Vec<Node> = children(n, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())).collect();
    let mut sizes = Vec::new();
    let mut others: Vec<((f64, f64), (&str, u8, [usize; 4]), Node, &str, &str)> = Vec::new();
    for c in children(cs, "constraint") {
        only_attributes(c, &[
            ("firstItem", None), ("firstAttribute", None), ("relation", None), ("secondItem", None), ("secondAttribute", None), ("multiplier", None),
            ("constant", None), ("symbolic", Some("YES")), ("priority", None), ("identifier", None), ("placeholder", Some("YES")), ("id", None),
        ])?;
        // A placeholder constraint is removed at build time, though its view
        // still doesn't translate its mask.
        if c.attribute("placeholder").is_some() {
            continue;
        }
        let first = c.attribute("firstItem").unwrap_or(id);
        let a = attr(c, "firstAttribute")?;
        let Some(second) = c.attribute("secondItem") else {
            if first != id || c.attribute("secondAttribute").is_some() || c.attribute("multiplier").is_some() {
                return fail("a subview's constraint without a second item on another view is not supported by this ibtool");
            }
            let rank = 2 * ["width", "height"].iter().position(|x| *x == a).ok_or_else(|| format!("a subview's own constraint on {a} is not supported by this ibtool"))?;
            if sizes.iter().any(|(r, _)| *r == rank) {
                return fail("two constraints on a subview's same dimension are not supported by this ibtool");
            }
            sizes.push((rank, c));
            continue;
        };
        if first == second {
            // An aspect ratio: after the constant on its dimension.
            let rank = match (first == id, a, attr(c, "secondAttribute")?) {
                (true, "width", "height") => 1,
                (true, "height", "width") => 3,
                _ => return fail("a constraint between an item and itself other than its own aspect ratio is not supported by this ibtool"),
            };
            if sizes.iter().any(|(r, _)| *r == rank) {
                return fail("two aspect ratios on a subview's same dimension are not supported by this ibtool");
            }
            sizes.push((rank, c));
            continue;
        }
        let key = constraint_key(id, first, a, second, attr(c, "secondAttribute")?, false)?;
        let v = direct.iter().find(|v| v.attribute("id") == Some(key.0)).ok_or_else(|| format!("a subview's constraints on {} are not supported by this ibtool", key.0))?;
        for item in [first, second] {
            if item != id && !direct.iter().any(|v| v.attribute("id") == Some(item)) {
                return fail(format!("a subview's constraints on {item} are not supported by this ibtool"));
            }
        }
        let f = subview_frame(*v)?;
        others.push(((f.x * f.x + f.y * f.y, f.x), key, c, first, second));
    }
    sizes.sort_by_key(|f| f.0);
    others.sort_by(|a, b| a.0 .0.total_cmp(&b.0 .0).then(a.0 .1.total_cmp(&b.0 .1)).then(a.1.cmp(&b.1)));
    order_ties(&mut others, |o| (o.1, o.2), &direct)?;
    let mut values = Vec::new();
    let item = |i: &str| reference(view_name(i));
    let mut made = Vec::new();
    for (rank, c) in sizes {
        let second = (rank % 2 == 1).then(|| reference(name.to_string()));
        made.push(document_constraint(nib, c, reference(name.to_string()), second)?);
    }
    for (_, _, c, first, second) in others {
        made.push(document_constraint(nib, c, item(first), Some(item(second)))?);
    }
    for m in made {
        let constraint_name = nib.fresh();
        scene.own_constraints.push((name.to_string(), constraint_name.clone()));
        match m {
            Value::New(o) => values.push(Value::New(o.named(constraint_name))),
            _ => unreachable!(),
        }
    }
    // An empty list writes nothing (one corpus storyboard).
    if values.is_empty() {
        return Ok(None);
    }
    Ok(Some(nib.array("NSMutableArray", values)))
}

fn subview(nib: &mut Nib, scene: &mut Scene, n: Node) -> Result<Value, String> {
    let f = subview_frame(n)?;
    placed_subview(nib, scene, n, f)
}

/// A subview where its superview lays it out.
fn placed_subview(nib: &mut Nib, scene: &mut Scene, n: Node, f: Frame) -> Result<Value, String> {
    let id = attr(n, "id")?;
    let name = view_name(id);
    scene.subviews.push((scene.parent.clone(), name.clone()));
    let mut values = Vec::new();
    geometry(&mut values, &f);
    let object = match n.tag_name().name() {
        "label" => {
            only_attributes(n, &[
                ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("userInteractionEnabled", Some("NO")), ("multipleTouchEnabled", Some("YES")),
                ("contentMode", None), ("insetsLayoutMarginsFromSafeArea", Some("NO")), ("alpha", None), ("hidden", Some("YES")), ("tag", None),
                ("restorationIdentifier", None), ("horizontalCompressionResistancePriority", None), ("verticalCompressionResistancePriority", None),
                ("adjustsFontForContentSizeCategory", Some("YES")),
                ("fixedFrame", Some("YES")), ("horizontalHuggingPriority", None), ("verticalHuggingPriority", None), ("text", None), ("textAlignment", None),
                ("lineBreakMode", None), ("numberOfLines", None), ("baselineAdjustment", Some("alignBaselines")),
                ("adjustsFontSizeToFit", Some("NO")), ("minimumFontSize", None), ("minimumScaleFactor", None),
                ("adjustsLetterSpacingToFitWidth", Some("YES")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("customClass", None),
                ("customModule", None), ("customModuleProvider", None), ("id", None),
            ])?;
            only_children(n, &[
                "rect", "autoresizingMask", "fontDescription", "nil", "color", "constraints", "connections", "string", "userDefinedRuntimeAttributes",
                "accessibility", "size",
            ])?;
            let pairs = runtime_attributes(nib, n, &name)?;
            scene.pairs.extend(pairs.into_iter().map(|p| (name.clone(), p)));
            accessibility(nib, scene, n, &name)?;
            view_connections(scene, n, &name)?;
            let highlighted = match (child(n, "nil", "highlightedColor"), child(n, "color", "highlightedColor")) {
                (Some(_), None) => None,
                (None, Some(c)) => Some(c),
                _ => return fail("a label needs one highlighted colour or nil"),
            };
            // No text colour at all is a declared labelColor; a nil one is
            // the default.
            let (text_color, declared_color) = match (child(n, "color", "textColor"), child(n, "nil", "textColor")) {
                (Some(c), None) => (Some(c), true),
                (None, Some(_)) => (None, false),
                (None, None) => (None, true),
                _ => return fail("a label needs one text colour or nil"),
            };
            // Its colours: text, highlighted and background, the background
            // before `UIDeepDrawRect` (one corpus storyboard).
            if let Some(c) = children(n, "color").find(|c| !matches!(c.attribute("key"), Some("textColor" | "highlightedColor" | "backgroundColor" | "shadowColor"))) {
                return fail(format!("a label's {} colour is not supported by this ibtool", c.attribute("key").unwrap_or("unkeyed")));
            }
            if let Some(c) = child(n, "color", "backgroundColor") {
                values.push(("UIBackgroundColor", color(nib, c)?));
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            // An opaque label, which older documents have (one probe).
            if n.attribute("opaque").is_none() {
                values.push(("UIOpaque", Value::Bool(true)));
            }
            hidden_and_tag(&mut values, n)?;
            if n.attribute("userInteractionEnabled").is_some() {
                values.push(("UIUserInteractionDisabled", Value::Bool(true)));
            }
            if n.attribute("multipleTouchEnabled").is_some() {
                values.push(("UIMultipleTouchEnabled", Value::Bool(true)));
            }
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            // A label that translates its autoresizing mask has the
            // document's, none when empty; otherwise flexible right and
            // bottom margins.
            let mask = match children(n, "autoresizingMask").next() {
                Some(m) if n.attribute("translatesAutoresizingMaskIntoConstraints").is_none() => autoresizing_mask(m)?,
                _ => 36,
            };
            if mask != 0 {
                values.push(("UIAutoresizingMask", Value::Int(mask)));
            }
            alpha(&mut values, n)?;
            let mode = content_mode(n.attribute("contentMode").unwrap_or("left"))?;
            if mode != 0 {
                values.push(("UIContentMode", Value::Int(mode)));
            }
            if n.attribute("clipsSubviews").is_some() {
                values.push(("UIClipsToBounds", Value::Bool(true)));
            }
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            // A label in a cell's style has no hugging priorities.
            match (n.attribute("horizontalHuggingPriority"), n.attribute("verticalHuggingPriority")) {
                (None, None) => {}
                _ => values.push(("UIViewContentHuggingPriority", hugging(nib, n)?)),
            }
            // Compression resistance, each 750 unless given.
            let resistance = (n.attribute("horizontalCompressionResistancePriority"), n.attribute("verticalCompressionResistancePriority"));
            if resistance != (None, None) {
                let (h, v) = (resistance.0.unwrap_or("750"), resistance.1.unwrap_or("750"));
                values.push(("UIViewContentCompressionResistancePriority", nib.string(&format!("{{{h}, {v}}}"))));
            }
            if n.attribute("insetsLayoutMarginsFromSafeArea").is_some() {
                values.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0)));
            }
            restoration_identifier(nib, &mut values, n);
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            // Long text is a <string> rather than an attribute.
            let text = match (n.attribute("text"), child(n, "string", "text")) {
                (Some(t), None) => t,
                (None, Some(s)) => {
                    only_children(s, &[])?;
                    s.text().unwrap_or("")
                }
                _ => return fail("a label needs one text for this ibtool"),
            };
            values.push(("UIText", nib.text(id, "text", text)));
            if n.attribute("adjustsFontForContentSizeCategory").is_some() {
                values.push(("UIAdjustsFontForContentSizeCategory", Value::Bool(true)));
            }
            values.push(("UIFont", font(nib, child(n, "fontDescription", "fontDescription"), None)?));
            // A declared text colour, even labelColor, stays as it is when
            // the trait collection changes.
            let text_color = match text_color {
                Some(c) => color(nib, c)?,
                None => nib.system_color("labelColor", &SystemColor::Gray([0.0, 1.0])),
            };
            values.push(("UITextColor", text_color));
            if let Some(c) = highlighted {
                values.push(("UIHighlightedColor", color(nib, c)?));
            }
            // A shadow colour, then its offset, (0, -1) unless given (two
            // probes).
            if let Some(c) = child(n, "color", "shadowColor") {
                values.push(("UIShadowColor", color(nib, c)?));
            }
            let offset = match children(n, "size").next() {
                None => [0.0, -1.0],
                Some(sz) if sz.attribute("key") == Some("shadowOffset") => [number(attr(sz, "width")?)?, number(attr(sz, "height")?)?],
                Some(_) => return fail("a label's sizes other than its shadow offset are not supported by this ibtool"),
            };
            values.push(("UIShadowOffset", doubles(&offset)));
            if let Some(lines) = n.attribute("numberOfLines") {
                values.push(("UINumberOfLines", Value::Int(lines.parse().map_err(|_| format!("not a number: {lines}"))?)));
            }
            // Older documents leave the baseline adjustment out, which is
            // aligning centres (one corpus storyboard).
            if n.attribute("baselineAdjustment").is_none() {
                values.push(("UIBaselineAdjustment", Value::Int(1)));
            }
            // Tail truncation, the default, is left out.
            let line_break = match attr(n, "lineBreakMode")? {
                "wordWrap" => Some(0),
                "characterWrap" => Some(1),
                "clip" => Some(2),
                "headTruncation" => Some(3),
                "tailTruncation" => None,
                "middleTruncation" => Some(5),
                other => return fail(format!("lineBreakMode=\"{other}\" on <label> is not supported by this ibtool")),
            };
            let minimum = minimum_font_size(n)?;
            if n.attribute("adjustsFontSizeToFit").is_none() {
                values.push(("UIAdjustsFontSizeToFit", Value::Bool(true)));
            }
            if let Some((size, _)) = minimum {
                values.push(("UIMinimumFontSize", Value::Float(size)));
            }
            if let Some(mode) = line_break {
                values.push(("UILineBreakMode", Value::Int(mode)));
            }
            values.push(("UITextAlignment", Value::Int(text_alignment(n.attribute("textAlignment"))?)));
            if let Some((_, scale)) = minimum {
                values.push(("UIMinimumScaleFactor", Value::Float(scale)));
            }
            if n.attribute("adjustsLetterSpacingToFitWidth").is_some() {
                values.push(("UIAdjustsLetterSpacingToFit", Value::Bool(true)));
            }
            values.push(("UIDisableUpdateTextColorOnTraitCollectionChange", Value::Bool(declared_color)));
            // A custom class is swapped in, last (two probes).
            if n.attribute("customClass").is_some() {
                let class = runtime_class_name(n, scene.module, "UILabel")?;
                values.push(("UIClassName", nib.string(&class)));
                values.push(("UIOriginalClassName", nib.string("UILabel")));
                Object::new("UIClassSwapper", values)
            } else {
                Object::new("UILabel", values)
            }
        }
        "textField" => {
            only_attributes(n, &[
                ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("contentMode", Some("scaleToFill")),
                ("contentHorizontalAlignment", Some("left")), ("contentVerticalAlignment", Some("center")), ("borderStyle", None),
                ("textAlignment", None), ("minimumFontSize", None), ("clearButtonMode", None), ("placeholder", None), ("text", None), ("enabled", Some("NO")),
                ("hidden", Some("YES")), ("restorationIdentifier", None), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
                ("customClass", None), ("customModule", None), ("customModuleProvider", None),
            ])?;
            let border = match n.attribute("borderStyle") {
                None => None,
                Some("line") => Some(1),
                Some("bezel") => Some(2),
                Some("roundedRect") => Some(3),
                Some(other) => return fail(format!("borderStyle=\"{other}\" on <textField> is not supported by this ibtool")),
            };
            let alignment = text_alignment(n.attribute("textAlignment"))?;
            let clear_button_mode = match n.attribute("clearButtonMode") {
                None => None,
                Some("whileEditing") => Some(1),
                Some("unlessEditing") => Some(2),
                Some("always") => Some(3),
                Some(other) => return fail(format!("clearButtonMode=\"{other}\" on <textField> is not supported by this ibtool")),
            };
            only_children(n, &["rect", "nil", "color", "fontDescription", "textInputTraits", "constraints", "connections", "accessibility"])?;
            accessibility(nib, scene, n, &name)?;
            let traits = children(n, "textInputTraits").next().ok_or("text field without text input traits")?;
            let trait_values = text_input_traits(nib, traits, false)?;
            for c in children(n, "connections") {
                only_children(c, &["outlet", "action"])?;
                for o in children(c, "outlet") {
                    only_attributes(o, &[("property", Some("delegate")), ("destination", None), ("id", None)])?;
                    if attr(o, "destination")? != scene.vc_id {
                        return fail("a text field's delegate other than the scene's view controller is not supported by this ibtool");
                    }
                    scene.control_outlets.push(("delegate".into(), name.clone()));
                }
                for a in children(c, "action") {
                    only_attributes(a, &[("selector", None), ("destination", None), ("eventType", None), ("id", None)])?;
                    let target = action_target(scene, a)?;
                    let mask = match attr(a, "eventType")? {
                        "valueChanged" => 4096,
                        "primaryActionTriggered" => 8192,
                        "editingDidBegin" => 65536,
                        "editingChanged" => 131072,
                        "editingDidEnd" => 262144,
                        "editingDidEndOnExit" => 524288,
                        other => return fail(format!("a text field's {other} action is not supported by this ibtool")),
                    };
                    scene.actions.push((attr(a, "selector")?.to_string(), name.clone(), mask, target.into()));
                }
            }
            // No text colour at all is the same as a nil one.
            let text_color = match (child(n, "color", "textColor"), child(n, "nil", "textColor")) {
                (Some(c), None) => Some(c),
                (None, _) => None,
                _ => return fail("a text field needs one text colour or nil"),
            };
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            hidden_and_tag(&mut values, n)?;
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            if n.attribute("clipsSubviews").is_some() {
                values.push(("UIClipsToBounds", Value::Bool(true)));
            }
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            restoration_identifier(nib, &mut values, n);
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            if n.attribute("enabled").is_some() {
                values.push(("UIDisabled", Value::Bool(true)));
            }
            values.push(("UIContentHorizontalAlignment", Value::Int(1)));
            values.push(("UIFont", font(nib, child(n, "fontDescription", "fontDescription"), None)?));
            if let Some(placeholder) = n.attribute("placeholder") {
                values.push(("UIPlaceholder", nib.text(id, "placeholder", placeholder)));
            }
            values.push(("UITextAlignment", Value::Int(alignment)));
            if let Some(text) = n.attribute("text") {
                values.push(("UIText", nib.text(id, "text", text)));
            }
            let text_color = match text_color {
                Some(c) => color(nib, c)?,
                None => nib.system_color("labelColor", &SystemColor::Gray([0.0, 1.0])),
            };
            values.push(("UITextColor", text_color));
            if let Some(border) = border {
                values.push(("UIBorderStyle", Value::Int(border)));
            }
            values.push(("UIAdjustsFontSizeToFit", Value::Bool(true)));
            values.push(("UIMinimumFontSize", Value::Float(number(attr(n, "minimumFontSize")?)? as f32)));
            if let Some(mode) = clear_button_mode {
                values.push(("UIClearButtonMode", Value::Int(mode)));
            }
            values.push(("UIClearButtonOffset", doubles(&[3.0, 1.0])));
            values.extend(trait_values);
            values.push(("UIDisableTextColorUpdateOnTraitCollectionChange", Value::Bool(true)));
            if border == Some(3) {
                values.push(("UIRoundedRectBackgroundCornerRadius", Value::Double(0.0)));
            }
            // A custom class swaps in for it, as a button's does (three
            // probes).
            if n.attribute("customClass").is_some() {
                let class = runtime_class_name(n, scene.module, "UITextField")?;
                values.push(("UIClassName", nib.string(&class)));
                values.push(("UIOriginalClassName", nib.string("UITextField")));
                Object::new("UIClassSwapper", values)
            } else {
                Object::new("UITextField", values)
            }
        }
        "tableView" => table::subview_table(nib, scene, n, &name, &f)?,
        "collectionView" => collection::subview_collection(nib, scene, n, &name, &f)?,
        "visualEffectView" => {
            only_attributes(n, &[
                ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("contentMode", Some("scaleToFill")),
                ("insetsLayoutMarginsFromSafeArea", Some("NO")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "view", "blurEffect", "constraints", "userDefinedRuntimeAttributes"])?;
            let pairs = runtime_attributes(nib, n, &name)?;
            scene.pairs.extend(pairs.into_iter().map(|p| (name.clone(), p)));
            if n.attribute("opaque").is_none() {
                return fail("an opaque visual effect view is not supported by this ibtool");
            }
            let blur = children(n, "blurEffect").next().ok_or("a visual effect view without a blur effect is not supported by this ibtool")?;
            only_attributes(blur, &[("style", None)])?;
            // UIBlurEffectStyle, and for the newer styles the automatic one.
            let (automatic, style) = match attr(blur, "style")? {
                "extraLight" => (None, 0),
                "light" => (None, 1),
                "dark" => (None, 2),
                "regular" => (Some(4), 1),
                "prominent" => (Some(5), 0),
                other => return fail(format!("blur style {other} is not supported by this ibtool")),
            };
            let content = child(n, "view", "contentView").ok_or("a visual effect view without a content view")?;
            only_attributes(content, &[
                ("key", None), ("insetsLayoutMarginsFromSafeArea", Some("NO")), ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("multipleTouchEnabled", Some("YES")), ("contentMode", Some("center")),
                ("id", None),
            ])?;
            only_children(content, &["rect", "autoresizingMask", "subviews", "constraints"])?;
            for required in ["opaque", "clipsSubviews", "multipleTouchEnabled", "contentMode"] {
                if content.attribute(required).is_none() {
                    return fail(format!("a visual effect view's content view without {required} is not supported by this ibtool"));
                }
            }
            for m in children(content, "autoresizingMask") {
                if autoresizing_mask(m)? != 18 {
                    return fail("a visual effect view's content view with another autoresizing mask is not supported by this ibtool");
                }
            }
            // Its content view is its child, holding its subviews.
            let content_name = view_name(attr(content, "id")?);
            scene.subviews.push((Some(name.clone()), content_name.clone()));
            let mut cv = Vec::new();
            geometry(&mut cv, &frame(content)?);
            let mut inner = Vec::new();
            for s in children(content, "subviews") {
                for c in s.children().filter(|c| c.is_element()) {
                    let outer = scene.parent.replace(content_name.clone());
                    let v = subview(nib, scene, c);
                    scene.parent = outer;
                    inner.push(v?);
                }
            }
            if !inner.is_empty() {
                cv.push(("UISubviews", nib.array("NSMutableArray", inner)));
            }
            cv.extend([("UIDeepDrawRect", Value::Bool(true)), ("UIMultipleTouchEnabled", Value::Bool(true)), ("UIAutoresizeSubviews", Value::Bool(true))]);
            cv.push(("UIAutoresizingMask", Value::Int(18)));
            cv.push(("UIShouldNotAdaptToMaterials", Value::Bool(true)));
            cv.push(("UIContentMode", Value::Int(4)));
            cv.push(("UIClipsToBounds", Value::Bool(true)));
            // Its constraints with its subviews, as a container's, listed
            // after its subviews (one probe).
            if let Some(own) = own_constraints(nib, scene, content, &content_name)? {
                cv.push(("UIViewAutolayoutConstraints", own));
                scene.effect_contents.push(content_name.clone());
            }
            if content.attribute("insetsLayoutMarginsFromSafeArea").is_some() {
                cv.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0)));
            }
            cv.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut cv);
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            values.push(("UIShouldNotAdaptToMaterials", Value::Bool(true)));
            if n.attribute("clipsSubviews").is_some() {
                values.push(("UIClipsToBounds", Value::Bool(true)));
            }
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            // Its own constraints, listed after its content view (one probe).
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
                scene.effect_contents.push(name.clone());
            }
            if n.attribute("insetsLayoutMarginsFromSafeArea").is_some() {
                values.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0)));
            }
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            let mut effect = Vec::new();
            if let Some(a) = automatic {
                effect.push(("UIBlurEffectAutomaticStyle", Value::Int(a)));
            }
            effect.push(("UIBlurEffectStyle", Value::Int(style)));
            values.push(("UIVisualEffectViewEffect", Value::New(Object::new("UIBlurEffect", effect))));
            values.push(("UIVisualEffectViewGroupName", Value::Nil));
            values.push(("UIVisualEffectViewContentView", Value::New(Object::new("_UIVisualEffectContentView", cv).named(content_name))));
            Object::new("UIVisualEffectView", values)
        }
        "pickerView" => {
            only_attributes(n, &[
                ("contentMode", Some("scaleToFill")), ("hidden", Some("YES")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "autoresizingMask", "color", "constraints", "connections"])?;
            view_connections(scene, n, &name)?;
            ignored_autoresizing_mask(n)?;
            for c in children(n, "color") {
                if c.attribute("key") != Some("backgroundColor") {
                    return fail("picker view colours other than a background colour are not supported by this ibtool");
                }
                values.push(("UIBackgroundColor", color(nib, c)?));
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIOpaque", Value::Bool(true)));
            hidden_and_tag(&mut values, n)?;
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewContentHuggingPriority", nib.string("{250, 250}")));
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            values.push(("UIAllowSelectingCells", Value::Bool(true)));
            Object::new("UIPickerView", values)
        }
        "datePicker" => {
            // Its date isn't archived; its mode unless date and time
            // (UIDatePickerMode), an inline style (UIPickerStyle 3) and its
            // minute interval, 0 unless given (eight probes).
            only_attributes(n, &[
                ("contentMode", Some("scaleToFill")), ("contentHorizontalAlignment", Some("center")), ("contentVerticalAlignment", Some("center")),
                ("datePickerMode", None), ("minuteInterval", None), ("style", None), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "date", "constraints"])?;
            let mode = match n.attribute("datePickerMode").unwrap_or("dateAndTime") {
                "time" => Some(0),
                "date" => Some(1),
                "dateAndTime" => None,
                "countDownTimer" => Some(3),
                other => return fail(format!("date picker mode {other} is not supported by this ibtool")),
            };
            let style = match n.attribute("style") {
                None => None,
                // Xcode lays wheels out at its own width: refused.
                Some("inline") => Some(3),
                Some(other) => return fail(format!("date picker style {other} is not supported by this ibtool")),
            };
            if mode.is_some() && style.is_some() {
                return fail("a date picker with both a mode and a style is not supported by this ibtool");
            }
            values.extend([("UIDeepDrawRect", Value::Bool(true)), ("UIOpaque", Value::Bool(true)), ("UIAutoresizeSubviews", Value::Bool(true))]);
            values.push(("UIAutoresizingMask", Value::Int(36)));
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            if let Some(m) = mode {
                values.push(("UIDatePickerMode", Value::Int(m)));
            }
            if let Some(s) = style {
                values.push(("UIPickerStyle", Value::Int(s)));
            }
            let interval: i64 = n.attribute("minuteInterval").map_or(Ok(0), |i| i.parse().map_err(|_| format!("not a number: {i}")))?;
            values.push(("UIMinuteInterval", Value::Int(interval)));
            if mode == Some(3) {
                values.push(("UIDatePickerUseCurrentDateDuringDecoding", Value::Bool(false)));
            }
            Object::new("UIDatePicker", values)
        }
        "segmentedControl" => {
            // Its segments are its subviews, the selected one last, each
            // 0x32 at (0, 16) whatever the control's size, positioned by
            // index (the first none, the second 3, the rest 1); `UISegments`
            // lists them in order (24 probes).
            only_attributes(n, &[
                ("opaque", Some("NO")), ("contentMode", Some("scaleToFill")), ("contentHorizontalAlignment", Some("left")),
                ("contentVerticalAlignment", Some("top")), ("segmentControlStyle", Some("plain")), ("selectedSegmentIndex", None),
                ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "segments", "constraints", "connections"])?;
            for c in children(n, "connections") {
                only_children(c, &["action"])?;
                for a in children(c, "action") {
                    only_attributes(a, &[("selector", None), ("destination", None), ("eventType", Some("valueChanged")), ("id", None)])?;
                    let target = action_target(scene, a)?;
                    scene.actions.push((attr(a, "selector")?.to_string(), name.clone(), 4096, target.into()));
                }
            }
            let id = attr(n, "id")?;
            let segments: Vec<Node> = children(n, "segments").flat_map(|ss| children(ss, "segment")).collect();
            let selected: usize = attr(n, "selectedSegmentIndex")?.parse().map_err(|_| "a segmented control's selected index must be a number")?;
            if segments.len() < 2 || selected >= segments.len() {
                return fail("a segmented control with fewer than two segments, or none selected, is not supported by this ibtool");
            }
            let mut built = Vec::new();
            for (i, sg) in segments.iter().enumerate() {
                only_attributes(*sg, &[("title", None)])?;
                let mut sv = vec![("UIBounds", doubles(&[0.0, 0.0, 0.0, 32.0])), ("UICenter", doubles(&[0.0, 16.0]))];
                sv.extend([("UIDeepDrawRect", Value::Bool(true)), ("UIAutoresizeSubviews", Value::Bool(true)), ("UIViewSemanticContentAttribute", Value::Int(0))]);
                nib.large_content(&mut sv);
                sv.push(("UISegmentInfo", nib.text(id, &format!("segmentTitles[{i}]"), attr(*sg, "title")?)));
                sv.push(("UISegmentStyle", Value::Int(5)));
                match i {
                    0 => {}
                    1 => sv.push(("UISegmentPosition", Value::Int(3))),
                    _ => sv.push(("UISegmentPosition", Value::Int(1))),
                }
                built.push((i, format!("{name} segment {i}"), Object::new("UISegment", sv)));
            }
            let order: Vec<String> = built.iter().map(|b| b.1.clone()).collect();
            let selected_segment = built.remove(selected);
            built.push(selected_segment);
            let subviews = built.into_iter().map(|(_, segment_name, o)| Value::New(o.named(segment_name))).collect();
            values.push(("UISubviews", nib.array("NSMutableArray", subviews)));
            values.extend([("UIDeepDrawRect", Value::Bool(true)), ("UIAutoresizeSubviews", Value::Bool(true)), ("UIAutoresizingMask", Value::Int(36))]);
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewContentHuggingPriority", nib.string("{250, 250}")));
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            values.extend([
                ("UIContentVerticalAlignment", Value::Int(1)),
                ("UIContentVerticalAlignment2", Value::Int(1)),
                ("UIContentHorizontalAlignment", Value::Int(1)),
                ("UIPointerInteractionEnabled", Value::Bool(true)),
            ]);
            values.push(("UISegments", nib.array("NSMutableArray", order.into_iter().map(reference).collect())));
            values.push(("UISelectedSegmentIndex", Value::Int(selected as i64)));
            Object::new("UISegmentedControl", values)
        }
        "mapView" => {
            // An MKMapView: a view's keys, zero layout margins, then MapKit's
            // with their defaults but for the scale and compass (five
            // probes).
            only_attributes(n, &[
                ("clipsSubviews", Some("YES")), ("multipleTouchEnabled", Some("YES")), ("contentMode", Some("scaleToFill")), ("mapType", Some("standard")),
                ("showsScale", Some("YES")), ("showsCompass", Some("NO")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "constraints", "connections"])?;
            for required in ["clipsSubviews", "multipleTouchEnabled"] {
                if n.attribute(required).is_none() {
                    return fail(format!("a map view without {required} is not supported by this ibtool"));
                }
            }
            view_connections(scene, n, &name)?;
            values.extend([
                ("UIDeepDrawRect", Value::Bool(true)),
                ("UIOpaque", Value::Bool(true)),
                ("UIMultipleTouchEnabled", Value::Bool(true)),
                ("UIAutoresizeSubviews", Value::Bool(true)),
                ("UIAutoresizingMask", Value::Int(36)),
                ("UIClipsToBounds", Value::Bool(true)),
            ]);
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewLayoutMargins", doubles(&[0.0; 4])));
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            values.extend([
                ("MKDelegate", Value::Nil),
                ("MKMapType", Value::Int(0)),
                ("MKZoomEnabled", Value::Bool(true)),
                ("MKScrollEnabled", Value::Bool(true)),
                ("MKShowsUserLocation", Value::Bool(false)),
                ("MKPitchEnabled", Value::Bool(true)),
                ("MKRotateEnabled", Value::Bool(true)),
                ("MKShowsBuildings", Value::Bool(true)),
                ("MKShowsTraffic", Value::Bool(false)),
                ("MKShowsCompass", Value::Bool(n.attribute("showsCompass").is_none())),
                ("MKShowsScaleKey", Value::Bool(n.attribute("showsScale").is_some())),
                ("MKShowsPointsOfInterest", Value::Bool(true)),
                ("MKPointOfInterestFilter", Value::Nil),
            ]);
            let configuration = Object::new("MKStandardMapConfiguration", vec![
                ("MKMapConfigurationElevationStyle", Value::Int(0)),
                ("MKMapConfigurationShowsTraffic", Value::Bool(false)),
                ("MKMapConfigurationShowsHiking", Value::Bool(false)),
                ("MKMapConfigurationShowsTopographicFeatures", Value::Bool(false)),
                ("MKMapConfigurationPointOfInterestFilter", Value::Nil),
                ("MKStandardMapConfigurationEmphasisStyle", Value::Int(0)),
            ]);
            values.push(("MKPreferredConfiguration", Value::New(configuration)));
            values.push(("MKPitchButtonVisibility", Value::Int(1)));
            values.push(("MKShowsUserTrackingButton", Value::Bool(false)));
            Object::new("MKMapView", values)
        }
        "searchBar" => {
            only_attributes(n, &[
                ("key", Some("tableHeaderView")), ("contentMode", Some("redraw")), ("text", None), ("placeholder", None), ("barStyle", Some("black")),
                ("searchBarStyle", Some("minimal")), ("translucent", Some("NO")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "autoresizingMask", "textInputTraits", "constraints", "connections"])?;
            view_connections(scene, n, &name)?;
            let traits = child(n, "textInputTraits", "textInputTraits").ok_or("a search bar without text input traits")?;
            only_attributes(traits, &[("key", None)])?;
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            if n.attribute("translucent").is_some() {
                values.push(("UIOpaque", Value::Bool(true)));
            }
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            let mask = match children(n, "autoresizingMask").next() {
                Some(m) if n.attribute("translatesAutoresizingMaskIntoConstraints").is_none() => autoresizing_mask(m)?,
                _ => 36,
            };
            values.push(("UIAutoresizingMask", Value::Int(mask)));
            values.push(("UIContentMode", Value::Int(3)));
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewContentHuggingPriority", nib.string("{250, 250}")));
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            if n.attribute("barStyle").is_some() {
                values.push(("UIBarStyle", Value::Int(1)));
            }
            values.push(("UISearchBarTranslucence", Value::Int(if n.attribute("translucent").is_some() { 2 } else { 1 })));
            values.push(("UIText", nib.text(id, "text", n.attribute("text").unwrap_or(""))));
            if let Some(p) = n.attribute("placeholder") {
                values.push(("UIPlaceholder", nib.text(id, "placeholder", p)));
            }
            values.push(("UIEnablesReturnKeyAutomatically", Value::Bool(false)));
            values.push(("UISecureTextEntry", Value::Bool(false)));
            if n.attribute("searchBarStyle").is_some() {
                values.push(("UISearchBarStyle", Value::Int(2)));
            }
            Object::new("UISearchBar", values)
        }
        "textView" => {
            only_attributes(n, &[
                ("clipsSubviews", Some("YES")), ("multipleTouchEnabled", Some("YES")), ("contentMode", Some("scaleToFill")), ("textAlignment", None),
                ("text", None), ("editable", Some("NO")), ("selectable", Some("NO")), ("scrollEnabled", Some("NO")), ("bounces", Some("NO")),
                ("bouncesZoom", Some("NO")), ("showsHorizontalScrollIndicator", Some("NO")), ("showsVerticalScrollIndicator", Some("NO")),
                ("indicatorStyle", None),
                ("adjustsFontForContentSizeCategory", Some("YES")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "color", "nil", "string", "fontDescription", "textInputTraits", "constraints", "connections", "accessibility"])?;
            // An accessibility configuration as any view's (one probe).
            accessibility(nib, scene, n, &name)?;
            for c in children(n, "color") {
                if !matches!(c.attribute("key"), Some("backgroundColor" | "textColor")) {
                    return fail("text view colours other than its background and text are not supported by this ibtool");
                }
            }
            // No text colour is the same as nil.
            let text_color = match (child(n, "color", "textColor"), child(n, "nil", "textColor")) {
                (Some(c), None) => Some(c),
                (None, _) => None,
                _ => return fail("a text view with a text colour and nil is not supported by this ibtool"),
            };
            let text = match (n.attribute("text"), children(n, "string").find(|c| c.attribute("key") == Some("text"))) {
                (Some(t), None) => Some(t.to_string()),
                (None, Some(s)) => {
                    only_attributes(s, &[("key", Some("text"))])?;
                    Some(s.text().unwrap_or("").to_string())
                }
                (None, None) => None,
                _ => return fail("a text view with two texts is not supported by this ibtool"),
            };
            let traits = children(n, "textInputTraits").next().ok_or("text view without text input traits")?;
            if traits.attribute("secureTextEntry").is_some() {
                return fail("a secure text view is not supported by this ibtool");
            }
            let trait_values = text_input_traits(nib, traits, true)?;
            for c in children(n, "connections") {
                only_children(c, &["outlet"])?;
                for o in children(c, "outlet") {
                    only_attributes(o, &[("property", Some("delegate")), ("destination", None), ("id", None)])?;
                    if attr(o, "destination")? != scene.vc_id {
                        return fail("a text view's delegate other than the scene's view controller is not supported by this ibtool");
                    }
                    scene.control_outlets.push(("delegate".into(), name.clone()));
                }
            }
            let scrolls = n.attribute("scrollEnabled").is_none();
            // An opaque grey white background is the system background.
            if let Some(c) = child(n, "color", "backgroundColor") {
                let background = if nib.color_matched && opaque_grey_white(c)? { system_color_named(nib, "systemBackgroundColor")? } else { color(nib, c)? };
                values.push(("UIBackgroundColor", background));
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIOpaque", Value::Bool(true)));
            if n.attribute("multipleTouchEnabled").is_some() {
                values.push(("UIMultipleTouchEnabled", Value::Bool(true)));
            }
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            // A text view that doesn't scroll doesn't clip either.
            if n.attribute("clipsSubviews").is_some() && scrolls {
                values.push(("UIClipsToBounds", Value::Bool(true)));
            }
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            values.push(("UIScrollViewIndicatorInsetAdjustmentBehavior", Value::Int(1)));
            // Its indicator style (UIScrollViewIndicatorStyle: black 1, white
            // 2; two probes).
            match n.attribute("indicatorStyle") {
                None | Some("default") => {}
                Some("black") => values.push(("UIIndicatorStyle", Value::Int(1))),
                Some("white") => values.push(("UIIndicatorStyle", Value::Int(2))),
                Some(other) => return fail(format!("indicatorStyle {other} is not supported by this ibtool")),
            }
            if n.attribute("bounces").is_some() {
                values.push(("UIBounceEnabled", Value::Bool(false)));
            }
            if n.attribute("showsHorizontalScrollIndicator").is_some() {
                values.push(("UIShowsHorizontalScrollIndicator", Value::Bool(false)));
            }
            if n.attribute("showsVerticalScrollIndicator").is_some() {
                values.push(("UIShowsVerticalScrollIndicator", Value::Bool(false)));
            }
            if !scrolls {
                values.push(("UIScrollDisabled", Value::Bool(true)));
            }
            if n.attribute("bouncesZoom").is_none() {
                values.push(("UIBouncesZoom", Value::Bool(true)));
            }
            values.push(("UIScrollViewContentInsetAdjustmentBehavior", Value::Int(0)));
            let alignment = text_alignment(n.attribute("textAlignment"))?;
            let font_node = Some(child(n, "fontDescription", "fontDescription").ok_or("a text view without a font")?);
            // The text is attributed with its colour, font and alignment
            // (AppKit's NSTextAlignment: right 1, center 2; left left out),
            // which the view's own values then refer to.
            if let Some(text) = text {
                // In `Base.lproj` its text is localizable as `<id>.text`, the
                // attributed text's string its development value (one probe).
                values.push(("UIText", nib.text(attr(n, "id")?, "text", &text)));
                let string = nib.string(&text);
                let mut attributes = Vec::new();
                if let Some(c) = text_color {
                    attributes.extend([nib.string("NSColor"), color(nib, c)?]);
                }
                attributes.extend([nib.string("NSFont"), font(nib, font_node, None)?]);
                let mut paragraph = Vec::new();
                let ns_alignment = match alignment {
                    0 => None,
                    1 => Some(2),
                    2 => Some(1),
                    other => Some(other),
                };
                if let Some(a) = ns_alignment {
                    paragraph.push(("NSAlignment", Value::Int(a)));
                }
                paragraph.push(("NSTabStops", Value::Nil));
                attributes.extend([nib.string("NSParagraphStyle"), Value::New(Object::new("NSMutableParagraphStyle", paragraph))]);
                let attributes = nib.array("NSDictionary", attributes);
                values.push(("UIAttributedText", Value::New(Object::new("NSAttributedString", vec![("NSString", string), ("NSAttributes", attributes)]))));
            }
            if n.attribute("adjustsFontForContentSizeCategory").is_some() {
                values.push(("UIAdjustsFontForContentSizeCategory", Value::Bool(true)));
            }
            values.push(("UIFont", font(nib, font_node, None)?));
            if let Some(c) = text_color {
                values.push(("UITextColor", color(nib, c)?));
            }
            values.push(("UITextAlignment", Value::Int(alignment)));
            if n.attribute("editable").is_some() {
                values.push(("UIEditable", Value::Bool(false)));
            }
            if n.attribute("selectable").is_some() {
                values.push(("UISelectable", Value::Bool(false)));
            }
            values.extend(trait_values);
            // Highlighted text is system blue, outlined in the secondary
            // system fill.
            let color_key = nib.string("NSColor");
            let blue = system_color_named(nib, "systemBlueColor")?;
            let outline_key = nib.string("NSTextHighlightOutlineColor");
            let fill = system_color_named(nib, "secondarySystemFillColor")?;
            values.push(("UITextHighlightAttributes", nib.array("NSDictionary", vec![color_key, blue, outline_key, fill])));
            values.push(("UITextAllowsNumberPadPopover", Value::Bool(true)));
            Object::new("UITextView", values)
        }
        "button" => {
            // `fixedFrame` is Interface Builder's alone, and a system button
            // is archived as a rounded-rect one; a custom one has no type.
            only_attributes(n, &[
                ("opaque", Some("NO")), ("contentMode", None), ("fixedFrame", Some("YES")), ("contentHorizontalAlignment", None),
                ("contentVerticalAlignment", None), ("buttonType", None), ("lineBreakMode", None), ("enabled", Some("NO")), ("selected", Some("YES")),
                ("clipsSubviews", Some("YES")), ("hidden", Some("YES")), ("tag", None), ("alpha", None), ("customClass", None), ("customModule", None),
                ("customModuleProvider", None), ("horizontalHuggingPriority", None), ("verticalHuggingPriority", None),
                ("horizontalCompressionResistancePriority", None), ("verticalCompressionResistancePriority", None),
                ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("insetsLayoutMarginsFromSafeArea", Some("NO")), ("id", None),
            ])?;
            // UIControlContentHorizontalAlignment and its vertical one,
            // the centre left out.
            let horizontal = match n.attribute("contentHorizontalAlignment") {
                None | Some("center") => None,
                Some("left") => Some(1),
                Some("right") => Some(2),
                Some("fill") => Some(3),
                Some("leading") => Some(4),
                Some("trailing") => Some(5),
                Some(other) => return fail(format!("contentHorizontalAlignment=\"{other}\" on <button> is not supported by this ibtool")),
            };
            let vertical = match n.attribute("contentVerticalAlignment") {
                None | Some("center") => None,
                Some("top") => Some(1),
                Some("bottom") => Some(2),
                Some("fill") => Some(3),
                Some(other) => return fail(format!("contentVerticalAlignment=\"{other}\" on <button> is not supported by this ibtool")),
            };
            let button_type = match n.attribute("buttonType") {
                Some("roundedRect" | "system") => Some(1),
                None => None,
                Some(other) => return fail(format!("buttonType=\"{other}\" on <button> is not supported by this ibtool")),
            };
            only_children(n, &[
                "rect", "autoresizingMask", "state", "connections", "fontDescription", "color", "constraints", "userDefinedRuntimeAttributes", "accessibility",
                "inset", "buttonConfiguration",
            ])?;
            // A configuration replaces the states and font, which Xcode then
            // leaves out (four probes).
            let configuration = child(n, "buttonConfiguration", "configuration");
            if configuration.is_some() && (child(n, "fontDescription", "fontDescription").is_some() || child(n, "inset", "contentEdgeInsets").is_some()) {
                return fail("a button configuration beside a font or insets is not supported by this ibtool");
            }
            let pairs = runtime_attributes(nib, n, &name)?;
            scene.pairs.extend(pairs.into_iter().map(|p| (name.clone(), p)));
            accessibility(nib, scene, n, &name)?;
            ignored_autoresizing_mask(n)?;
            // Its states by UIControlState, in that order whatever the
            // document's: normal 0, highlighted 1, disabled 2, selected 4.
            let mut states: Vec<(i64, &str, Node)> = Vec::new();
            for st in children(n, "state") {
                let key = attr(st, "key")?;
                let value = match key {
                    "normal" => 0,
                    "highlighted" => 1,
                    "disabled" => 2,
                    "selected" => 4,
                    other => return fail(format!("a button's {other} state is not supported by this ibtool")),
                };
                if states.iter().any(|s| s.0 == value) {
                    return fail("a button with two of one state is not supported by this ibtool");
                }
                only_attributes(st, &[("key", None), ("title", None), ("image", None), ("backgroundImage", None)])?;
                only_children(st, &["color"])?;
                for c in children(st, "color") {
                    if !matches!(c.attribute("key"), Some("titleColor" | "titleShadowColor")) {
                        return fail("button state colours other than title and title shadow colours are not supported by this ibtool");
                    }
                }
                states.push((value, key, st));
            }
            states.sort_by_key(|s| s.0);
            for c in children(n, "color") {
                if !matches!(c.attribute("key"), Some("backgroundColor" | "tintColor")) {
                    return fail("button colours other than background and tint colours are not supported by this ibtool");
                }
            }
            for c in children(n, "connections") {
                only_children(c, &["action", "segue"])?;
                for a in children(c, "action") {
                    only_attributes(a, &[("selector", None), ("destination", None), ("eventType", None), ("id", None)])?;
                    let target = action_target(scene, a)?;
                    let mask = control_event(attr(a, "eventType")?)?;
                    scene.actions.push((attr(a, "selector")?.to_string(), name.clone(), mask, target.into()));
                }
                // Xcode keeps only a control's last segue.
                if children(c, "segue").count() > 1 {
                    return fail("a control with more than one segue is not supported by this ibtool");
                }
                for sg in children(c, "segue") {
                    let placeholder = format!("upstream segue {}", scene.segues.len());
                    scene.segues.push(attr(sg, "id")?.to_string());
                    scene.actions.push(("perform:".into(), name.clone(), 64, placeholder));
                }
            }
            if let Some(c) = child(n, "color", "backgroundColor") {
                values.push(("UIBackgroundColor", color(nib, c)?));
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            if n.attribute("opaque").is_none() {
                values.push(("UIOpaque", Value::Bool(true)));
            }
            hidden_and_tag(&mut values, n)?;
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            alpha(&mut values, n)?;
            // A content mode other than scale to fill after the mask (one
            // probe).
            let mode = content_mode(n.attribute("contentMode").unwrap_or("scaleToFill"))?;
            if mode != 0 {
                values.push(("UIContentMode", Value::Int(mode)));
            }
            if n.attribute("clipsSubviews").is_some() {
                if child(n, "color", "tintColor").is_some() {
                    return fail("a clipping button with a tint colour is not supported by this ibtool");
                }
                values.push(("UIClipsToBounds", Value::Bool(true)));
            }
            if let Some(c) = child(n, "color", "tintColor") {
                values.push(("UITintColor", color(nib, c)?));
            }
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            // Hugging (each 250 unless given) and compression resistance
            // (each 750 unless given), when given (three probes).
            let given = |h: &str, v: &str| n.attribute(h).is_some() || n.attribute(v).is_some();
            if given("horizontalHuggingPriority", "verticalHuggingPriority") {
                values.push(("UIViewContentHuggingPriority", hugging(nib, n)?));
            }
            if given("horizontalCompressionResistancePriority", "verticalCompressionResistancePriority") {
                let p = |k: &str| n.attribute(k).unwrap_or("750");
                let value = format!("{{{}, {}}}", p("horizontalCompressionResistancePriority"), p("verticalCompressionResistancePriority"));
                values.push(("UIViewContentCompressionResistancePriority", nib.string(&value)));
            }
            // Turned off, before the semantic content attribute (one probe).
            if n.attribute("insetsLayoutMarginsFromSafeArea").is_some() {
                values.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0)));
            }
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            // Selected, then disabled (two probes).
            if n.attribute("selected").is_some() {
                values.push(("UISelected", Value::Bool(true)));
            }
            if n.attribute("enabled").is_some() {
                values.push(("UIDisabled", Value::Bool(true)));
            }
            if let Some(v) = vertical {
                values.push(("UIContentVerticalAlignment", Value::Int(v)));
                values.push(("UIContentVerticalAlignment2", Value::Int(v)));
            }
            if let Some(h) = horizontal {
                values.push(("UIContentHorizontalAlignment", Value::Int(h)));
            }
            let mut stateful = Vec::new();
            for (value, key, st) in states.iter().filter(|_| configuration.is_none()) {
                stateful.push(nib.int_number(*value));
                let mut content = Vec::new();
                if let Some(title) = st.attribute("title") {
                    // A localizable build's keys for other states' titles
                    // aren't established.
                    if nib.localizable && *value != 0 {
                        return fail("a localizable button's title for a state other than normal is not supported by this ibtool");
                    }
                    content.push(("UITitle", nib.text(id, &format!("{key}Title"), title)));
                }
                if let Some(image) = st.attribute("image") {
                    content.push(("UIImage", nib.image(image)));
                }
                if let Some(image) = st.attribute("backgroundImage") {
                    content.push(("UIBackgroundImage", nib.image(image)));
                }
                if let Some(c) = child(*st, "color", "titleColor") {
                    content.push(("UITitleColor", color(nib, c)?));
                }
                if let Some(c) = child(*st, "color", "titleShadowColor") {
                    content.push(("UIShadowColor", color(nib, c)?));
                }
                stateful.push(Value::New(Object::new("UIButtonContent", content)));
            }
            // No states at all, no stateful content (two probes).
            if configuration.is_none() && !stateful.is_empty() {
                values.push(("UIButtonStatefulContent", nib.array("NSMutableDictionary", stateful)));
            }
            // NSLineBreakMode unless the default middle truncation (two
            // probes).
            let line_break = match n.attribute("lineBreakMode") {
                None | Some("middleTruncation") => None,
                Some("wordWrap") => Some(0),
                Some("tailTruncation") => Some(4),
                Some(other) => return fail(format!("a button's lineBreakMode {other} is not supported by this ibtool")),
            };
            if let Some(mode) = line_break {
                if configuration.is_some() {
                    return fail("a button configuration beside a line break mode is not supported by this ibtool");
                }
                values.push(("UILineBreakMode", Value::Int(mode)));
            }
            values.push(("UIAdjustsImageWhenHighlighted", Value::Bool(true)));
            values.push(("UIAdjustsImageWhenDisabled", Value::Bool(true)));
            if let Some(t) = button_type {
                values.push(("UIButtonType", Value::Int(t)));
            }
            // A custom button's default font is larger.
            let default_size = if button_type.is_some() { 15.0 } else { 18.0 };
            if configuration.is_none() {
                values.push(("UIFont", font(nib, child(n, "fontDescription", "fontDescription"), Some(default_size))?));
            }
            // Its insets: top, left, bottom and right from the document's
            // minimum y and x and maximum y and x.
            if let Some(i) = children(n, "inset").find(|i| !["contentEdgeInsets", "titleEdgeInsets", "imageEdgeInsets"].contains(&i.attribute("key").unwrap_or(""))) {
                return fail(format!("a button's {} inset is not supported by this ibtool", i.attribute("key").unwrap_or("unkeyed")));
            }
            for (k, key) in [("contentEdgeInsets", "UIContentEdgeInsets"), ("titleEdgeInsets", "UITitleEdgeInsets"), ("imageEdgeInsets", "UIImageEdgeInsets")] {
                if let Some(i) = child(n, "inset", k) {
                    only_attributes(i, &[("key", None), ("minX", None), ("minY", None), ("maxX", None), ("maxY", None)])?;
                    let edge = |a: &str| number(attr(i, a)?);
                    values.push((key, doubles(&[edge("minY")?, edge("minX")?, edge("maxY")?, edge("maxX")?])));
                }
            }
            let configuration = match configuration {
                Some(c) => button_configuration(nib, c, id)?,
                None => Value::Nil,
            };
            values.push(("UIButtonConfiguration", configuration));
            if n.attribute("customClass").is_some() {
                let class = runtime_class_name(n, scene.module, "UIButton")?;
                values.push(("UIClassName", nib.string(&class)));
                values.push(("UIOriginalClassName", nib.string("UIButton")));
                Object::new("UIClassSwapper", values)
            } else {
                Object::new("UIButton", values)
            }
        }
        "stackView" => {
            only_attributes(n, &[
                ("opaque", Some("NO")), ("contentMode", Some("scaleToFill")), ("axis", Some("vertical")), ("distribution", None), ("alignment", None),
                ("spacing", None), ("spacingType", None), ("baselineRelativeArrangement", Some("YES")), ("layoutMarginsRelativeArrangement", Some("YES")),
                ("layoutMarginsFollowReadableWidth", Some("YES")), ("hidden", Some("YES")), ("insetsLayoutMarginsFromSafeArea", Some("NO")),
                ("customClass", None), ("customModule", None), ("customModuleProvider", None),
                ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "subviews", "edgeInsets", "directionalEdgeInsets", "color", "constraints", "userDefinedRuntimeAttributes"])?;
            let pairs = runtime_attributes(nib, n, &name)?;
            scene.pairs.extend(pairs.into_iter().map(|p| (name.clone(), p)));
            let vertical = n.attribute("axis").is_some();
            let distribution = match n.attribute("distribution") {
                None | Some("fill") => 0,
                Some("fillEqually") => 1,
                Some("fillProportionally") => 2,
                Some("equalSpacing") => 3,
                Some("equalCentering") => 4,
                Some(other) => return fail(format!("stack view distribution {other} is not supported by this ibtool")),
            };
            // UIStackViewAlignment: leading and top are one value, trailing
            // and bottom another, whichever the axis.
            let alignment = match (n.attribute("alignment"), vertical) {
                (None | Some("fill"), _) => 0,
                (Some("leading" | "top"), _) => 1,
                (Some("firstBaseline"), false) => 2,
                (Some("center"), _) => 3,
                (Some("trailing" | "bottom"), _) => 4,
                (Some("lastBaseline"), false) => 5,
                (Some(other), _) => return fail(format!("stack view alignment {other} is not supported by this ibtool here")),
            };
            // Edge insets are its layout margins, which it then arranges
            // within, whatever its arrangement setting says; without them the
            // setting changes nothing.
            // Plain or directional (one probe), as other views' are.
            let (margins, directional) = match layout_margins(n)? {
                Some((m, d)) => (Some(m), d),
                None => (None, false),
            };
            let mut arranged = Vec::new();
            for s in children(n, "subviews") {
                for c in s.children().filter(|c| c.is_element()) {
                    let outer = scene.parent.replace(name.clone());
                    let v = subview(nib, scene, c);
                    scene.parent = outer;
                    arranged.push(v?);
                }
            }
            let refs = arranged_names(&arranged);
            // An empty stack view has neither list (one probe).
            let empty = arranged.is_empty();
            if !empty {
                values.push(("UISubviews", nib.array("NSMutableArray", arranged)));
            }
            for c in children(n, "color") {
                if c.attribute("key") != Some("backgroundColor") {
                    return fail("stack view colours other than a background colour are not supported by this ibtool");
                }
                values.push(("UIBackgroundColor", color(nib, c)?));
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            if n.attribute("hidden").is_some() {
                values.push(("UIHidden", Value::Bool(true)));
            }
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            values.push(("UIViewLayoutMargins", doubles(&margins.unwrap_or([0.0; 4]))));
            if n.attribute("layoutMarginsFollowReadableWidth").is_some() {
                values.push(("UIViewLayoutMarginsFollowReadableWidth", Value::Bool(true)));
            }
            if directional {
                values.push(("UIViewLayoutMarginsAreDirectional", Value::Bool(true)));
            }
            if n.attribute("insetsLayoutMarginsFromSafeArea").is_some() {
                values.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0)));
            }
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            if !empty {
                values.push(("UIStackViewArrangedSubviews", nib.array("NSArray", refs)));
            }
            values.push(("UIStackViewAxis", Value::Int(vertical as i64)));
            values.push(("UIStackViewDistribution", Value::Int(distribution)));
            values.push(("UIStackViewAlignment", Value::Int(alignment)));
            // The standard spacing is archived as none, whatever the spacing.
            let spacing = match n.attribute("spacingType") {
                Some("standard") => 0.0,
                None => n.attribute("spacing").map(number).transpose()?.unwrap_or(0.0),
                Some(other) => return fail(format!("spacingType=\"{other}\" on <stackView> is not supported by this ibtool")),
            };
            values.push(("UIStackViewSpacing", Value::Double(spacing)));
            values.push(("UIStackViewBaselineRelative", Value::Bool(n.attribute("baselineRelativeArrangement").is_some())));
            values.push(("UIStackViewLayoutMarginsRelative", Value::Bool(margins.is_some())));
            // A custom class is swapped in, last (one probe).
            if n.attribute("customClass").is_some() {
                let class = runtime_class_name(n, scene.module, "UIStackView")?;
                values.push(("UIClassName", nib.string(&class)));
                values.push(("UIOriginalClassName", nib.string("UIStackView")));
                Object::new("UIClassSwapper", values)
            } else {
                Object::new("UIStackView", values)
            }
        }
        "view" | "containerView" | "scrollView" => {
            // A container view is a plain view its embed segue fills; a
            // scroll view one with the scrolling values tables have.
            let container = n.has_tag_name("containerView");
            let scroll = n.has_tag_name("scrollView");
            const SCROLLING: [&str; 6] =
                ["showsHorizontalScrollIndicator", "showsVerticalScrollIndicator", "alwaysBounceVertical", "directionalLockEnabled", "minimumZoomScale", "maximumZoomScale"];
            if !scroll && SCROLLING.iter().any(|a| n.attribute(*a).is_some()) {
                return fail("scrolling attributes on a <view> are not supported by this ibtool");
            }
            if container {
                // Its placeholder's intrinsic size is Interface Builder's alone.
                only_attributes(n, &[
                    ("opaque", Some("NO")), ("contentMode", Some("scaleToFill")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")),
                    ("insetsLayoutMarginsFromSafeArea", Some("NO")), ("placeholderIntrinsicWidth", None), ("placeholderIntrinsicHeight", None),
                    ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("id", None),
                ])?;
                only_children(n, &["rect", "autoresizingMask", "color", "constraints", "connections"])?;
                let segues: Vec<Node> = children(n, "connections").flat_map(|c| c.children().filter(|c| c.is_element())).collect();
                let [sg] = segues[..] else {
                    return fail("a container view needs one embed segue for this ibtool");
                };
                if !sg.has_tag_name("segue") || sg.attribute("kind") != Some("embed") {
                    return fail("a container view's connections other than its embed segue are not supported by this ibtool");
                }
                let placeholder = format!("upstream segue {}", scene.segues.len());
                scene.segues.push(attr(sg, "id")?.to_string());
                scene.containers.push((placeholder, name.clone()));
            } else {
                // A table's header and footer views are its subviews.
                only_attributes(n, &[
                    ("key", None), ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("contentMode", None), ("fixedFrame", Some("YES")),
                    ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("alpha", None), ("hidden", Some("YES")), ("tag", None),
                    ("userInteractionEnabled", Some("NO")), ("multipleTouchEnabled", Some("YES")), ("clearsContextBeforeDrawing", Some("NO")),
                    ("preservesSuperviewLayoutMargins", Some("YES")), ("insetsLayoutMarginsFromSafeArea", Some("NO")), ("restorationIdentifier", None),
                    ("layoutMarginsFollowReadableWidth", Some("YES")), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("id", None),
                    ("showsHorizontalScrollIndicator", Some("NO")), ("showsVerticalScrollIndicator", Some("NO")), ("alwaysBounceVertical", Some("YES")),
                    ("directionalLockEnabled", Some("YES")), ("minimumZoomScale", None), ("maximumZoomScale", None),
                ])?;
                if scroll && n.attribute("key").is_some() {
                    return fail("a scroll view as a table's header or footer view is not supported by this ibtool");
                }
                if !matches!(n.attribute("key"), None | Some("tableHeaderView" | "tableFooterView")) {
                    return fail(format!("key=\"{}\" on <view> is not supported by this ibtool", n.attribute("key").unwrap_or("")));
                }
                only_children(n, &[
                    "rect", "autoresizingMask", "color", "subviews", "constraints", "connections", "edgeInsets", "directionalEdgeInsets",
                    "userDefinedRuntimeAttributes", "accessibility",
                ])?;
                view_connections(scene, n, &name)?;
                let pairs = runtime_attributes(nib, n, &name)?;
                scene.pairs.extend(pairs.into_iter().map(|p| (name.clone(), p)));
                accessibility(nib, scene, n, &name)?;
            }
            let mut inner = Vec::new();
            for s in children(n, "subviews") {
                for c in s.children().filter(|c| c.is_element()) {
                    let outer = scene.parent.replace(name.clone());
                    let v = subview(nib, scene, c);
                    scene.parent = outer;
                    inner.push(v?);
                }
            }
            if !inner.is_empty() {
                values.push(("UISubviews", nib.array("NSMutableArray", inner)));
            }
            for c in children(n, "color") {
                if c.attribute("key") != Some("backgroundColor") {
                    return fail("view colours other than a background colour are not supported by this ibtool");
                }
                values.push(("UIBackgroundColor", color(nib, c)?));
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            if n.attribute("opaque").is_none() {
                values.push(("UIOpaque", Value::Bool(true)));
            }
            hidden_and_tag(&mut values, n)?;
            if n.attribute("userInteractionEnabled").is_some() {
                values.push(("UIUserInteractionDisabled", Value::Bool(true)));
            }
            if n.attribute("multipleTouchEnabled").is_some() {
                values.push(("UIMultipleTouchEnabled", Value::Bool(true)));
            }
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            let mask = match children(n, "autoresizingMask").next() {
                Some(m) => autoresizing_mask(m)?,
                None => 36,
            };
            values.push(("UIAutoresizingMask", Value::Int(mask)));
            alpha(&mut values, n)?;
            if n.attribute("clearsContextBeforeDrawing").is_some() {
                values.push(("UIClearsContextBeforeDrawing", Value::Bool(false)));
            }
            let mode = content_mode(n.attribute("contentMode").unwrap_or("scaleToFill"))?;
            if mode != 0 {
                values.push(("UIContentMode", Value::Int(mode)));
            }
            if n.attribute("clipsSubviews").is_some() {
                values.push(("UIClipsToBounds", Value::Bool(true)));
            }
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            margins(&mut values, n)?;
            restoration_identifier(nib, &mut values, n);
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            if scroll {
                values.push(("UIScrollViewIndicatorInsetAdjustmentBehavior", Value::Int(1)));
                if n.attribute("showsHorizontalScrollIndicator").is_some() {
                    values.push(("UIShowsHorizontalScrollIndicator", Value::Bool(false)));
                }
                if n.attribute("showsVerticalScrollIndicator").is_some() {
                    values.push(("UIShowsVerticalScrollIndicator", Value::Bool(false)));
                }
                values.push(("UIBouncesZoom", Value::Bool(true)));
                if n.attribute("alwaysBounceVertical").is_some() {
                    values.push(("UIAlwaysBounceVertical", Value::Bool(true)));
                }
                if n.attribute("directionalLockEnabled").is_some() {
                    values.push(("UIDirectionalLockEnabled", Value::Bool(true)));
                }
                for (a, key) in [("minimumZoomScale", "UIMinimumZoomScale"), ("maximumZoomScale", "UIMaximumZoomScale")] {
                    if let Some(z) = n.attribute(a) {
                        values.push((key, Value::Float(number(z)? as f32)));
                    }
                }
                values.push(("UIScrollViewContentInsetAdjustmentBehavior", Value::Int(0)));
            }
            let original = if scroll { "UIScrollView" } else { "UIView" };
            // A custom class is swapped in for the view when it loads.
            if n.attribute("customClass").is_some() {
                let class = runtime_class_name(n, scene.module, original)?;
                values.push(("UIClassName", nib.string(&class)));
                values.push(("UIOriginalClassName", nib.string(original)));
                Object::new("UIClassSwapper", values)
            } else {
                if n.attribute("customModule").is_some() || n.attribute("customModuleProvider").is_some_and(|p| !p.is_empty()) {
                    return fail("a custom module without a custom class is not supported by this ibtool");
                }
                Object::new(original, values)
            }
        }
        "switch" | "slider" | "stepper" | "activityIndicatorView" => simple_control(nib, scene, n, &name, &f, values)?,
        "imageView" => {
            only_attributes(n, &[
                ("clipsSubviews", Some("YES")), ("userInteractionEnabled", Some("NO")), ("contentMode", None),
                ("horizontalHuggingPriority", None), ("verticalHuggingPriority", None), ("image", None),
                ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("customClass", None), ("customModule", None), ("customModuleProvider", None),
                ("hidden", Some("YES")), ("tag", None), ("alpha", None), ("horizontalCompressionResistancePriority", None),
                ("verticalCompressionResistancePriority", None), ("adjustsImageSizeForAccessibilityContentSizeCategory", Some("YES")),
                ("fixedFrame", Some("YES")), ("id", None),
            ])?;
            only_children(n, &["rect", "autoresizingMask", "color", "constraints", "connections", "userDefinedRuntimeAttributes", "accessibility"])?;
            view_connections(scene, n, &name)?;
            let pairs = runtime_attributes(nib, n, &name)?;
            scene.pairs.extend(pairs.into_iter().map(|p| (name.clone(), p)));
            accessibility(nib, scene, n, &name)?;

            let content_mode = match attr(n, "contentMode")? {
                "scaleToFill" => 0,
                "scaleAspectFit" => 1,
                "scaleAspectFill" => 2,
                "center" => 4,
                other => return fail(format!("contentMode {other} on <imageView> is not supported by this ibtool")),
            };
            let mut tint = None;
            for c in children(n, "color") {
                match c.attribute("key") {
                    Some("backgroundColor") => values.push(("UIBackgroundColor", color(nib, c)?)),
                    Some("tintColor") => tint = Some(color(nib, c)?),
                    _ => return fail("image view colours other than a background or tint colour are not supported by this ibtool"),
                }
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIOpaque", Value::Bool(true)));
            hidden_and_tag(&mut values, n)?;
            if n.attribute("userInteractionEnabled").is_some() {
                values.push(("UIUserInteractionDisabled", Value::Bool(true)));
            }
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            // Its autoresizing mask as given, whether or not it translates
            // it (three probes).
            let mask = match children(n, "autoresizingMask").next() {
                Some(m) => autoresizing_mask(m)?,
                None => 36,
            };
            values.push(("UIAutoresizingMask", Value::Int(mask)));
            alpha(&mut values, n)?;
            if content_mode != 0 {
                values.push(("UIContentMode", Value::Int(content_mode)));
            }
            if n.attribute("clipsSubviews").is_some() {
                values.push(("UIClipsToBounds", Value::Bool(true)));
            }
            if let Some(tint) = tint {
                values.push(("UITintColor", tint));
            }
            if does_not_translate(scene, n) {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            if let Some(own) = own_constraints(nib, scene, n, &name)? {
                values.push(("UIViewAutolayoutConstraints", own));
            }
            if n.attribute("horizontalHuggingPriority").is_some() || n.attribute("verticalHuggingPriority").is_some() {
                values.push(("UIViewContentHuggingPriority", hugging(nib, n)?));
            }
            // Compression resistance, each 750 unless given (one probe).
            let resistance = (n.attribute("horizontalCompressionResistancePriority"), n.attribute("verticalCompressionResistancePriority"));
            if resistance != (None, None) {
                let (h, v) = (resistance.0.unwrap_or("750"), resistance.1.unwrap_or("750"));
                values.push(("UIViewContentCompressionResistancePriority", nib.string(&format!("{{{h}, {v}}}"))));
            }
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            if let Some(image) = n.attribute("image") {
                values.push(("UIImage", nib.image(image)));
            }
            // One probe.
            if n.attribute("adjustsImageSizeForAccessibilityContentSizeCategory").is_some() {
                values.push(("UIAdjustsImageSizeForAccessibilityContentSizeCategory", Value::Bool(true)));
            }
            if n.attribute("customClass").is_some() {
                let class = runtime_class_name(n, scene.module, "UIImageView")?;
                values.push(("UIClassName", nib.string(&class)));
                values.push(("UIOriginalClassName", nib.string("UIImageView")));
                Object::new("UIClassSwapper", values)
            } else {
                Object::new("UIImageView", values)
            }
        }
        other => return fail(format!("<{other}> is not supported by this ibtool")),
    };
    Ok(Value::New(object.named(name)))
}

/// References to the views just built, in order.
fn arranged_names(views: &[Value]) -> Vec<Value> {
    views
        .iter()
        .map(|v| match v {
            Value::New(o) => reference(o.name.clone().expect("subviews are named")),
            _ => unreachable!("subviews are new objects"),
        })
        .collect()
}

/// The size Xcode lays a root view out at for a `<device>`, whatever the
/// view's frame says, or None for ids it doesn't know, where the frame is
/// used: measured per id and orientation (docs/clean-room/ibtool.md). Ids
/// not measured are refused.
fn device_size(platform: Platform, id: &str, orientation: &str) -> Result<Option<(f64, f64)>, String> {
    const PORTRAIT: [(&str, f64, f64); 17] = [
        ("retina4_0", 320.0, 568.0), ("retina4_7", 375.0, 667.0), ("retina5_5", 414.0, 736.0), ("retina5_9", 375.0, 812.0),
        ("retina6_0", 390.0, 844.0), ("retina6_1", 414.0, 896.0), ("retina6_12", 393.0, 852.0), ("retina6_3", 402.0, 874.0),
        ("retina6_5", 414.0, 896.0), ("retina6_7", 430.0, 932.0), ("retina6_9", 440.0, 956.0), ("ipad7_9", 744.0, 1133.0),
        ("ipad9_7", 768.0, 1024.0), ("ipad10_2", 810.0, 1080.0), ("ipad10_9rounded", 820.0, 1180.0),
        ("ipad11_0rounded", 834.0, 1210.0), ("ipad12_9rounded", 1024.0, 1366.0),
    ];
    // Laid out the same whichever the orientation.
    const FIXED: [&str; 2] = ["retina3_5", "retina6_72"];
    // Not known to Xcode's ibtool: the view's frame is used.
    const UNKNOWN: [&str; 5] = ["retina5_4", "ipad10_5", "ipad12_9", "ipad8_3rounded", "ipad13_0rounded"];
    if orientation != "portrait" && orientation != "landscape" {
        return fail(format!("device orientation {orientation} is not supported by this ibtool"));
    }
    if platform == Platform::Tvos {
        return match (id, orientation) {
            ("appleTV", "landscape") => Ok(Some((1920.0, 1080.0))),
            ("appleTV4K", _) => Ok(None),
            _ => fail(format!("device {id} ({orientation}) is not supported by this ibtool")),
        };
    }
    if FIXED.contains(&id) {
        return Ok(Some((393.0, 852.0)));
    }
    if UNKNOWN.contains(&id) {
        return Ok(None);
    }
    let &(_, w, h) = PORTRAIT.iter().find(|(d, ..)| *d == id).ok_or_else(|| format!("device {id} is not supported by this ibtool"))?;
    Ok(Some(if orientation == "landscape" { (h, w) } else { (w, h) }))
}

/// Where Xcode puts a root view's constraint (docs/clean-room/ibtool.md):
/// grouped by the item other than the root view, or the first item between
/// siblings, the groups by where their item is (below); in a group,
/// constraints to a view deeper than the root's subviews (`nested` second
/// items) first, then those with the root view, then those between
/// siblings. Those with the root view go by kind (horizontal centres,
/// vertical centres, left and right edges, horizontal edges with a leading
/// or trailing one, then the rest), then by first and second attribute in
/// one order, where leading ties with left and trailing with right, and
/// between a leading and a trailing edge the one whose root view's edge is
/// trailing first (a 272-constraint probe and 40 random ones). Those to
/// deeper views and between siblings go by first and second attribute in
/// another order. Equal keys are refused: Xcode leaves them in the
/// document's order, which it doesn't keep stably.
fn constraint_key<'a>(root: &str, first: &'a str, a1: &str, second: &'a str, a2: &str, nested: bool) -> Result<(&'a str, u8, [usize; 4]), String> {
    // The last baseline (`baseline`) after the first (three probes).
    const SIBLINGS: [&str; 20] = [
        "centerXWithinMargins", "centerX", "rightMargin", "leftMargin", "right", "left", "trailingMargin", "leadingMargin", "trailing", "leading",
        "centerYWithinMargins", "centerY", "topMargin", "bottomMargin", "bottom", "top", "firstBaseline", "baseline", "width", "height",
    ];
    let rank = |order: &[&str], a: &str| order.iter().position(|o| *o == a).ok_or_else(|| format!("constraints on {a} are not supported by this ibtool"));
    let axis = |a: &str| match a {
        "leading" | "trailing" | "centerX" | "left" | "right" | "leadingMargin" | "trailingMargin" | "leftMargin" | "rightMargin"
        | "centerXWithinMargins" => 0,
        "top" | "bottom" | "centerY" | "topMargin" | "bottomMargin" | "centerYWithinMargins" | "firstBaseline" | "baseline" => 1,
        _ => 2,
    };
    if a1 != a2 && (axis(a1) != axis(a2) || axis(a1) == 2) {
        return fail(format!("constraints between {a1} and {a2} are not supported by this ibtool"));
    }
    // Xcode rewrites a left or right edge tied to a leading or trailing one.
    let absolute = |a: &str| ["left", "right", "leftMargin", "rightMargin"].contains(&a);
    let directional = |a: &str| ["leading", "trailing", "leadingMargin", "trailingMargin"].contains(&a);
    if (absolute(a1) && directional(a2)) || (directional(a1) && absolute(a2)) {
        return fail(format!("constraints between {a1} and {a2} are not supported by this ibtool"));
    }
    let (group, kind, attributes) = if first == root || second == root {
        (if first == root { second } else { first }, 1, with_root_key(a1, a2, first == root)?)
    } else {
        (first, if nested { 0 } else { 2 }, [rank(&SIBLINGS, a1)?, rank(&SIBLINGS, a2)?, 0, 0])
    };
    Ok((group, kind, attributes))
}

/// Puts constraints of the same key (the same first item and attributes,
/// differing in their second items) in Xcode's order, for constraints of no
/// constant, multiplier, relation or priority to sibling views: by id where
/// the second items' frames are the same (three of the corpus's); for
/// centres, by those frames from the bottom, then from the right (19
/// probes). Others are refused: with a constant the order depends on the
/// ids too, and other attributes are ordered otherwise (top to bottom by
/// the bottom edge, leading to trailing by the trailing edge from the right,
/// eight probes, neither settled).
fn order_ties<'a, 'i, T>(v: &mut [T], of: impl Fn(&T) -> ((&'a str, u8, [usize; 4]), Node<'a, 'i>), siblings: &[Node<'a, 'i>]) -> Result<(), String> {
    let unstable = || fail("two constraints Xcode orders unstably (the same items and attributes) are not supported by this ibtool");
    let mut start = 0;
    while start < v.len() {
        let key = of(&v[start]).0;
        let end = start + v[start..].iter().take_while(|t| of(t).0 == key).count();
        if end - start > 1 {
            let mut run = Vec::new();
            for t in &v[start..end] {
                let c = of(t).1;
                if ["multiplier", "relation", "priority"].iter().any(|a| c.attribute(*a).is_some()) || c.attribute("constant").is_some_and(|k| k.parse::<f64>() != Ok(0.0)) {
                    return unstable();
                }
                let second = c.attribute("secondItem").and_then(|id| siblings.iter().find(|s| s.attribute("id") == Some(id)));
                let Some(second) = second else { return unstable() };
                if run.iter().any(|(_, _, s): &(Frame, &str, &str)| Some(*s) == second.attribute("id")) {
                    return unstable();
                }
                let f = frame(*second)?;
                run.push((f, c.attribute("id").unwrap_or(""), second.attribute("id").unwrap_or("")));
            }
            let c = of(&v[start]).1;
            let same = run.iter().all(|(f, ..)| (f.x, f.y, f.w, f.h) == (run[0].0.x, run[0].0.y, run[0].0.w, run[0].0.h));
            let centres = ["centerX", "centerY"].contains(&c.attribute("firstAttribute").unwrap_or("")) && c.attribute("firstAttribute") == c.attribute("secondAttribute");
            if !(same && c.attribute("firstAttribute") == c.attribute("secondAttribute")) && !centres {
                return unstable();
            }
            for (i, (a, ..)) in run.iter().enumerate() {
                if run[..i].iter().any(|(b, ..)| (b.x, b.y) == (a.x, a.y) && (b.w, b.h) != (a.w, a.h)) {
                    return unstable();
                }
            }
            let mut order: Vec<usize> = (0..run.len()).collect();
            order.sort_by(|&i, &j| run[j].0.y.total_cmp(&run[i].0.y).then(run[j].0.x.total_cmp(&run[i].0.x)).then(run[i].1.cmp(run[j].1)));
            let rank: Vec<&str> = order.iter().map(|&i| run[i].1).collect();
            v[start..end].sort_by_key(|t| rank.iter().position(|id| of(t).1.attribute("id") == Some(*id)));
        }
        start = end;
    }
    Ok(())
}

/// The order of a constraint between a root view and another item, from its
/// attributes and whether the root view is its first item: see
/// `constraint_key`.
fn with_root_key(a1: &str, a2: &str, root_first: bool) -> Result<[usize; 4], String> {
    // Margins and baselines placed by two probes of every attribute.
    const WITH_ROOT: [&str; 19] = [
        "centerXWithinMargins", "centerX", "centerYWithinMargins", "centerY", "rightMargin", "leftMargin", "right", "left", "trailingMargin",
        "leadingMargin", "trailing", "leading", "width", "topMargin", "bottomMargin", "bottom", "top", "firstBaseline", "height",
    ];
    // Leading and trailing edges, each tied with its left or right one.
    const DIRECTIONAL: [(&str, &str); 4] = [("trailingMargin", "rightMargin"), ("leadingMargin", "leftMargin"), ("trailing", "right"), ("leading", "left")];
    let rank = |a: &str| WITH_ROOT.iter().position(|o| *o == a).ok_or_else(|| format!("constraints on {a} are not supported by this ibtool"));
    let (r1, r2) = (rank(a1)?, rank(a2)?);
    let either = |set: &[&str]| set.contains(&a1) || set.contains(&a2);
    let directional = |a: &str| DIRECTIONAL.iter().position(|(d, lr)| *d == a || *lr == a);
    let trailing = |a: &str| directional(a).is_some_and(|i| i % 2 == 0);
    Ok(if either(&["centerXWithinMargins", "centerX"]) {
        [0, r1, r2, 0]
    } else if either(&["centerYWithinMargins", "centerY"]) {
        [1, r1, r2, 0]
    } else if [a1, a2].iter().all(|a| ["rightMargin", "leftMargin", "right", "left"].contains(a)) {
        [2, r1, r2, 0]
    } else if let (Some(d1), Some(d2)) = (directional(a1), directional(a2)) {
        let root_edge = if root_first { a1 } else { a2 };
        // Edges on one side tie whichever way round.
        let side = usize::from(trailing(a1) != trailing(a2) && !trailing(root_edge));
        [3, d1, d2, side]
    } else if a1 == a2 || [a1, a2].iter().all(|a| ["topMargin", "bottomMargin", "bottom", "top"].contains(a)) {
        [4, r1, r2, 0]
    } else {
        return fail(format!("constraints between {a1} and {a2} with a root view are not supported by this ibtool"));
    })
}

/// How far a constraint's legacy constant moves for an item's margin
/// attribute (`direction` from `attribute_codes`): the item's own layout
/// margin on that edge when its document gives it margins (leading as left
/// and trailing as right), otherwise 8. A centre within margins that aren't
/// even is refused: Xcode's offset for it isn't half their difference.
fn margin_offset(c: Node, item: Option<&str>, attribute: &str, direction: i64) -> Result<f64, String> {
    // The item: given, or the view holding the constraint.
    let view = match item {
        Some(id) => c.document().descendants().find(|d| d.is_element() && d.attribute("id") == Some(id)),
        None => c.parent().and_then(|cs| cs.parent()),
    };
    let Some((m, _)) = view.map(layout_margins).transpose()?.flatten() else { return Ok(8.0 * direction as f64) };
    Ok(match attribute {
        "topMargin" => m[0],
        "leftMargin" | "leadingMargin" => m[1],
        "bottomMargin" => -m[2],
        "rightMargin" | "trailingMargin" => -m[3],
        "centerXWithinMargins" if m[1] == m[3] => 0.0,
        "centerYWithinMargins" if m[0] == m[2] => 0.0,
        "centerXWithinMargins" | "centerYWithinMargins" => return fail("a centre within uneven layout margins is not supported by this ibtool"),
        _ => 0.0,
    })
}

/// A constraint of the document's: its attributes (a margin's own code,
/// then for the legacy keys the edge's, its constant moved by the item's
/// margin), relation, multiplier, constant (a symbolic one is `NSSpace`)
/// and priority.
fn document_constraint(nib: &mut Nib, c: Node, first: Value, second: Option<Value>) -> Result<Value, String> {
    let (v1, l1, s1) = attribute_codes(attr(c, "firstAttribute")?)?;
    let mut values = vec![("NSFirstItem", first), ("NSFirstAttributeV2", Value::Int(v1)), ("NSFirstAttribute", Value::Int(l1))];
    match c.attribute("relation") {
        None => {}
        Some("greaterThanOrEqual") => values.push(("NSRelation", Value::Int(1))),
        Some("lessThanOrEqual") => values.push(("NSRelation", Value::Int(-1))),
        Some(other) => return fail(format!("constraint relation {other} is not supported by this ibtool")),
    }
    if let Some(m) = multiplier(c)? {
        values.push(("NSMultiplier", Value::Double(m)));
    }
    let mut margin = -margin_offset(c, c.attribute("firstItem"), attr(c, "firstAttribute")?, s1)?;
    if let Some(second) = second {
        let (v2, l2, s2) = attribute_codes(attr(c, "secondAttribute")?)?;
        values.extend([("NSSecondItem", second), ("NSSecondAttributeV2", Value::Int(v2)), ("NSSecondAttribute", Value::Int(l2))]);
        margin += margin_offset(c, Some(attr(c, "secondItem")?), attr(c, "secondAttribute")?, s2)?;
    }
    let symbolic = c.attribute("symbolic").is_some();
    let constant = if symbolic { 0.0 } else { c.attribute("constant").map(number).transpose()?.unwrap_or(0.0) };
    if constant != 0.0 {
        values.push(("NSConstantV2", Value::Double(constant)));
    }
    if symbolic {
        values.push(("NSSymbolicConstant", nib.string("NSSpace")));
    }
    values.push(("NSShouldBeArchived", Value::Bool(true)));
    if let Some(p) = c.attribute("priority") {
        let p: i64 = p.parse().map_err(|_| format!("constraint priority {p} is not supported by this ibtool"))?;
        if p != 1000 {
            values.push(("NSPriority", Value::Int(p)));
        }
    }
    if let Some(identifier) = c.attribute("identifier") {
        values.push(("NSLayoutIdentifier", nib.string(identifier)));
    }
    if constant + margin != 0.0 {
        values.push(("NSConstant", Value::Double(constant + margin)));
    }
    Ok(Value::New(Object::new("NSLayoutConstraint", values)))
}

/// A layout attribute's code, its edge's code for the legacy keys, and its
/// margin's direction (1 inward from the leading, left or top, -1 from the
/// trailing, right or bottom, 0 without a margin).
fn attribute_codes(name: &str) -> Result<(i64, i64, i64), String> {
    Ok(match name {
        "leftMargin" => (13, 1, 1),
        "rightMargin" => (14, 2, -1),
        "topMargin" => (15, 3, 1),
        "bottomMargin" => (16, 4, -1),
        "leadingMargin" => (17, 5, 1),
        "trailingMargin" => (18, 6, -1),
        "centerXWithinMargins" => (19, 9, 0),
        "centerYWithinMargins" => (20, 10, 0),
        "firstBaseline" => (12, 12, 0),
        "baseline" => (11, 11, 0),
        other => {
            let code = layout_attribute(other)?;
            (code, code, 0)
        }
    })
}

fn constraint(first: Value, first_attr: i64, second: Option<(Value, i64)>, constant: f64, priority: Option<i64>) -> Value {
    let mut values = vec![("NSFirstItem", first), ("NSFirstAttributeV2", Value::Int(first_attr)), ("NSFirstAttribute", Value::Int(first_attr))];
    if let Some((item, a)) = second {
        values.push(("NSSecondItem", item));
        values.push(("NSSecondAttributeV2", Value::Int(a)));
        values.push(("NSSecondAttribute", Value::Int(a)));
    }
    if constant != 0.0 {
        values.push(("NSConstantV2", Value::Double(constant)));
    }
    values.push(("NSShouldBeArchived", Value::Bool(true)));
    if constant != 0.0 {
        values.push(("NSConstant", Value::Double(constant)));
    }
    if let Some(p) = priority {
        values.push(("NSPriority", Value::Int(p)));
    }
    Value::New(Object::new("NSLayoutConstraint", values))
}

/// The root view's safe-area layout guide, as archived where a constraint
/// first uses it: owned by the view, with the system constraints that pin
/// its edges to the view's.
fn safe_area_guide(nib: &mut Nib, root: &str) -> Value {
    let identifier = nib.string("UIViewSafeAreaLayoutGuide");
    let me = || reference("safe area guide");
    let view = || reference(root.to_string());
    // NSLayoutAttribute: left 1, right 2, top 3, bottom 4.
    let pins = [("top", me(), 3, view()), ("left", me(), 1, view()), ("bottom", view(), 4, me()), ("right", view(), 2, me())];
    let mut system = Vec::new();
    for (edge, first, a, second) in pins {
        let name = nib.string(&format!("UIViewSafeAreaLayoutGuide-{edge}"));
        system.push(Value::New(Object::new("NSLayoutConstraint", vec![
            ("NSFirstItem", first),
            ("NSFirstAttributeV2", Value::Int(a)),
            ("NSFirstAttribute", Value::Int(a)),
            ("NSSecondItem", second),
            ("NSSecondAttributeV2", Value::Int(a)),
            ("NSSecondAttribute", Value::Int(a)),
            ("NSShouldBeArchived", Value::Bool(false)),
            ("NSLayoutIdentifier", name),
        ])));
    }
    let system = nib.array("NSArray", system);
    Value::New(Object::new("UILayoutGuide", vec![
        ("UILayoutGuideOwningView", view()),
        ("UILayoutGuideIdentifier", identifier),
        ("UILayoutGuideOwningViewIsLocked", Value::Bool(true)),
        ("UILayoutGuideShouldBeArchived", Value::Bool(true)),
        ("UILayoutGuideAllowsNegativeDimensions", Value::Bool(true)),
        ("UILayoutGuideSystemConstraints", system),
    ]).named("safe area guide"))
}

/// A constraint's multiplier, as Xcode archives it: a float's value (a
/// ratio `a:b` divided), or None for 1.
fn multiplier(c: Node) -> Result<Option<f64>, String> {
    let Some(m) = c.attribute("multiplier") else { return Ok(None) };
    // Interface Builder writes a ratio with either separator.
    let value = match m.split_once([':', '/']) {
        Some((a, b)) => number(a)? / number(b)?,
        None => number(m)?,
    };
    Ok(if value == 1.0 { None } else { Some(value as f32 as f64) })
}


/// A view controller's legacy top or bottom layout guide: a hidden view with
/// a width constraint UIKit replaces, and the constraints that pin it.
fn legacy_guide(nib: &mut Nib, root: &str, kind: &'static str) -> (Value, Vec<String>) {
    let name = format!("guide {kind}");
    let mut values = vec![
        ("UIDeepDrawRect", Value::Bool(true)),
        ("UIOpaque", Value::Bool(true)),
        ("UIHidden", Value::Bool(true)),
        ("UIAutoresizeSubviews", Value::Bool(true)),
        ("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)),
    ];
    let support = match constraint(reference(name.clone()), 7, None, 0.0, Some(999)) {
        Value::New(o) => Value::New(Object { class: "_UILayoutSupportConstraint", ..o }.with_fallback("NSLayoutConstraint")),
        _ => unreachable!(),
    };
    values.push(("UIViewAutolayoutConstraints", nib.array("NSMutableArray", vec![support])));
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    let identifier = if kind == "top" { "_UIViewControllerTop" } else { "_UIViewControllerBottom" };
    values.push(("_UILayoutGuideIdentifier", nib.string(identifier)));
    let me = || reference(name.clone());
    let view = || reference(root.to_string());
    let pins = if kind == "top" {
        vec![constraint(me(), 5, Some((view(), 5)), 0.0, None), constraint(me(), 3, Some((view(), 3)), 0.0, None), constraint(me(), 8, None, 0.0, None)]
    } else {
        vec![constraint(me(), 5, Some((view(), 5)), 0.0, None), constraint(me(), 8, None, 0.0, None), constraint(view(), 4, Some((me(), 4)), 0.0, None)]
    };
    let mut names = Vec::new();
    let named: Vec<Value> = pins
        .into_iter()
        .map(|p| match p {
            Value::New(o) => {
                let n = nib.fresh();
                names.push(n.clone());
                Value::New(o.named(n))
            }
            _ => unreachable!(),
        })
        .collect();
    values.push(("_UILayoutGuideConstraintsToRemove", nib.array("NSArray", named)));
    let guide = Object::new("_UILayoutGuide", values).with_fallback("UIView").named(name);
    (Value::New(guide), names)
}

fn proxy(nib: &mut Nib, id: &str, name: &str) -> Value {
    let identifier = nib.string(id);
    Value::New(Object::new("UIProxyObject", vec![("UIProxiedObjectIdentifier", identifier)]).named(name))
}

fn outlet(nib: &mut Nib, label: &str, source: &str, destination: &str) -> Value {
    let label = nib.string(label);
    Value::New(Object::new("UIRuntimeOutletConnection", vec![("UILabel", label), ("UISource", reference(source)), ("UIDestination", reference(destination))]))
}

/// An outlet collection: `destinations` in a mutable array, the class the
/// property is filled with, and whether it adds to an existing collection.
fn outlet_collection(nib: &mut Nib, label: &str, source: &str, destinations: Vec<Value>, class: &str, adds: bool) -> Value {
    let label = nib.string(label);
    let collection = nib.array("NSMutableArray", destinations);
    let class = nib.string(class);
    Value::New(Object::new("UIRuntimeOutletCollectionConnection", vec![
        ("UILabel", label),
        ("UISource", reference(source)),
        ("UIDestination", collection),
        ("runtimeCollectionClassName", class),
        ("addsContentToExistingCollection", Value::Bool(adds)),
    ]))
}

/// A navigation item's bar button items' actions and segues' `perform:`,
/// by selector: actions with one selector the right item's first (`items`
/// in the left-right order), the `perform:`s in their templates' order.
fn bar_events(nib: &mut Nib, items: &[String], actions: &[(String, String)], performs: &[(String, String)], controller: &str) -> Vec<Value> {
    let mut events: Vec<(&str, usize, &str, &str)> = Vec::new();
    for (selector, item) in actions {
        let from_right = items.len() - items.iter().position(|i| i == item).unwrap_or(0);
        events.push((selector, from_right, item, controller));
    }
    events.extend(performs.iter().enumerate().map(|(i, (item, template))| ("perform:", i, item.as_str(), template.as_str())));
    events.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    events.into_iter().map(|(selector, _, item, destination)| event_connection(nib, selector, item, destination)).collect()
}

/// An action without an event mask: a gesture recognizer's or a bar button
/// item's.
/// An action's event, as UIControlEvents' mask (13 probes): Xcode writes
/// `menuActionTriggered` with none, so it is refused.
fn control_event(event: &str) -> Result<i64, String> {
    Ok(match event {
        "touchDown" => 1,
        "touchDownRepeat" => 2,
        "touchDragInside" => 4,
        "touchDragOutside" => 8,
        "touchDragEnter" => 16,
        "touchDragExit" => 32,
        "touchUpInside" => 64,
        "touchUpOutside" => 128,
        "touchCancel" => 256,
        "valueChanged" => 4096,
        "primaryActionTriggered" => 8192,
        other => return fail(format!("a control's {other} action is not supported by this ibtool")),
    })
}

/// The placeholder a control's action goes to: the scene's view controller
/// (upstream) or its first responder.
fn action_target(scene: &Scene, a: Node) -> Result<&'static str, String> {
    let destination = attr(a, "destination")?;
    if destination == scene.vc_id {
        return Ok("upstream");
    }
    let responder = a
        .ancestors()
        .find(|n| n.has_tag_name("scene"))
        .and_then(|sc| sc.descendants().find(|d| d.has_tag_name("placeholder") && d.attribute("placeholderIdentifier") == Some("IBFirstResponder")));
    if responder.and_then(|r| r.attribute("id")) == Some(destination) {
        return Ok("responder");
    }
    fail("actions that target anything but the scene's view controller or first responder are not supported by this ibtool")
}

fn event_connection(nib: &mut Nib, selector: &str, source: &str, destination: &str) -> Value {
    let label = nib.string(selector);
    Value::New(Object::new("UIRuntimeEventConnection", vec![("UILabel", label), ("UISource", reference(source)), ("UIDestination", reference(destination))]))
}

fn root_object(nib: &mut Nib, top: Vec<Value>, objects: Vec<Value>, connections: Vec<Value>) -> Object {
    root_object_with_pairs(nib, top, objects, connections, Vec::new())
}

/// A nib's root object with key-value pairs: values set by key path when
/// the nib loads.
fn root_object_with_pairs(nib: &mut Nib, top: Vec<Value>, objects: Vec<Value>, connections: Vec<Value>, pairs: Vec<Value>) -> Object {
    root_object_full(nib, top, objects, connections, Vec::new(), pairs)
}

/// A nib's root object with accessibility configurations and key-value
/// pairs.
fn root_object_full(nib: &mut Nib, top: Vec<Value>, objects: Vec<Value>, connections: Vec<Value>, accessibility: Vec<Value>, pairs: Vec<Value>) -> Object {
    let top = nib.array("NSArray", top);
    let objects = nib.array("NSArray", objects);
    // No connections are the nib's one empty array.
    let connections = if connections.is_empty() { nib.empty_array() } else { nib.array("NSArray", connections) };
    Object::new("NSObject", vec![
        ("UINibTopLevelObjectsKey", top),
        ("UINibObjectsKey", objects),
        ("UINibConnectionsKey", connections),
        ("UINibVisibleWindowsKey", nib.empty_array()),
        ("UINibAccessibilityConfigurationsKey", if accessibility.is_empty() { nib.empty_array() } else { nib.array("NSArray", accessibility) }),
        ("UINibTraitStorageListsKey", nib.empty_array()),
        ("UINibKeyValuePairsKey", if pairs.is_empty() { nib.empty_array() } else { nib.array("NSArray", pairs) }),
        ("IBUINibLNEVersionKey", Value::Int(1)),
    ])
}

/// What a navigation controller's nib needs of its root view controller,
/// which it embeds.
struct RootController<'a, 'i> {
    vc: Node<'a, 'i>,
    /// Whether it has a nib of its own too (initial or identified).
    loadable: bool,
    view_nib_name: String,
    class_name: String,
    /// UIViewController or UITableViewController.
    original_class: &'static str,
    upstream_used: bool,
    /// The segues its view's controls trigger, as listed with their
    /// placeholders' indices, and its own.
    view_segues: Vec<(usize, Node<'a, 'i>)>,
    own_segues: Vec<Node<'a, 'i>>,
    /// Its scene's gesture recognizers in document order, and their ids in
    /// their placeholders' order.
    recognizers: Vec<Node<'a, 'i>>,
    gesture_order: Vec<String>,
}

/// The id of a navigation controller's root view controller, its one
/// relationship segue.
fn navigation_root<'a>(nav: Node<'a, '_>) -> Result<&'a str, String> {
    only_children(nav, &["navigationBar", "connections", "toolbarItems", "nil", "tabBarItem"])?;
    for c in children(nav, "toolbarItems") {
        if c.has_children() {
            return fail("navigation controllers with toolbar items are not supported by this ibtool");
        }
    }
    // Simulated metrics are Interface Builder's alone (one probe).
    for c in children(nav, "nil").filter(|c| !c.attribute("key").is_some_and(|k| k.starts_with("simulated"))) {
        if c.attribute("name") != Some("viewControllers") {
            return fail("<nil> inside <navigationController> is not supported by this ibtool");
        }
    }
    let connections = children(nav, "connections").next().ok_or("a navigation controller without a root view controller")?;
    only_children(connections, &["segue"])?;
    let segues: Vec<Node> = children(connections, "segue").collect();
    if segues.len() != 1 {
        return fail("navigation controllers with segues other than their root's are not supported by this ibtool");
    }
    only_attributes(segues[0], &[("destination", None), ("kind", Some("relationship")), ("relationship", Some("rootViewController")), ("id", None)])?;
    attr(segues[0], "destination")
}

/// A controller a container's nib holds: a navigation controller's root
/// or a tab bar controller's child (`name` its object's name, `parent` its
/// container's), as in its own nib, with a tab bar item before its
/// navigation item, which refers to `bar` in a navigation controller.
struct Embedded {
    value: Value,
    templates: segue::Templates,
    /// The name of the navigation item the document gives it, if any.
    navigation_item: Option<String>,
    /// Its navigation item's bar button items, after the templates, and
    /// their actions (selector, item), after the templates' outlets.
    bar_items: Vec<String>,
    bar_actions: Vec<(String, String)>,
    /// Its gesture recognizers' names, in reverse document order, and their
    /// actions (selector, recognizer) by selector.
    gestures: Vec<String>,
    gesture_actions: Vec<(String, String)>,
}

#[allow(clippy::too_many_arguments)]
fn embedded_controller(
    nib: &mut Nib,
    root: &RootController,
    name: &str,
    parent: &str,
    bar: Option<&str>,
    tab_item: Option<Value>,
    nib_names: &HashMap<String, String>,
) -> Result<Embedded, String> {
    let vc = root.vc;
    if vc.attribute("restorationIdentifier").is_some() {
        return fail("an embedded controller's restoration identifier is not supported by this ibtool");
    }
    // Its title is encoded before its navigation item, which may share the
    // string.
    let title = match vc.attribute("title") {
        Some(title) => Some(nib.text(attr(vc, "id")?, "title", title)),
        None => None,
    };
    let explicit_item = child(vc, "navigationItem", "navigationItem");
    let item_name = format!("{name} navigation item");
    let mut bar_items = Vec::new();
    let mut bar_actions = Vec::new();
    let mut bar_segues = Vec::new();
    let item = match (explicit_item, bar) {
        (Some(i), _) => {
            let parts = navigation_item(nib, i, &item_name, attr(vc, "id")?)?;
            bar_items = parts.bar_items;
            bar_actions = parts.actions;
            bar_segues = parts.segues;
            match parts.value {
                Value::New(mut o) => {
                    if let Some(bar) = bar {
                        o.values.push(("UINavigationBar", reference(bar)));
                    }
                    Some(Value::New(o))
                }
                _ => unreachable!(),
            }
        }
        (None, Some(bar)) => Some(Value::New(
            Object::new("UINavigationItem", vec![("UILargeTitleDisplayMode", Value::Int(0)), ("UINavigationBar", reference(bar))])
                .named(format!("{name} implied navigation item")),
        )),
        (None, None) => None,
    };
    let gesture_count = root.recognizers.len();
    let mut templates = segue::templates(nib, name, root.upstream_used, 1 + gesture_count, &root.view_segues, &root.own_segues, &bar_segues, nib_names)?;
    // Its gesture recognizers, defined in its external objects table, in
    // their placeholders' order (two probes).
    let mut gestures = Vec::new();
    let mut gesture_actions = Vec::new();
    if gesture_count > 0 {
        if templates.array.is_some() || !bar_items.is_empty() || extended_edges(vc)?.is_some() || vc.attribute("automaticallyAdjustsScrollViewInsets").is_some() {
            return fail("an embedded controller's gesture recognizers beside segues, bar button items or these settings are not supported by this ibtool");
        }
        let mut defined = Vec::new();
        for r in &root.recognizers {
            let id = attr(*r, "id")?;
            let (object, selectors) = gesture_recognizer(nib, *r, attr(vc, "id")?)?;
            defined.push((id, format!("{name} gesture {id}"), object, selectors));
        }
        let mut entries = Vec::new();
        for (i, g) in root.gesture_order.iter().enumerate() {
            let k = defined.iter().position(|d| d.0 == g).expect("recognized above");
            let (_, gesture_name, object, _) = &mut defined[k];
            entries.push(nib.string(&format!("UpstreamPlaceholder-{}", i + 1)));
            entries.push(Value::New(std::mem::replace(object, Object::new("", Vec::new())).named(gesture_name.clone())));
        }
        templates.external.splice(0..0, entries);
        for (_, gesture_name, _, selectors) in &defined {
            gesture_actions.extend(selectors.iter().map(|s| (s.clone(), gesture_name.clone())));
        }
        gesture_actions.sort_by(|a, b| a.0.cmp(&b.0));
        gestures = defined.iter().rev().map(|d| d.1.clone()).collect();
    }
    // An empty table is one object however many controllers have one.
    let external = if templates.external.is_empty() {
        nib.shared("empty dictionary".into(), || Object::new("NSDictionary", vec![("NSInlinedValue", Value::Bool(true))]))
    } else {
        nib.array("NSDictionary", std::mem::take(&mut templates.external))
    };
    let nib_name = nib.string(&root.view_nib_name);
    let mut values = Vec::new();
    if let Some(title) = title {
        values.push(("UITitle", title));
    }
    if let Some(tab_item) = tab_item {
        values.push(("UITabBarItem", tab_item));
    }
    if let Some(item) = item {
        values.push(("UINavigationItem", item));
    }
    values.extend([("UIExternalObjectsTableForViewLoading", external), ("UINibName", nib_name)]);
    if let Some(identifier) = vc.attribute("storyboardIdentifier") {
        values.push(("UIStoryboardIdentifier", nib.string(identifier)));
        if vc.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
            values.push(("UIRestorationIdentifier", nib.string(identifier)));
        }
    }
    values.push(("UIParentViewController", reference(parent)));
    if gesture_count > 0 {
        let alive = root.recognizers.iter().map(|r| attr(*r, "id").map(|id| reference(format!("{name} gesture {id}")))).collect::<Result<_, _>>()?;
        values.push(("UITopLevelObjectsToKeepAliveFromStoryboard", nib.array("NSArray", alive)));
    }
    if let Some(array) = templates.array.take() {
        values.push(("UIStoryboardSegueTemplates", array));
    }
    if let Some(edges) = extended_edges(vc)? {
        values.push(("UIKeyEdgesForExtendedLayout", Value::Int(edges)));
    }
    if vc.attribute("automaticallyAdjustsScrollViewInsets").is_some() {
        values.push(("UIKeyAutomaticallyAdjustsScrollViewInsets", Value::Int(0)));
    }
    if vc.attribute("clearsSelectionOnViewWillAppear").is_some() {
        values.push(("UIClearsSelectionOnViewWillAppear", Value::Bool(false)));
    }
    // One with its own nib too is swapped in, as it is there.
    let controller = if vc.attribute("customClass").is_some() || root.loadable {
        values.push(("UIClassName", nib.string(&root.class_name)));
        values.push(("UIOriginalClassName", nib.string(root.original_class)));
        Object::new("UIClassSwapper", values)
    } else {
        Object::new(root.original_class, values)
    };
    Ok(Embedded {
        value: Value::New(controller.named(name)),
        templates,
        navigation_item: explicit_item.map(|_| item_name),
        bar_items,
        bar_actions,
        gestures,
        gesture_actions,
    })
}

/// A navigation controller's bar (`name`), at the height Xcode gives it
/// whatever the document says, holding `item`.
fn navigation_bar(nib: &mut Nib, nav: Node, name: &str, item: &str, delegate: &str) -> Result<Value, String> {
    let bar = children(nav, "navigationBar").next().ok_or("a navigation controller without a navigation bar")?;
    only_attributes(
        bar,
        &[
            ("key", Some("navigationBar")), ("contentMode", Some("scaleToFill")), ("insetsLayoutMarginsFromSafeArea", Some("NO")), ("largeTitles", Some("YES")),
            ("barStyle", Some("black")), ("translucent", Some("NO")), ("id", None),
        ],
    )?;
    only_children(bar, &["rect", "autoresizingMask", "color", "textAttributes"])?;
    if let Some(c) = children(bar, "color").find(|c| !matches!(c.attribute("key"), Some("backgroundColor" | "tintColor" | "barTintColor"))) {
        return fail(format!("a navigation bar's {} colour is not supported by this ibtool", c.attribute("key").unwrap_or("unkeyed")));
    }
    if let Some(t) = children(bar, "textAttributes").find(|t| !matches!(t.attribute("key"), Some("titleTextAttributes" | "largeTitleTextAttributes"))) {
        return fail(format!("a navigation bar's {} are not supported by this ibtool", t.attribute("key").unwrap_or("unkeyed text attributes")));
    }
    for m in children(bar, "autoresizingMask") {
        if autoresizing_mask(m)? != 0 || m.has_children() {
            return fail("navigation bars with an autoresizing mask are not supported by this ibtool");
        }
    }
    let f = frame(bar)?;
    if f.x != 0.0 {
        return fail("navigation bars not at x 0 are not supported by this ibtool");
    }
    let opaque = bar.attribute("translucent").is_some();
    let mut values = vec![("UIBounds", doubles(&[0.0, 0.0, 0.0, 54.0])), ("UICenter", doubles(&[f.x, f.y + 27.0]))];
    if let Some(c) = child(bar, "color", "backgroundColor") {
        values.push(("UIBackgroundColor", color(nib, c)?));
    }
    values.push(("UIDeepDrawRect", Value::Bool(true)));
    if opaque {
        values.push(("UIOpaque", Value::Bool(true)));
    }
    values.push(("UIAutoresizeSubviews", Value::Bool(true)));
    values.push(("UIAutoresizingMask", Value::Int(2)));
    let tint = match child(bar, "color", "tintColor") {
        Some(own) => Some(own),
        None => document_tint(bar)?,
    };
    if let Some(tint) = tint {
        values.push(("UITintColor", color(nib, tint)?));
    }
    let hugging = nib.string("{250, 250}");
    values.extend([
        ("UIViewContentHuggingPriority", hugging),
        ("UIViewLayoutMargins", doubles(&[0.0, 0.0, 0.0, 0.0])),
        ("UIViewLayoutMarginsAreDirectional", Value::Bool(true)),
    ]);
    if bar.attribute("insetsLayoutMarginsFromSafeArea").is_some() {
        values.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0)));
    }
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    if bar.attribute("barStyle").is_some() {
        values.push(("UIBarStyle", Value::Int(1)));
    }
    if bar.attribute("largeTitles").is_some() {
        values.push(("UIBarPrefersLargeTitles", Value::Bool(true)));
    }
    if let Some(t) = child(bar, "textAttributes", "largeTitleTextAttributes") {
        values.push(("UIBarLargeTitleTextAttributes", text_attributes(nib, t)?));
    }
    values.push(("UIIsLocked", Value::Bool(true)));
    if let Some(c) = child(bar, "color", "barTintColor") {
        values.push(("UIBarTintColor", color(nib, c)?));
    }
    let items = nib.array("NSMutableArray", vec![reference(item)]);
    values.push(("UIItems", items));
    values.push(("UIDelegate", reference(delegate)));
    if let Some(t) = child(bar, "textAttributes", "titleTextAttributes") {
        values.push(("UITitleTextAttributes", text_attributes(nib, t)?));
    }
    values.push(("UIBarPosition", Value::Int(2)));
    values.push(("UIBarTranslucence", Value::Int(if opaque { 2 } else { 1 })));
    Ok(Value::New(Object::new("UINavigationBar", values).named(name)))
}

/// A bar's `<textAttributes>`: a dictionary of its text colour (`NSColor`)
/// and font (`NSFont`), in that order.
fn text_attributes(nib: &mut Nib, t: Node) -> Result<Value, String> {
    only_attributes(t, &[("key", None)])?;
    only_children(t, &["color", "fontDescription"])?;
    let mut entries = Vec::new();
    if let Some(c) = children(t, "color").next() {
        if c.attribute("key") != Some("textColor") || children(t, "color").count() > 1 {
            return fail("text attributes other than one text colour and a font are not supported by this ibtool");
        }
        entries.extend([nib.string("NSColor"), color(nib, c)?]);
    }
    if let Some(f) = child(t, "fontDescription", "fontDescription") {
        entries.extend([nib.string("NSFont"), font(nib, Some(f), None)?]);
    }
    Ok(nib.array("NSDictionary", entries))
}

/// A button's plain configuration with a title: a `UIButtonConfiguration`
/// of UIKit's defaults (a clear background and stroke, corner radius 17,
/// title padding 1), its title an attributed string, localizable as
/// `<id>.configuration.title`, and an empty subtitle (four probes).
fn button_configuration(nib: &mut Nib, c: Node, id: &str) -> Result<Value, String> {
    only_attributes(c, &[("key", Some("configuration")), ("style", Some("plain")), ("title", None)])?;
    only_children(c, &[])?;
    let clear = nib.system_color("clearColor", &SystemColor::Gray([0.0, 0.0]));
    let stroke = nib.system_color("clearColor", &SystemColor::Gray([0.0, 0.0]));
    let insets = nib.string("{0, 0, 0, 0}");
    let background = Object::new("UIBackgroundConfiguration", vec![
        ("customView", Value::Nil),
        ("hasCustomized-customView", Value::Bool(false)),
        ("cornerRadius", Value::Double(17.0)),
        ("hasCustomized-cornerRadius", Value::Bool(false)),
        ("maskedCorners", Value::Int(15)),
        ("hasCustomized-maskedCorners", Value::Bool(false)),
        ("backgroundColor", clear),
        ("hasCustomized-backgroundColor", Value::Bool(false)),
        ("backgroundColorTransformerIdentifier", Value::Int(0)),
        ("hasCustomized-backgroundColorTransformerIdentifier", Value::Bool(false)),
        ("visualEffect", Value::Nil),
        ("hasCustomized-visualEffect", Value::Bool(false)),
        ("visualEffectGroupName", Value::Nil),
        ("hasCustomized-visualEffectGroupName", Value::Bool(false)),
        ("visualEffectblurClippingMode", Value::Int(0)),
        ("image", Value::Nil),
        ("hasCustomized-image", Value::Bool(false)),
        ("imageContentMode", Value::Int(0)),
        ("hasCustomized-imageContentMode", Value::Bool(false)),
        ("backgroundInsets", insets),
        ("hasCustomized-backgroundInsets", Value::Bool(false)),
        ("edgesAddingLayoutMarginsToBackgroundInsets", Value::Int(0)),
        ("hasCustomized-edgesAddingLayoutMarginsToBackgroundInsets", Value::Bool(false)),
        ("strokeColor", stroke),
        ("hasCustomized-strokeColor", Value::Bool(true)),
        ("strokeColorTransformerIdentifier", Value::Int(0)),
        ("hasCustomized-strokeColorTransformerIdentifier", Value::Bool(false)),
        ("strokeWidth", Value::Double(1.0)),
        ("hasCustomized-strokeWidth", Value::Bool(true)),
        ("strokeOutset", Value::Double(0.0)),
        ("hasCustomized-strokeOutset", Value::Bool(false)),
        ("strokeLocation", Value::Int(0)),
        ("defaultStyle", Value::Int(0)),
        ("defaultStyle_v2", Value::Int(0)),
    ]);
    let title = nib.text(id, "configuration.title", c.attribute("title").unwrap_or(""));
    let attributes = nib.shared("empty attributes".into(), || Object::new("NSDictionary", vec![("NSInlinedValue", Value::Bool(true))]));
    let subtitle = nib.string("");
    Ok(Value::New(Object::new("UIButtonConfiguration", vec![
        ("UIButtonConfigurationButtonStyle", Value::Int(0)),
        ("UIButtonConfigurationBackground", Value::New(background)),
        ("UIButtonConfigurationCornerStyle", Value::Int(0)),
        ("UIButtonConfigurationButtonSize", Value::Int(0)),
        ("UIButtonConfigurationMacIdiomStyle", Value::Int(0)),
        ("UIButtonConfigurationBaseBackgroundColor", Value::Nil),
        ("UIButtonConfigurationBaseForegroundColor", Value::Nil),
        ("UIButtonConfigurationImage", Value::Nil),
        ("UIButtonConfigurationPreferredSymbolConfigurationForImage", Value::Nil),
        ("UIButtonConfigurationSymbolContentTransition", Value::Nil),
        ("UIButtonConfigurationShowsActivityIndicator", Value::Bool(false)),
        ("UIButtonConfigurationTitle", Value::New(Object::new("NSAttributedString", vec![("NSString", title), ("NSAttributes", attributes)]))),
        ("UIButtonConfigurationSubtitle", Value::New(Object::new("NSAttributedString", vec![("NSString", subtitle)]))),
        ("UIButtonConfigurationImagePlacement", Value::Int(2)),
        ("UIButtonConfigurationImagePadding", Value::Float(0.0)),
        ("UIButtonConfigurationTitlePadding", Value::Float(1.0)),
        ("UIButtonConfigurationTitleAlignment", Value::Int(0)),
    ])))
}

/// A navigation controller's attributes, checked.
fn check_navigation_controller(nav: Node) -> Result<(), String> {
    only_attributes(
        nav,
        &[
            ("id", None),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
            ("storyboardIdentifier", None),
            ("useStoryboardIdentifierAsRestorationIdentifier", Some("YES")),
            ("title", None),
            ("automaticallyAdjustsScrollViewInsets", Some("NO")),
            ("sceneMemberID", Some("viewController")),
        ],
    )?;
    if let Some(identifier) = nav.attribute("storyboardIdentifier") {
        if identifier.is_empty() || identifier.contains('/') || !identifier.is_ascii() {
            return fail(format!("storyboard identifier {identifier:?} is not supported by this ibtool"));
        }
    } else if nav.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
        return fail("useStoryboardIdentifierAsRestorationIdentifier without a storyboard identifier is not supported by this ibtool");
    }
    Ok(())
}

/// A navigation controller's object (`name`), with its root (`root`) and
/// bar inside: in its own nib, or a tab bar controller's child there
/// (`tab`: its tab bar item and parent).
#[allow(clippy::too_many_arguments)]
fn navigation_controller(
    nib: &mut Nib,
    module: Option<&str>,
    nav: Node,
    name: &str,
    root: Embedded,
    root_name: &str,
    swapped: bool,
    title: Option<Value>,
    held: Option<(Option<Value>, &str)>,
) -> Result<Value, String> {
    let bar_name = format!("{name} bar");
    let item = root.navigation_item.clone().unwrap_or_else(|| format!("{root_name} implied navigation item"));
    let mut values = Vec::new();
    if let Some(title) = title {
        values.push(("UITitle", title));
    }
    if let Some((item, parent)) = held {
        if let Some(item) = item {
            values.push(("UITabBarItem", item));
        }
        values.push(("UIParentViewController", reference(parent)));
    } else if let Some(identifier) = nav.attribute("storyboardIdentifier") {
        values.push(("UIStoryboardIdentifier", nib.string(identifier)));
        if nav.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
            values.push(("UIRestorationIdentifier", nib.string(identifier)));
        }
    }
    values.push(("UIChildViewControllers", nib.array("NSMutableArray", vec![root.value])));
    if nav.attribute("automaticallyAdjustsScrollViewInsets").is_some() {
        values.push(("UIKeyAutomaticallyAdjustsScrollViewInsets", Value::Int(0)));
    }
    values.push(("UIViewControllers", nib.array("NSMutableArray", vec![reference(root_name)])));
    values.push(("UINavigationBar", navigation_bar(nib, nav, &bar_name, &item, name)?));
    // Swapped in for its class only when initial, identified or of a
    // custom class.
    let controller = if swapped || nav.attribute("customClass").is_some() {
        let class = runtime_class_name(nav, module, "UINavigationController")?;
        values.push(("UIClassName", nib.string(&class)));
        values.push(("UIOriginalClassName", nib.string("UINavigationController")));
        Object::new("UIClassSwapper", values)
    } else {
        Object::new("UINavigationController", values)
    };
    Ok(Value::New(controller.named(name)))
}

/// A navigation controller's title, made before what it holds: a string
/// equal to its root's title is archived with its own.
fn navigation_title(nib: &mut Nib, nav: Node) -> Result<Option<Value>, String> {
    Ok(match nav.attribute("title") {
        Some(title) => Some(nib.text(attr(nav, "id")?, "title", title)),
        None => None,
    })
}

/// A navigation controller's nib: the scenes' placeholders, the controller
/// swapped in for UINavigationController, its root view controller (as in
/// its own nib, with its navigation item, real or implied) and its bar.
fn navigation_controller_nib(
    platform: Platform,
    color_matched: bool,
    localizable: bool,
    module: Option<&str>,
    nav: Node,
    root: &RootController,
    swapped: bool,
    nib_names: &HashMap<String, String>,
) -> Result<(String, Vec<u8>), String> {
    check_navigation_controller(nav)?;
    if child(nav, "tabBarItem", "tabBarItem").is_some() {
        return fail("a navigation controller's tab bar item in its own nib is not supported by this ibtool");
    }
    let nav_id = attr(nav, "id")?;
    let name = nav.attribute("storyboardIdentifier").map_or_else(|| format!("UINavigationController-{nav_id}"), str::to_string);
    let mut nib = Nib::new(platform, color_matched, localizable);
    // The root's scene's storyboard placeholder comes first.
    let root_storyboard = proxy(&mut nib, "UIStoryboardPlaceholder", "root storyboard");
    let owner = proxy(&mut nib, "IBFilesOwner", "owner");
    let responder = proxy(&mut nib, "IBFirstResponder", "responder");
    let storyboard = proxy(&mut nib, "UIStoryboardPlaceholder", "storyboard");
    let title = navigation_title(&mut nib, nav)?;
    let mut embedded = embedded_controller(&mut nib, root, "root", "controller", Some("controller bar"), None, nib_names)?;
    let explicit_item = embedded.navigation_item.clone();
    let templates_objects = std::mem::take(&mut embedded.templates.objects);
    let templates_outlets = std::mem::take(&mut embedded.templates.outlets);
    let bar_items = std::mem::take(&mut embedded.bar_items);
    let bar_actions = std::mem::take(&mut embedded.bar_actions);
    let templates_performs = std::mem::take(&mut embedded.templates.performs);
    let gestures = std::mem::take(&mut embedded.gestures);
    let gesture_actions = std::mem::take(&mut embedded.gesture_actions);
    let controller = navigation_controller(&mut nib, module, nav, "controller", embedded, "root", swapped, title, None)?;
    // The root's gesture recognizers follow the controller (two probes).
    let mut objects = vec![reference("root storyboard"), reference("owner"), reference("responder"), reference("storyboard"), reference("controller")];
    objects.extend(gestures.iter().map(|g| reference(g.clone())));
    objects.extend([reference("controller bar"), reference("root")]);
    if let Some(item) = explicit_item {
        objects.push(reference(item));
    }
    objects.extend(templates_objects);
    objects.extend(bar_items.iter().map(|b| reference(b.clone())));
    let mut connections = vec![
        outlet(&mut nib, "storyboard", "controller", "storyboard"),
        outlet(&mut nib, "storyboard", "root", "root storyboard"),
        outlet(&mut nib, "sceneViewController", "owner", "controller"),
    ];
    for t in &templates_outlets {
        connections.push(outlet(&mut nib, "viewController", t, "root"));
    }
    connections.extend(bar_events(&mut nib, &bar_items, &bar_actions, &templates_performs, "root"));
    for (selector, gesture) in &gesture_actions {
        connections.push(event_connection(&mut nib, selector, gesture, "root"));
    }
    let mut top = vec![root_storyboard, owner, responder, storyboard, controller];
    top.extend(gestures.iter().map(|g| reference(g.clone())));
    Ok((name, nib::archive(root_object(&mut nib, top, objects, connections))))
}

pub fn compile(xml: &str, minimum_deployment_target: Option<&str>, module: Option<&str>, localizable: bool) -> Result<Compiled, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    let platform = match root.attribute("type") {
        Some("com.apple.InterfaceBuilder3.CocoaTouch.Storyboard.XIB") => Platform::Ios,
        Some("com.apple.InterfaceBuilder.AppleTV.Storyboard") => Platform::Tvos,
        _ => return fail("not an iOS or tvOS storyboard"),
    };
    // Without a <device>, Interface Builder lays an iOS scene out on its
    // generic canvas below iOS 17, and at the view's frame from iOS 17 on;
    // Xcode's ibtool fails on a tvOS one.
    let device = children(root, "device").next();
    let has_device = device.is_some();
    let device_size = match device {
        Some(d) => device_size(platform, attr(d, "id")?, attr(d, "orientation")?)?,
        None => None,
    };
    let generic_canvas = if has_device {
        false
    } else if platform == Platform::Tvos {
        return fail("a tvOS storyboard without a device is not compiled by Xcode's ibtool");
    } else {
        let target = minimum_deployment_target.ok_or("a storyboard without a device needs --minimum-deployment-target for this ibtool")?;
        let major: u32 = target.split('.').next().and_then(|m| m.parse().ok()).ok_or_else(|| format!("not a deployment target: {target}"))?;
        major < 17
    };
    let color_matched = root.attribute("colorMatched") == Some("YES");
    let initial = root.attribute("initialViewController");
    let scenes = children(root, "scenes").next().ok_or("storyboard has no scenes")?;
    let scene_nodes: Vec<Node> = children(scenes, "scene").collect();
    // Navigation controllers, with their root view controllers' ids. One
    // that can be loaded (initial or identified) embeds its root
    // controller, whose view nib is then compiled even if nothing else
    // loads it.
    // Every controller's own nib's name, and the controllers only segues
    // make loadable.
    let controllers: Vec<Node> = scene_nodes.iter().filter_map(|s| segue::scene_controller(*s)).collect();
    let mut nib_names: HashMap<String, String> = HashMap::new();
    for c in &controllers {
        nib_names.insert(attr(*c, "id")?.to_string(), segue::nib_name(*c)?);
    }
    let reached = segue::segue_destinations(&controllers, initial)?;
    // An embed segue's destination is laid out at its container's size, and
    // so is every controller a segue leads to from it, unless one has a
    // freeform size of its own.
    let mut embedded: HashMap<&str, (f64, f64)> = HashMap::new();
    for container in root.descendants().filter(|d| d.has_tag_name("containerView")) {
        for sg in container.descendants().filter(|d| d.has_tag_name("segue")) {
            let destination = attr(sg, "destination")?;
            let kinds = ["viewController", "tableViewController", "collectionViewController", "viewControllerPlaceholder"];
            if !controllers.iter().any(|c| c.attribute("id") == Some(destination) && kinds.contains(&c.tag_name().name())) {
                return fail("an embed segue to anything but a view or table view controller is not supported by this ibtool");
            }
            let f = frame(container)?;
            if embedded.insert(destination, (f.w, f.h)).is_some() {
                return fail("a controller embedded more than once is not supported by this ibtool");
            }
        }
    }
    let mut queue: Vec<&str> = embedded.keys().copied().collect();
    while let Some(id) = queue.pop() {
        let size = embedded[id];
        let Some(c) = controllers.iter().find(|c| c.attribute("id") == Some(id)) else { continue };
        let segues = c.descendants().filter(|d| d.has_tag_name("segue") && !matches!(d.attribute("kind"), Some("relationship" | "unwind" | "embed")));
        for sg in segues {
            let destination = attr(sg, "destination")?;
            if !controllers.iter().any(|c| c.attribute("id") == Some(destination) && matches!(c.tag_name().name(), "viewController" | "tableViewController" | "collectionViewController")) {
                return fail("a segue from an embedded controller to anything but a view or table view controller is not supported by this ibtool");
            }
            match embedded.get(destination) {
                Some(other) if *other != size => return fail("a controller laid out at two containers' sizes is not supported by this ibtool"),
                Some(_) => {}
                None => {
                    embedded.insert(destination, size);
                    queue.push(destination);
                }
            }
        }
    }
    let mut navs = Vec::new();
    let mut parents = Vec::new();
    for (index, scene_node) in scene_nodes.iter().enumerate() {
        if let Some(nav) = scene_node.descendants().find(|d| d.has_tag_name("navigationController")) {
            let root_id = navigation_root(nav)?;
            let nav_id = attr(nav, "id")?;
            let loadable = initial == Some(nav_id) || nav.attribute("storyboardIdentifier").is_some() || reached.contains(nav_id);
            navs.push((index, nav, root_id, loadable));
        }
        let parent = scene_node.descendants().find_map(|d| match d.tag_name().name() {
            "tabBarController" => Some((parent::Kind::Tab, d)),
            "splitViewController" => Some((parent::Kind::Split, d)),
            _ => None,
        });
        if let Some((kind, node)) = parent {
            let kids = parent::children_of(kind, node)?;
            let id = attr(node, "id")?;
            let loadable = initial == Some(id) || node.attribute("storyboardIdentifier").is_some() || reached.contains(id);
            parents.push((index, kind, node, kids, loadable));
        }
    }
    // A loadable tab bar or split view controller holds its children, and
    // the roots of those that are navigation controllers.
    let held_children: HashSet<&str> = parents.iter().filter(|p| p.4).flat_map(|p| p.3.iter().map(|k| k.1)).collect();
    let in_held_nav = |id: &str| navs.iter().any(|n| n.2 == id && held_children.contains(attr(n.1, "id").unwrap_or("")));
    // Each scene's files and Info.plist entry, by scene.
    let mut outputs: Vec<(usize, Vec<(String, Vec<u8>)>, Option<(String, bool, bool)>)> = Vec::new();
    let mut roots: HashMap<&str, RootController> = HashMap::new();
    // Storyboard references by nib name, with what they refer to.
    let mut references: Vec<(String, cf::Value)> = Vec::new();
    for (index, scene_node) in scene_nodes.iter().enumerate() {
        let scene_node = *scene_node;
        only_children(scene_node, &["objects", "point"])?;
        let objects_node = children(scene_node, "objects").next().ok_or("scene without objects")?;
        only_children(objects_node, &[
            "viewController", "tableViewController", "collectionViewController", "navigationController", "tabBarController", "splitViewController", "placeholder",
            "exit", "viewControllerPlaceholder", "tapGestureRecognizer", "panGestureRecognizer", "swipeGestureRecognizer",
        ])?;
        // An exit is Interface Builder's alone; the unwind segues to it
        // compile to templates of their own.
        for exit in children(objects_node, "exit") {
            only_attributes(exit, &[("id", None), ("userLabel", Some("Exit")), ("sceneMemberID", Some("exit"))])?;
            let id = attr(exit, "id")?;
            let unwinds = |d: Node| d.has_tag_name("segue") && d.attribute("kind") == Some("unwind") && d.attribute("destination") == Some(id);
            if scene_node.document().descendants().any(|d| d != exit && !unwinds(d) && d.attributes().any(|a| a.value() == id)) {
                return fail("an exit referred to other than by an unwind segue is not supported by this ibtool");
            }
        }
        // A storyboard reference a segue leads to is a placeholder in a nib
        // of its own, which the Info.plist resolves; one nothing leads to
        // compiles to nothing.
        if let Some(placeholder) = children(objects_node, "viewControllerPlaceholder").next() {
            only_attributes(placeholder, &[
                ("storyboardName", None), ("referencedIdentifier", None), ("bundleIdentifier", None), ("id", None), ("sceneMemberID", Some("viewController")),
            ])?;
            let id = attr(placeholder, "id")?;
            if initial == Some(id) {
                return fail("an initial storyboard reference is not supported by this ibtool");
            }
            if !reached.contains(id) {
                continue;
            }
            let name = segue::nib_name(placeholder)?;
            let mut nib = Nib::new(platform, color_matched, localizable);
            let owner = proxy(&mut nib, "IBFilesOwner", "owner");
            let responder = proxy(&mut nib, "IBFirstResponder", "responder");
            let storyboard = proxy(&mut nib, "UIStoryboardPlaceholder", "storyboard");
            let identifier = nib.string(&name);
            let controller = Value::New(Object::new("UIStoryboardViewControllerPlaceholder", vec![("UIStoryboardIdentifier", identifier)]).named("controller"));
            let objects = vec![reference("owner"), reference("responder"), reference("storyboard"), reference("controller")];
            let connections = vec![outlet(&mut nib, "sceneViewController", "owner", "controller")];
            let root = root_object(&mut nib, vec![owner, responder, storyboard, controller], objects, connections);
            outputs.push((index, vec![(format!("{name}.nib"), nib::archive(root))], Some((name.clone(), false, false))));
            let mut target = Vec::new();
            if let Some(bundle) = placeholder.attribute("bundleIdentifier") {
                target.push(("UIReferencedBundleIdentifier", cf::Value::string(bundle)));
            }
            if let Some(controller) = placeholder.attribute("referencedIdentifier") {
                target.push(("UIReferencedControllerIdentifier", cf::Value::string(controller)));
            }
            target.push(("UIReferencedStoryboardName", cf::Value::string(attr(placeholder, "storyboardName")?)));
            references.push((name, cf::Value::dict(target)));
            continue;
        }
        if ["navigationController", "tabBarController", "splitViewController"].iter().any(|t| children(objects_node, t).next().is_some()) {
            continue;
        }
        let vc = objects_node
            .children()
            .find(|c| matches!(c.tag_name().name(), "viewController" | "tableViewController" | "collectionViewController"))
            .ok_or("scene without a view controller")?;
        // A table view controller's view is its table.
        let is_table = vc.has_tag_name("tableViewController");
        let is_collection = vc.has_tag_name("collectionViewController");
        // A table or collection view is the controller's root view.
        let is_list = is_table || is_collection;
        let view_key = match vc.tag_name().name() {
            "tableViewController" => "tableView",
            "collectionViewController" => "collectionView",
            _ => "view",
        };
        let original_class = segue::original_class(vc);
        only_attributes(vc, &[
            ("id", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None),
            ("storyboardIdentifier", None), ("useStoryboardIdentifierAsRestorationIdentifier", Some("YES")), ("title", None),
            ("automaticallyAdjustsScrollViewInsets", Some("NO")), ("clearsSelectionOnViewWillAppear", Some("NO")), ("userLabel", None),
            ("restorationIdentifier", None),
            ("sceneMemberID", Some("viewController")),
        ])?;
        if !is_table && vc.attribute("clearsSelectionOnViewWillAppear").is_some() {
            if is_collection {
                return fail("the clearsSelectionOnViewWillAppear attribute of <collectionViewController> is not supported by this ibtool");
            }
            return fail("the clearsSelectionOnViewWillAppear attribute of <viewController> is not supported by this ibtool");
        }
        let class_name = runtime_class_name(vc, module, original_class)?;
        only_children(vc, &[view_key, "layoutGuides", "connections", "navigationItem", "tabBarItem", "size", "toolbarItems", "extendedEdge", "userDefinedRuntimeAttributes", "refreshControl"])?;
        if !is_table && children(vc, "refreshControl").next().is_some() {
            return fail("a refresh control outside a table view controller is not supported by this ibtool");
        }
        // Empty toolbar items change nothing compiled.
        if children(vc, "toolbarItems").any(|t| t.has_children()) {
            return fail("view controllers with toolbar items are not supported by this ibtool");
        }
        if children(vc, "size").any(|s| s.attribute("key") != Some("freeformSize")) {
            return fail("sizes other than a freeform size inside <viewController> are not supported by this ibtool");
        }
        let vc_id = attr(vc, "id")?;
        let view = children(vc, view_key).next().ok_or("view controller without a view")?;
        if is_list && children(vc, "layoutGuides").next().is_some() {
            return fail("a table or collection view controller with layout guides is not supported by this ibtool");
        }
        if !is_list {
            only_children(view, &[
                "rect", "autoresizingMask", "viewLayoutGuide", "color", "subviews", "constraints", "connections", "userDefinedRuntimeAttributes", "edgeInsets",
                "directionalEdgeInsets", "accessibility",
            ])?;
            only_attributes(view, &[
                ("key", Some("view")), ("contentMode", Some("scaleToFill")), ("insetsLayoutMarginsFromSafeArea", None),
                ("layoutMarginsFollowReadableWidth", Some("YES")), ("preservesSuperviewLayoutMargins", Some("YES")), ("opaque", Some("NO")),
                ("clearsContextBeforeDrawing", Some("NO")), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("id", None),
            ])?;
        }
        let view_id = attr(view, "id")?;
        let root_name = view_name(view_id);

        let mut scene = Scene::new(vc_id, module, view, None);
        // The gesture recognizers, which the view nib reaches through the
        // first placeholders.
        let recognizers: Vec<Node> = objects_node.children().filter(|c| c.tag_name().name().ends_with("GestureRecognizer")).collect();
        if !recognizers.is_empty() && is_list {
            return fail("a gesture recognizer in a table or collection view controller's scene is not supported by this ibtool");
        }
        scene.gestures = recognizers.iter().map(|r| attr(*r, "id").map(str::to_string)).collect::<Result<_, _>>()?;
        let gesture = recognizers.len();
        for g in children(vc, "layoutGuides") {
            only_children(g, &["viewControllerLayoutGuide"])?;
            for guide in children(g, "viewControllerLayoutGuide") {
                let kind = match attr(guide, "type")? {
                    "top" => "top",
                    "bottom" => "bottom",
                    other => return fail(format!("layout guide type {other} is not supported by this ibtool")),
                };
                scene.guides.push((attr(guide, "id")?, kind));
            }
        }
        if scene.guides.iter().map(|g| g.1).collect::<Vec<_>>() != ["top", "bottom"] && !scene.guides.is_empty() {
            return fail("layout guides other than a top and a bottom one are not supported by this ibtool");
        }
        let safe_area = children(view, "viewLayoutGuide").find(|g| g.attribute("key") == Some("safeArea"));
        // A document using safe areas gives every root view a safe-area
        // guide, the same whether or not its view lists one (three probes).
        let uses_safe_areas = view.document().root_element().attribute("useSafeAreas") == Some("YES");
        // So does one with legacy guides, beside them, listed or not (three
        // probes: a launch screen with both, and with each alone).
        let has_safe_area = safe_area.is_some() || uses_safe_areas;
        if !is_list && !has_safe_area && scene.guides.is_empty() {
            return fail("a view needs either a safe-area layout guide or legacy top and bottom layout guides for this ibtool");
        }
        let safe_area_listed_id = safe_area.and_then(|g| g.attribute("id"));
        if !scene.guides.is_empty()
            && safe_area_listed_id.is_some_and(|id| view.descendants().any(|c| c.has_tag_name("constraint") && (c.attribute("firstItem") == Some(id) || c.attribute("secondItem") == Some(id))))
        {
            return fail("constraints on a safe-area guide beside legacy layout guides are not supported by this ibtool");
        }

        // Outlets from the view controller into its view, and its own
        // segues.
        let mut vc_outlets = Vec::new();
        // Outlet collections: (property, collection class, views), by first
        // appearance, the views in document order.
        let mut vc_collections: Vec<(String, String, Vec<String>)> = Vec::new();
        let own_segues: Vec<Node> = children(vc, "connections").flat_map(|c| children(c, "segue")).collect();
        for c in children(vc, "connections") {
            only_children(c, &["outlet", "outletCollection", "segue"])?;
            for o in children(c, "outlet") {
                only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
                vc_outlets.push((attr(o, "property")?.to_string(), view_name(attr(o, "destination")?)));
            }
            for o in children(c, "outletCollection") {
                only_attributes(o, &[("property", None), ("destination", None), ("collectionClass", None), ("id", None)])?;
                let (property, class) = (attr(o, "property")?, o.attribute("collectionClass").unwrap_or("NSArray"));
                if !matches!(class, "NSArray" | "NSMutableArray") {
                    return fail(format!("an outlet collection of class {class} is not supported by this ibtool"));
                }
                let destination = view_name(attr(o, "destination")?);
                match vc_collections.iter_mut().find(|c| c.0 == property) {
                    Some(c) if c.1 == class => c.2.push(destination),
                    Some(_) => return fail("an outlet collection of two classes is not supported by this ibtool"),
                    None => vc_collections.push((property.to_string(), class.to_string(), vec![destination])),
                }
            }
        }

        let mut nib = Nib::new(platform, color_matched, localizable);
        // A controller's freeform size (`<size key="freeformSize">`, the
        // simulated metrics' own size) replaces the device's or the generic
        // canvas's, but not the frame a device-less view from iOS 17 keeps.
        let freeform = match child(vc, "size", "freeformSize") {
            Some(s) => Some((number(attr(s, "width")?)?, number(attr(s, "height")?)?)),
            None => None,
        };
        // The generic canvas is size classes' 600x600; without them, the
        // default device's 393x852.
        let traits = root.attribute("useTraitCollections") == Some("YES");
        let f = match (generic_canvas, has_device, device_size, freeform) {
            (true, _, _, None) | (false, true, Some(_), None) if embedded.contains_key(vc_id) => {
                let (w, h) = embedded[vc_id];
                Frame { x: 0.0, y: 0.0, w, h }
            }
            (false, false, _, _) | (false, true, None, None) if embedded.contains_key(vc_id) => {
                return fail("an embedded controller without a device or the generic canvas is not supported by this ibtool")
            }
            (true, _, _, None) if traits => Frame { x: 0.0, y: 0.0, w: 600.0, h: 600.0 },
            (true, _, _, None) => Frame { x: 0.0, y: 0.0, w: 393.0, h: 852.0 },
            (true, _, _, Some((w, h))) if traits => Frame { x: 0.0, y: 0.0, w, h },
            (true, _, _, Some(_)) => return fail("a freeform size without trait collections is not supported by this ibtool"),
            // On a device, the view keeps its frame's origin (three probes).
            (false, true, Some(_), Some((w, h))) | (false, true, Some((w, h)), None) => {
                let origin = child(view, "rect", "frame").map(|_| frame(view)).transpose()?;
                Frame { x: origin.as_ref().map_or(0.0, |o| o.x), y: origin.as_ref().map_or(0.0, |o| o.y), w, h }
            }
            (false, true, None, Some(_)) => return fail("a freeform size with this device is not supported by this ibtool"),
            (false, _, _, _) => frame(view)?,
        };
        if is_list && !vc_collections.is_empty() {
            return fail("a table or collection view controller's outlet collections are not supported by this ibtool");
        }
        let (view_root, upstream_used) = if is_table {
            let (root, upstream_used, segues, listing) = table::table_view_nib(platform, color_matched, localizable, module, vc_id, view, &f, &vc_outlets, child(vc, "refreshControl", "refreshControl"))?;
            scene.segues = segues;
            scene.segue_listing = listing;
            (root, upstream_used)
        } else if is_collection {
            let (root, upstream_used, segues) = collection::collection_view_nib(platform, color_matched, localizable, module, vc_id, view, &f, &vc_outlets)?;
            scene.segues = segues;
            (root, upstream_used)
        } else {
            // Laid out at another size than its frame's, the root view resizes
            // its autoresizing subviews: only those whose masks keep their
            // frames (fixed left and top margins, size) are established.
            let document = frame(view)?;
            if (f.w, f.h) != (document.w, document.h) {
                for s in children(view, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())) {
                    if s.attribute("translatesAutoresizingMaskIntoConstraints").is_some() {
                        continue;
                    }
                    let mask = match children(s, "autoresizingMask").next() {
                        Some(m) => autoresizing_mask(m)?,
                        None => 36,
                    };
                    if mask & (1 | 2 | 8 | 16) != 0 {
                        return fail("an autoresizing subview of a view laid out at another size than its frame is not supported by this ibtool");
                    }
                }
            }
            let mask = match children(view, "autoresizingMask").next() {
                Some(m) => autoresizing_mask(m)?,
                None => 0,
            };
            let background = child(view, "color", "backgroundColor");
            if let Some(c) = children(view, "color").find(|c| !matches!(c.attribute("key"), Some("backgroundColor" | "tintColor"))) {
                return fail(format!("a root view's {} colour is not supported by this ibtool", c.attribute("key").unwrap_or("unkeyed")));
            }
            let tint = match child(view, "color", "tintColor") {
                Some(own) => Some(own),
                None => document_tint(view)?,
            };

            // The view's nib, in encoding order.
            let owner = proxy(&mut nib, "IBFilesOwner", "owner");
            let responder = proxy(&mut nib, "IBFirstResponder", "responder");
            if !vc_collections.is_empty() && !scene.gesture_views.is_empty() {
                return fail("outlet collections beside a gesture recognizer are not supported by this ibtool");
            }
            let upstream_used = !vc_outlets.is_empty()
                || !vc_collections.is_empty()
                || view.descendants().any(|d| (d.has_tag_name("action") || d.has_tag_name("outlet")) && d.attribute("destination") == Some(vc_id));
            let mut top = vec![owner, responder];
            let mut objects = vec![reference("owner"), reference("responder")];
            if upstream_used {
                top.push(proxy(&mut nib, &format!("UpstreamPlaceholder-{}", 1 + gesture), "upstream"));
                objects.push(reference("upstream"));
            }

            let mut values = Vec::new();
            geometry(&mut values, &f);
            let pairs = runtime_attributes(&mut nib, view, &root_name)?;
            scene.pairs.extend(pairs.into_iter().map(|p| (root_name.clone(), p)));
            // Its accessibility configuration, first in the objects' order
            // (one probe).
            accessibility(&mut nib, &mut scene, view, &root_name)?;
            let mut subview_values = Vec::new();
            for s in children(view, "subviews") {
                for c in s.children().filter(|c| c.is_element()) {
                    subview_values.push(subview(&mut nib, &mut scene, c)?);
                }
            }
            // Each control's segue has a placeholder of its own, after the
            // controller's.
            view_connections(&mut scene, view, &root_name)?;
            let numbered = gesture_order(&scene.gesture_views);
            if numbered.len() != gesture {
                return fail("a gesture recognizer no view has is not supported by this ibtool");
            }
            let gesture_proxies: Vec<Value> =
                numbered.iter().enumerate().map(|(i, g)| proxy(&mut nib, &format!("UpstreamPlaceholder-{}", i + 1), &format!("gesture {g}"))).collect();
            top.splice(2..2, gesture_proxies);
            objects.splice(2..2, numbered.iter().map(|g| reference(format!("gesture {g}"))));
            for i in 0..scene.segues.len() {
                let n = i + 1 + gesture + usize::from(upstream_used);
                let name = format!("upstream segue {i}");
                top.push(proxy(&mut nib, &format!("UpstreamPlaceholder-{n}"), &name));
                objects.push(reference(name));
            }
            let mut guide_constraints = Vec::new();
            let guide_kinds: Vec<&'static str> = scene.guides.iter().map(|g| g.1).collect();
            for kind in guide_kinds {
                let (guide, names) = legacy_guide(&mut nib, &root_name, kind);
                subview_values.push(guide);
                guide_constraints.extend(names);
            }
            if !subview_values.is_empty() {
                values.push(("UISubviews", nib.array("NSMutableArray", subview_values)));
            }
            if let Some(background) = background {
                values.push(("UIBackgroundColor", color(&mut nib, background)?));
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            if view.attribute("opaque").is_none() {
                values.push(("UIOpaque", Value::Bool(true)));
            }
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(mask)));
            if view.attribute("clearsContextBeforeDrawing").is_some() {
                if tint.is_some() {
                    return fail("a root view's tint colour beside clearsContextBeforeDrawing is not supported by this ibtool");
                }
                values.push(("UIClearsContextBeforeDrawing", Value::Bool(false)));
            }
            if let Some(tint) = tint {
                values.push(("UITintColor", color(&mut nib, tint)?));
            }

            // The view's constraints: the guides' first, then the document's in
            // the order Xcode gives them, by first attribute.
            let safe_area_id = safe_area.map(|g| attr(g, "id")).transpose()?;
            let mut known: Vec<&str> = vec![view_id];
            known.extend(view.descendants().filter(|d| d.is_element() && d.parent().is_some_and(|p| p.has_tag_name("subviews"))).filter_map(|d| d.attribute("id")));
            let item = |id: &str| -> Result<Value, String> {
                if let Some((_, kind)) = scene.guides.iter().find(|g| g.0 == id) {
                    return Ok(reference(format!("guide {kind}")));
                }
                if !known.contains(&id) {
                    return fail(format!("constraint item {id} is not supported by this ibtool"));
                }
                Ok(reference(view_name(id)))
            };
            // A group's place: its item's frame origin by distance from the
            // root view's, then x, then id; the safe-area guide's is the origin.
            let direct: Vec<Node> = children(view, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())).collect();
            // The bottom legacy guide's is the root view's foot (three
            // probes).
            let bottom_guide = scene.guides.iter().find(|g| g.1 == "bottom").map(|g| g.0);
            let place = |id: &str| -> Result<(f64, f64), String> {
                if Some(id) == safe_area_id {
                    return Ok((0.0, 0.0));
                }
                if Some(id) == bottom_guide {
                    return Ok((f.h * f.h, 0.0));
                }
                if let Some(v) = direct.iter().find(|v| v.attribute("id") == Some(id)) {
                    let f = subview_frame(*v)?;
                    return Ok((f.x * f.x + f.y * f.y, f.x));
                }
                // A view nested in plain views: its origin in the root view's
                // coordinates (four probes).
                let nested = || format!("root view constraints on a view nested in other than plain views ({id}) are not supported by this ibtool");
                let mut v = view.descendants().find(|d| d.is_element() && d.attribute("id") == Some(id)).ok_or_else(nested)?;
                let (mut x, mut y) = (0.0, 0.0);
                while v != view {
                    if !v.has_tag_name("view") || v.parent().is_none_or(|p| !p.has_tag_name("subviews")) {
                        return fail(nested());
                    }
                    let parent = v.parent().and_then(|p| p.parent()).ok_or_else(nested)?;
                    // The root's subview where it is laid out, if that
                    // doesn't resize it and so move what it holds.
                    let f = if parent == view {
                        let (laid, document) = (subview_frame(v)?, frame(v)?);
                        if (laid.w, laid.h) != (document.w, document.h) {
                            return fail(nested());
                        }
                        laid
                    } else {
                        frame(v)?
                    };
                    (x, y) = (x + f.x, y + f.y);
                    v = parent;
                }
                Ok((x * x + y * y, x))
            };
            let mut xml_constraints: Vec<((f64, f64), (&str, u8, [usize; 4]), Node, &str, Option<&str>)> = Vec::new();
            for cs in children(view, "constraints") {
                for c in children(cs, "constraint") {
                    only_attributes(c, &[
            ("firstItem", None), ("firstAttribute", None), ("relation", None), ("secondItem", None), ("secondAttribute", None), ("multiplier", None),
            ("constant", None), ("symbolic", Some("YES")), ("priority", None), ("identifier", None), ("placeholder", Some("YES")), ("id", None),
        ])?;
                    if c.attribute("placeholder").is_some() {
                        continue;
                    }
                    let first = c.attribute("firstItem").unwrap_or(view_id);
                    let second = c.attribute("secondItem").unwrap_or(view_id);
                    if c.attribute("secondItem").is_none() && c.attribute("secondAttribute").is_none() {
                        return fail("a root view constraint without a second item is not supported by this ibtool");
                    }
                    if first == second {
                        return fail("a constraint between an item and itself is not supported by this ibtool");
                    }
                    // Where a legacy guide's own constraints go isn't established;
                    // one as the second item of a sibling constraint is.
                    let legacy = |id: &str| scene.guides.iter().any(|g| g.0 == id);
                    let from_bottom_guide = Some(first) == bottom_guide && second != view_id && !legacy(second);
                    if (legacy(first) && !from_bottom_guide) || (legacy(second) && first == view_id) {
                        return fail("constraints from a legacy layout guide, or between one and the root view, are not supported by this ibtool");
                    }
                    for id in [first, second] {
                        if id != view_id && Some(id) != safe_area_id && !known.contains(&id) && !legacy(id) {
                            return fail(format!("constraints on {id} are not supported by this ibtool"));
                        }
                    }
                    let nested = second != view_id && Some(second) != safe_area_id && !legacy(second) && !direct.iter().any(|v| v.attribute("id") == Some(second));
                    let key = constraint_key(view_id, first, attr(c, "firstAttribute")?, second, attr(c, "secondAttribute")?, nested)?;
                    xml_constraints.push((place(key.0)?, key, c, first, Some(second)));
                }
            }
            xml_constraints.sort_by(|a, b| a.0 .0.total_cmp(&b.0 .0).then(a.0 .1.total_cmp(&b.0 .1)).then(a.1.cmp(&b.1)));
            order_ties(&mut xml_constraints, |o| (o.1, o.2), &direct)?;
            // Views deeper than the root's subviews that its constraints name,
            // in their order: the objects list them after the root's
            // subviews, not under their own superviews (six probes).
            let mut pulled: Vec<String> = Vec::new();
            for (_, _, _, first, second) in &xml_constraints {
                for id in std::iter::once(*first).chain(*second) {
                    let name = view_name(id);
                    let nested = scene.subviews.iter().any(|s| s.1 == name && s.0.is_some());
                    if nested && !pulled.contains(&name) {
                        pulled.push(name);
                    }
                }
            }
            let mut constraint_values: Vec<Value> = guide_constraints.iter().map(|n| reference(n.clone())).collect();
            let mut constraint_names = Vec::new();
            // The safe-area guide is archived where a constraint first uses it.
            let mut safe_area_used = false;
            let mut resolve = |nib: &mut Nib, id: &str| -> Result<Value, String> {
                if Some(id) != safe_area_id {
                    return item(id);
                }
                if safe_area_used {
                    return Ok(reference("safe area guide"));
                }
                safe_area_used = true;
                Ok(safe_area_guide(nib, &root_name))
            };
            for (_, _, c, first, second) in &xml_constraints {
                let first = resolve(&mut nib, first)?;
                let second = match second {
                    Some(s) => Some(resolve(&mut nib, s)?),
                    None => None,
                };
                let name = nib.fresh();
                constraint_names.push(name.clone());
                match document_constraint(&mut nib, *c, first, second)? {
                    Value::New(o) => constraint_values.push(Value::New(o.named(name))),
                    _ => unreachable!(),
                }
            }
            if !constraint_values.is_empty() {
                values.push(("UIViewAutolayoutConstraints", nib.array("NSMutableArray", constraint_values)));
            }
            if safe_area_used {
                values.push(("UIViewLayoutGuides", nib.array("NSMutableArray", vec![reference("safe area guide")])));
            }
            margins(&mut values, view)?;
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            // A custom class is swapped in for the root view when it loads.
            let root_object = if view.attribute("customClass").is_some() {
                let class = runtime_class_name(view, module, "UIView")?;
                values.push(("UIClassName", nib.string(&class)));
                values.push(("UIOriginalClassName", nib.string("UIView")));
                Object::new("UIClassSwapper", values)
            } else {
                Object::new("UIView", values)
            };
            top.push(Value::New(root_object.named(root_name.clone())));

            objects.push(reference(root_name.clone()));
            // The safe-area guide follows the root view.
            if has_safe_area {
                let identifier = nib.string("");
                objects.push(Value::New(Object::new("UILayoutGuide", vec![
                    ("UILayoutGuideOwningView", Value::Nil),
                    ("UILayoutGuideIdentifier", identifier),
                    ("UILayoutGuideOwningViewIsLocked", Value::Bool(false)),
                    ("UILayoutGuideShouldBeArchived", Value::Bool(false)),
                    ("UILayoutGuideAllowsNegativeDimensions", Value::Bool(false)),
                ]).named("safe area")));
            }
            objects.extend(constraint_names.iter().map(|n| reference(n.clone())));
            objects.extend(scene.guides.iter().map(|g| reference(format!("guide {}", g.1))));
            // Then breadth first: each view in turn, from a queue, lists the
            // constraints it holds and then its subviews, which join the queue.
            let mut queue: std::collections::VecDeque<Option<&String>> = [None].into();
            // The views in the objects' order: the root, then breadth first.
            let mut order = vec![root_name.clone()];
            while let Some(v) = queue.pop_front() {
                let subviews_first = v.is_some_and(|v| scene.effect_contents.contains(v));
                let constraints = |v: &String| scene.own_constraints.iter().filter(|c| &c.0 == v).map(|c| reference(c.1.clone())).collect::<Vec<_>>();
                if let Some(v) = v.filter(|_| !subviews_first) {
                    objects.extend(constraints(v));
                }
                for (_, s) in scene.subviews.iter().filter(|s| s.0.as_ref() == v && !pulled.contains(&s.1)) {
                    objects.push(reference(s.clone()));
                    order.push(s.clone());
                    queue.push_back(Some(s));
                }
                if let Some(v) = v.filter(|_| subviews_first) {
                    objects.extend(constraints(v));
                }
                if v.is_none() {
                    for p in &pulled {
                        objects.push(reference(p.clone()));
                        order.push(p.clone());
                        queue.push_back(Some(p));
                    }
                }
            }
            // Runtime attributes and accessibility configurations go in that
            // order too, each view's as the document gives them (eight
            // probes, and six of accessibility on several views), but one
            // corpus storyboard's views' attributes don't, for reasons not
            // established: attributes of more than one view, or beside a
            // switch's frame, are refused.
            let distinct = |names: Vec<&String>| names.into_iter().collect::<std::collections::HashSet<_>>().len();
            let user_views = distinct(scene.pairs.iter().map(|p| &p.0).filter(|n| !scene.switch_frames.contains(n)).collect());
            if user_views > 1 || (user_views == 1 && !scene.switch_frames.is_empty()) {
                return fail("user-defined runtime attributes of more than one view, or beside a switch, are not supported by this ibtool");
            }
            let position = |name: &String| order.iter().position(|o| o == name).unwrap_or(usize::MAX);
            scene.pairs.sort_by_key(|p| position(&p.0));
            scene.accessibility.sort_by_key(|a| position(&a.0));

            // Outlets, `view` among them, then actions, each in byte order of
            // their names; actions with the same selector in document order.
            let mut connections = Vec::new();
            let mut outlets: Vec<(&str, &str, &str)> = vc_outlets.iter().map(|(l, d)| (l.as_str(), "upstream", d.as_str())).collect();
            outlets.push(("view", "owner", &root_name));
            outlets.extend(scene.containers.iter().map(|(p, v)| ("containerView", p.as_str(), v.as_str())));
            outlets.extend(scene.control_outlets.iter().map(|(l, v)| (l.as_str(), v.as_str(), "upstream")));
            for (label, source, destination) in &scene.view_outlets {
                if *destination != root_name && !scene.subviews.iter().any(|s| s.1 == *destination) {
                    return fail("a view's outlets to anything but the controller and its views are not supported by this ibtool");
                }
                outlets.push((label, source, destination));
            }
            outlets.sort_by(|a, b| a.0.cmp(b.0));
            // A view's gesture recognizers are an outlet collection, before
            // the outlets.
            // The root view's first, as the objects list it (one probe).
            let mut gesture_sources: Vec<&str> = scene.gesture_views.iter().map(|(v, _)| v.as_str()).collect();
            gesture_sources.sort_by_key(|v| *v != root_name);
            outlets.splice(0..0, gesture_sources.into_iter().map(|v| ("gestureRecognizers", v, "")));
            // The controller's outlet collections, before its outlets.
            for (label, class, views) in &vc_collections {
                for v in views {
                    if *v != root_name && !scene.subviews.iter().any(|s| s.1 == *v) {
                        return fail("an outlet collection of anything but the controller's views is not supported by this ibtool");
                    }
                }
                let views = views.iter().map(|v| reference(v.clone())).collect();
                connections.push(outlet_collection(&mut nib, label, "upstream", views, class, false));
            }
            for (label, source, destination) in outlets {
                if label == "gestureRecognizers" {
                    let gestures = &scene.gesture_views.iter().find(|(v, _)| v == source).expect("listed above").1;
                    let gestures = gestures.iter().map(|g| reference(format!("gesture {g}"))).collect();
                    connections.push(outlet_collection(&mut nib, label, source, gestures, "NSArray", true));
                } else {
                    connections.push(outlet(&mut nib, label, source, destination));
                }
            }
            // By event, then selector.
            scene.actions.sort_by(|a, b| (a.2, &a.0).cmp(&(b.2, &b.0)));
            for (selector, source, mask, destination) in &scene.actions {
                let label = nib.string(selector);
                connections.push(Value::New(Object::new("UIRuntimeEventConnection", vec![
                    ("UILabel", label),
                    ("UISource", reference(source.clone())),
                    ("UIDestination", reference(destination.clone())),
                    ("UIEventMask", Value::Int(*mask)),
                ])));
            }
            let pairs = std::mem::take(&mut scene.pairs).into_iter().map(|p| p.1).collect();
            let accessibility = std::mem::take(&mut scene.accessibility).into_iter().map(|a| a.1).collect();
            (root_object_full(&mut nib, top, objects, connections, accessibility, pairs), upstream_used)
        };

        // The view controller's nib: the scene's placeholders and a class
        // swapper standing in for the controller, which loads the view nib.
        if let Some(identifier) = vc.attribute("storyboardIdentifier") {
            // Xcode can't write a name with a slash, and writes a non-ASCII
            // one decomposed.
            if identifier.is_empty() || identifier.contains('/') || !identifier.is_ascii() {
                return fail(format!("storyboard identifier {identifier:?} is not supported by this ibtool"));
            }
        }
        // A controller with a storyboard identifier is loaded by it.
        let vc_nib_name = vc.attribute("storyboardIdentifier").map_or_else(|| format!("{original_class}-{vc_id}"), str::to_string);
        let view_nib_name = format!("{vc_id}-view-{view_id}");
        // A scene is loaded when it is initial or identified, when a segue
        // leads to it, or when a navigation controller embeds it; it is
        // swapped in for its class only when initial, identified or of a
        // custom class.
        let is_initial = initial == Some(vc_id);
        let identified = vc.attribute("storyboardIdentifier").is_some();
        let segue_destination = reached.contains(vc_id);
        let embedded = navs.iter().any(|n| n.2 == vc_id && n.3) || held_children.contains(vc_id) || in_held_nav(vc_id);
        let embedded_here = embedded;
        let swapped = is_initial || identified || vc.attribute("customClass").is_some();
        let view_segues = listed_segues(&scene, vc);

        let mut vnib = Nib::new(platform, color_matched, localizable);
        let owner = proxy(&mut vnib, "IBFilesOwner", "owner");
        let responder = proxy(&mut vnib, "IBFirstResponder", "responder");
        let storyboard = proxy(&mut vnib, "UIStoryboardPlaceholder", "storyboard");
        let mut swapper = Vec::new();
        if let Some(title) = vc.attribute("title") {
            swapper.push(("UITitle", vnib.text(vc_id, "title", title)));
        }
        // Its tab bar item, the document's, comes before its navigation item.
        let has_tab_item = child(vc, "tabBarItem", "tabBarItem").is_some();
        if has_tab_item {
            swapper.push(("UITabBarItem", tab::tab_bar_item(&mut vnib, vc, "tab bar item")?.0));
        }
        let mut bar_items = Vec::new();
        let mut bar_actions = Vec::new();
        let mut bar_segues = Vec::new();
        if let Some(item) = child(vc, "navigationItem", "navigationItem") {
            let parts = navigation_item(&mut vnib, item, "navigation item", vc_id)?;
            swapper.push(("UINavigationItem", parts.value));
            bar_items = parts.bar_items;
            bar_actions = parts.actions;
            bar_segues = parts.segues;
            if has_tab_item && !bar_items.is_empty() {
                return fail("a controller with both a tab bar item and bar button items is not supported by this ibtool");
            }
        }
        let templates = segue::templates(&mut vnib, "controller", upstream_used, 1 + gesture, &view_segues, &own_segues, &bar_segues, &nib_names)?;
        let mut external = templates.external;
        // The recognizers: the nib's objects in reverse document order,
        // kept alive in document order, mapped to their placeholders.
        // A navigation controller's root that nothing loads on its own has
        // them in the navigation controller's nib (two probes).
        let nav_root_only = navs.iter().any(|n| n.2 == vc_id && n.3) && !held_children.contains(vc_id) && !in_held_nav(vc_id);
        if !recognizers.is_empty() {
            if embedded_here && !(nav_root_only && !is_initial && !identified && !segue_destination) {
                return fail("a gesture recognizer in an embedded controller's scene is not supported by this ibtool");
            }
            if extended_edges(vc)?.is_some() || vc.attribute("automaticallyAdjustsScrollViewInsets").is_some() {
                return fail("a gesture recognizer with these controller settings is not supported by this ibtool");
            }
        }
        let mut gesture_objects = Vec::new();
        for r in recognizers.iter().rev() {
            let (object, selectors) = gesture_recognizer(&mut vnib, *r, vc_id)?;
            gesture_objects.push((Value::New(object.named(format!("gesture {}", attr(*r, "id")?))), attr(*r, "id")?, selectors));
        }
        let mut placeholders = Vec::new();
        for (i, g) in gesture_order(&scene.gesture_views).iter().enumerate() {
            placeholders.extend([vnib.string(&format!("UpstreamPlaceholder-{}", i + 1)), reference(format!("gesture {g}"))]);
        }
        external.splice(0..0, placeholders);
        let external = vnib.array("NSDictionary", external);
        swapper.push(("UIExternalObjectsTableForViewLoading", external));
        swapper.push(("UINibName", vnib.string(&view_nib_name)));
        // Its own restoration identifier after the nib name (one probe).
        if let Some(identifier) = vc.attribute("restorationIdentifier") {
            if vc.attribute("storyboardIdentifier").is_some() {
                return fail("a controller's restoration identifier beside a storyboard identifier is not supported by this ibtool");
            }
            swapper.push(("UIRestorationIdentifier", vnib.string(identifier)));
        }
        if let Some(identifier) = vc.attribute("storyboardIdentifier") {
            swapper.push(("UIStoryboardIdentifier", vnib.string(identifier)));
            if vc.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
                swapper.push(("UIRestorationIdentifier", vnib.string(identifier)));
            }
        } else if vc.attribute("useStoryboardIdentifierAsRestorationIdentifier").is_some() {
            return fail("useStoryboardIdentifierAsRestorationIdentifier without a storyboard identifier is not supported by this ibtool");
        }
        if let Some(array) = templates.array {
            swapper.push(("UIStoryboardSegueTemplates", array));
        }
        if !recognizers.is_empty() {
            let alive = recognizers.iter().map(|r| attr(*r, "id").map(|id| reference(format!("gesture {id}")))).collect::<Result<_, _>>()?;
            swapper.push(("UITopLevelObjectsToKeepAliveFromStoryboard", vnib.array("NSArray", alive)));
        }
        if let Some(edges) = extended_edges(vc)? {
            swapper.push(("UIKeyEdgesForExtendedLayout", Value::Int(edges)));
        }
        if vc.attribute("automaticallyAdjustsScrollViewInsets").is_some() {
            swapper.push(("UIKeyAutomaticallyAdjustsScrollViewInsets", Value::Int(0)));
        }
        if vc.attribute("clearsSelectionOnViewWillAppear").is_some() {
            swapper.push(("UIClearsSelectionOnViewWillAppear", Value::Bool(false)));
        }
        let controller = if swapped {
            swapper.push(("UIClassName", vnib.string(&class_name)));
            swapper.push(("UIOriginalClassName", vnib.string(original_class)));
            Object::new("UIClassSwapper", swapper)
        } else {
            Object::new(original_class, swapper)
        };
        let controller = Value::New(controller.named("controller"));
        let mut connections = vec![outlet(&mut vnib, "storyboard", "controller", "storyboard"), outlet(&mut vnib, "sceneViewController", "owner", "controller")];
        for t in &templates.outlets {
            connections.push(outlet(&mut vnib, "viewController", t, "controller"));
        }
        connections.extend(bar_events(&mut vnib, &bar_items, &bar_actions, &templates.performs, "controller"));
        let mut top = vec![owner, responder, storyboard];
        if !recognizers.is_empty() && !bar_items.is_empty() {
            return fail("a gesture recognizer with bar button items is not supported by this ibtool");
        }
        let mut objects = vec![reference("owner"), reference("responder"), reference("storyboard")];
        // Their actions by selector, those with one selector in document
        // order (two probes).
        let mut gesture_actions: Vec<(String, &str)> =
            gesture_objects.iter().rev().flat_map(|(_, id, selectors)| selectors.iter().map(|s| (s.clone(), *id))).collect();
        gesture_actions.sort_by(|a, b| a.0.cmp(&b.0));
        for (object, id, _) in gesture_objects {
            top.push(object);
            objects.push(reference(format!("gesture {id}")));
        }
        for (selector, id) in gesture_actions {
            connections.push(event_connection(&mut vnib, &selector, &format!("gesture {id}"), "controller"));
        }
        objects.push(reference("controller"));
        if child(vc, "navigationItem", "navigationItem").is_some() {
            objects.push(reference("navigation item"));
        }
        if has_tab_item {
            objects.push(reference("tab bar item"));
        }
        objects.extend(templates.objects);
        objects.extend(bar_items.iter().map(|b| reference(b.clone())));
        top.push(controller);
        // The controller's runtime attributes are its own nib's.
        let pairs = runtime_attributes(&mut vnib, vc, "controller")?;
        if !pairs.is_empty() && embedded {
            return fail("an embedded controller's runtime attributes are not supported by this ibtool");
        }
        let vc_root = root_object_with_pairs(&mut vnib, top, objects, connections, pairs);
        if embedded {
            roots.insert(vc_id, RootController {
                vc,
                loadable: swapped,
                view_nib_name: view_nib_name.clone(),
                class_name: class_name.clone(),
                original_class,
                upstream_used,
                view_segues: view_segues.clone(),
                own_segues: own_segues.clone(),
                recognizers: recognizers.clone(),
                gesture_order: gesture_order(&scene.gesture_views).into_iter().cloned().collect(),
            });
        }
        if !is_initial && !identified && !segue_destination && !embedded {
            continue;
        }
        let mut scene_files = vec![(format!("{view_nib_name}.nib"), nib::archive(view_root))];
        let mut entry = None;
        if is_initial || identified || segue_destination {
            scene_files.push((format!("{vc_nib_name}.nib"), nib::archive(vc_root)));
            entry = Some((vc_nib_name.clone(), is_initial, identified));
        }
        outputs.push((index, scene_files, entry));
    }
    for &(index, nav, root_id, loadable) in &navs {
        if !loadable && held_children.contains(attr(nav, "id")?) {
            // Held by its parent's nib alone.
            continue;
        }
        if !loadable {
            return fail("a navigation controller nothing loads is not supported by this ibtool");
        }
        if held_children.contains(attr(nav, "id")?) {
            return fail("a child navigation controller that is loaded on its own too is not supported by this ibtool");
        }
        let root = roots.get(root_id).ok_or("a navigation controller's root view controller is not in a supported scene")?;
        let swapped = initial == Some(attr(nav, "id")?) || nav.attribute("storyboardIdentifier").is_some();
        let (name, bytes) = navigation_controller_nib(platform, color_matched, localizable, module, nav, root, swapped, &nib_names)?;
        let entry = (name.clone(), initial == Some(attr(nav, "id")?), nav.attribute("storyboardIdentifier").is_some());
        outputs.push((index, vec![(format!("{name}.nib"), bytes)], Some(entry)));
    }
    for (index, kind, node, kids, loadable) in &parents {
        if !loadable {
            return fail(format!("a {} nothing loads is not supported by this ibtool", kind.class()));
        }
        let mut held = Vec::new();
        for &(relationship, id) in kids {
            if let Some(&(_, nav, root_id, _)) = navs.iter().find(|n| attr(n.1, "id").ok() == Some(id)) {
                let root = roots.get(root_id).ok_or("a navigation controller's root view controller is not in a supported scene")?;
                held.push((relationship, parent::Child::Navigation(nav, root)));
            } else {
                let root = roots.get(id).ok_or("a child controller is not in a supported scene")?;
                held.push((relationship, parent::Child::Controller(root)));
            }
        }
        let id = attr(*node, "id")?;
        let swapped = initial == Some(id) || node.attribute("storyboardIdentifier").is_some();
        let (name, bytes) = parent::parent_controller_nib(platform, color_matched, localizable, module, *kind, *node, &held, swapped, &nib_names)?;
        let entry = (name.clone(), initial == Some(id), node.attribute("storyboardIdentifier").is_some());
        outputs.push((*index, vec![(format!("{name}.nib"), bytes)], Some(entry)));
    }
    outputs.sort_by_key(|o| o.0);
    let mut files = Vec::new();
    let mut identifiers = Vec::new();
    // The Info.plist lists the controllers in the order they load.
    let order = segue::load_order(&controllers, initial);
    let position = |index: usize| segue::scene_controller(scene_nodes[index]).and_then(|c| c.attribute("id")).and_then(|id| order.iter().position(|o| *o == id));
    let mut entries = Vec::new();
    for (index, scene_files, entry) in outputs {
        files.extend(scene_files);
        entries.extend(entry.map(|e| (position(index), e)));
    }
    entries.sort_by_key(|e| e.0.unwrap_or(usize::MAX));
    identifiers.extend(entries.into_iter().map(|e| e.1));
    // Xcode renames nibs whose names differ only in case and keeps one of
    // equal identifiers.
    let mut names: Vec<String> = files.iter().map(|(n, _)| n.to_lowercase()).collect();
    names.sort();
    if names.windows(2).any(|w| w[0] == w[1]) {
        return fail("storyboard identifiers that are equal ignoring case are not supported by this ibtool");
    }
    let mut info = Vec::new();
    // The entry point is the same object as its identifier's key when the
    // document names it; a generated name is made twice.
    let mut shared_entry = None;
    if initial.is_some() {
        let entry = identifiers.iter().find(|i| i.1).ok_or("the initial view controller is not in a supported scene")?;
        info.push(("UIStoryboardDesignatedEntryPointIdentifier", cf::Value::string(entry.0.clone())));
        if entry.2 {
            shared_entry = Some(entry.0.clone());
        }
    }
    info.push(("UIStoryboardVersion", cf::Value::Integer(1)));
    info.push(("UIViewControllerIdentifiersToNibNames", cf::Value::dict(identifiers.iter().map(|(n, ..)| (n.clone(), cf::Value::string(n.clone()))))));
    if !references.is_empty() {
        info.push(("UIViewControllerIdentifiersToExternalStoryboardReferences", cf::Value::dict(references)));
    }
    let info = cf::Value::dict(info);
    // In the external references, equal keys and values are one string
    // each (two probes).
    let share = |_: &cf::Value, at: &cf::binary::Position| {
        use cf::binary::Step;
        if let [Step::Value(k), Step::Value(_), _] = at {
            return k == "UIViewControllerIdentifiersToExternalStoryboardReferences";
        }
        let Some(entry) = &shared_entry else { return false };
        match at {
            [Step::Value(k)] => k == "UIStoryboardDesignatedEntryPointIdentifier",
            [Step::Value(k), Step::Key(id)] => k == "UIViewControllerIdentifiersToNibNames" && id == entry,
            _ => false,
        }
    };
    files.push(("Info.plist".into(), cf::binary::to_bytes_with(&info, &share, cf::binary::Dictionaries::Grown)));
    Ok(Compiled { files })
}
