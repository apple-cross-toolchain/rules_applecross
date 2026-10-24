//! The supported subset of iOS and tvOS storyboards and the nibs it compiles
//! to; see docs/clean-room/ibtool.md.

use crate::fonts;
use crate::nib::{self, reference, Object, Value};
use roxmltree::{Document, Node};
use std::collections::HashMap;

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

fn only_children(n: Node, allowed: &[&str]) -> Result<(), String> {
    for c in n.children().filter(|c| c.is_element()) {
        if !allowed.contains(&c.tag_name().name()) {
            return fail(format!("<{}> inside <{}> is not supported by this ibtool", c.tag_name().name(), n.tag_name().name()));
        }
    }
    Ok(())
}

/// Refuses attributes other than `allowed`, and values other than those
/// given (a `None` value accepts any).
fn only_attributes(n: Node, allowed: &[(&str, Option<&str>)]) -> Result<(), String> {
    for a in n.attributes() {
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

/// The system colours a document declares, which must be iOS ones this
/// ibtool knows; their values are Xcode's, not the document's.
fn system_colors(doc: &Document, platform: Platform) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for r in doc.root_element().children().filter(|c| c.has_tag_name("resources")) {
        for sc in children(r, "systemColor") {
            let name = attr(sc, "name")?.to_string();
            if platform != Platform::Ios {
                return fail(format!("system color {name}: system colors outside iOS are not supported by this ibtool"));
            }
            if !SYSTEM_COLORS.iter().any(|(n, _)| *n == name) {
                return fail(format!("system color {name} is not supported by this ibtool"));
            }
            out.push(name);
        }
    }
    Ok(out)
}

/// Colour components as Interface Builder archives them: each rounded to 10
/// significant digits, as a float followed by the double when the float
/// isn't exact, then the components as text (`%.3g`, alpha only when not 1).
fn color_components(names: &[(&'static str, &'static str)], values: &[f64]) -> (Vec<(&'static str, Value)>, Vec<u8>) {
    let values: Vec<f64> = values.iter().map(|&x| significant_10(x)).collect();
    let mut keys = Vec::new();
    for (&(key, double_key), &x) in names.iter().zip(&values) {
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
    shared: HashMap<String, String>,
    next: usize,
}

impl Nib {
    fn new(platform: Platform) -> Nib {
        Nib { platform, shared: HashMap::new(), next: 0 }
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

fn color(nib: &mut Nib, n: Node, colors: &[String]) -> Result<Value, String> {
    // Xcode 11's form names the system colour (and an older fallback), the
    // older form only a UIKit class colour; neither is declared in the
    // document. Both are archived as that system colour.
    if let Some(name) = n.attribute("xcode11CocoaTouchSystemColor").or_else(|| n.attribute("cocoaTouchSystemColor")) {
        only_attributes(n, &[("key", None), ("xcode11CocoaTouchSystemColor", None), ("cocoaTouchSystemColor", None)])?;
        let sc = &SYSTEM_COLORS.iter().find(|(c, _)| *c == name).ok_or_else(|| format!("system color {name} is not supported by this ibtool"))?.1;
        return Ok(nib.system_color(name, sc));
    }
    if let Some(name) = n.attribute("systemColor") {
        only_attributes(n, &[("key", None), ("systemColor", None)])?;
        if !colors.iter().any(|c| c == name) {
            return fail(format!("system color {name} is not defined in the document"));
        }
        let sc = &SYSTEM_COLORS.iter().find(|(c, _)| *c == name).unwrap().1;
        return Ok(nib.system_color(name, sc));
    }
    // Equal colours are one object, compared at 10 significant digits. A
    // grey is archived as given (no conversion), named when it's exactly
    // black, white or clear.
    if n.attribute("customColorSpace") == Some("genericGamma22GrayColorSpace") {
        only_attributes(n, &[("key", None), ("white", None), ("alpha", None), ("colorSpace", Some("custom")), ("customColorSpace", None)])?;
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

/// A text style's font as Xcode archives it: name, traits and the size
/// recorded for NSFont, per platform.
fn text_style(platform: Platform, style: &str) -> Result<(&'static str, i64, f64), String> {
    Ok(match (platform, style) {
        (Platform::Ios, "UICTFontTextStyleHeadline") => (".SFUI-Semibold", 2, 14.0),
        (Platform::Ios, "UICTFontTextStyleCallout") => (".SFUI-Regular", 0, 13.0),
        (Platform::Tvos, "UICTFontTextStyleTitle1") => (".SFUI-Medium", 0, 76.0),
        _ => return fail(format!("the {style} text style is not supported by this ibtool here")),
    })
}

/// A font: `<fontDescription>`, or a control's default system font. Equal
/// fonts are one object.
fn font(nib: &mut Nib, n: Option<Node>, default_size: Option<f64>) -> Result<Value, String> {
    let (style, name, size) = match n {
        Some(n) => {
            if let Some(style) = n.attribute("style") {
                only_attributes(n, &[("key", None), ("style", None)])?;
                (Some(style), None, 0.0)
            } else if let Some(name) = n.attribute("name") {
                only_attributes(n, &[("key", None), ("name", None), ("family", None), ("pointSize", None)])?;
                (None, Some(name), number(attr(n, "pointSize")?)?)
            } else {
                only_attributes(n, &[("key", None), ("type", Some("system")), ("pointSize", None)])?;
                (None, None, number(attr(n, "pointSize")?)?)
            }
        }
        None => (None, None, default_size.ok_or("a font is required")?),
    };
    // Xcode archives a font it finds under its own name.
    let name = name.map(|name| fonts::ALIASES.iter().find(|(a, _)| *a == name).map_or(name, |(_, to)| to));
    let key = format!("font:{style:?}:{name:?}:{size}");
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
            let name = ".SFUI-Regular";
            values.push(("UIFontName", nib.string(name)));
            let usage_key = nib.string("NSCTFontUIUsageAttribute");
            let usage = nib.string("CTFontRegularUsage");
            let size_key = nib.string("NSFontSizeAttribute");
            let size_value = nib.double_number(size);
            let attributes = nib.array("NSDictionary", vec![usage_key, usage, size_key, size_value]);
            values.push(("UIFontDescriptor", Value::New(Object::new("UIFontDescriptor", vec![
                ("UIFontDescriptorAttributes", attributes),
                ("UIFontDescriptorOptions", Value::Int(2147517444)),
            ]))));
            (name, 0, size, true)
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

fn geometry(values: &mut Vec<(&'static str, Value)>, f: &Frame) {
    values.push(("UIBounds", doubles(&[0.0, 0.0, f.w, f.h])));
    values.push(("UICenter", doubles(&[f.x + f.w / 2.0, f.y + f.h / 2.0])));
}

fn view_name(id: &str) -> String {
    format!("view {id}")
}

/// What compiling a scene's view collects besides the view itself.
struct Scene<'a> {
    vc_id: &'a str,
    /// The document's system colours.
    colors: &'a [String],
    /// Subviews in depth-first preorder, for the nib's objects.
    subviews: Vec<String>,
    /// Control actions: (selector, source, event mask), in depth-first order.
    actions: Vec<(String, String, i64)>,
    /// Ids of the legacy top and bottom layout guides.
    guides: Vec<(&'a str, &'static str)>,
}

fn hugging(nib: &mut Nib, n: Node) -> Result<Value, String> {
    let h = attr(n, "horizontalHuggingPriority")?;
    let v = attr(n, "verticalHuggingPriority")?;
    Ok(nib.string(&format!("{{{h}, {v}}}")))
}

fn text_alignment(s: &str) -> Result<i64, String> {
    Ok(match s {
        "center" => 1,
        "natural" => 4,
        other => return fail(format!("textAlignment {other} is not supported by this ibtool")),
    })
}

fn subview(nib: &mut Nib, scene: &mut Scene, n: Node) -> Result<Value, String> {
    let id = attr(n, "id")?;
    let name = view_name(id);
    scene.subviews.push(name.clone());
    let f = frame(n)?;
    let mut values = Vec::new();
    geometry(&mut values, &f);
    let object = match n.tag_name().name() {
        "label" => {
            only_attributes(n, &[
                ("opaque", Some("NO")), ("userInteractionEnabled", Some("NO")), ("contentMode", Some("left")),
                ("horizontalHuggingPriority", None), ("verticalHuggingPriority", None), ("text", None), ("textAlignment", None),
                ("lineBreakMode", Some("tailTruncation")), ("numberOfLines", None), ("baselineAdjustment", Some("alignBaselines")),
                ("adjustsFontSizeToFit", Some("NO")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "fontDescription", "nil", "color"])?;
            if child(n, "nil", "highlightedColor").is_none() {
                return fail("labels with a highlighted colour are not supported by this ibtool");
            }
            let text_color = match (child(n, "color", "textColor"), child(n, "nil", "textColor")) {
                (Some(c), None) => Some(c),
                (None, Some(_)) => None,
                _ => return fail("a label needs one text colour or nil"),
            };
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIUserInteractionDisabled", Value::Bool(true)));
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            values.push(("UIContentMode", Value::Int(7)));
            if n.attribute("translatesAutoresizingMaskIntoConstraints").is_some() {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            values.push(("UIViewContentHuggingPriority", hugging(nib, n)?));
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            values.push(("UIText", nib.string(attr(n, "text")?)));
            values.push(("UIFont", font(nib, child(n, "fontDescription", "fontDescription"), None)?));
            // A declared text colour, even labelColor, stays as it is when
            // the trait collection changes.
            let declared_color = text_color.is_some();
            let text_color = match text_color {
                Some(c) => color(nib, c, scene.colors)?,
                None => nib.system_color("labelColor", &SystemColor::Gray([0.0, 1.0])),
            };
            values.push(("UITextColor", text_color));
            values.push(("UIShadowOffset", doubles(&[0.0, -1.0])));
            if let Some(lines) = n.attribute("numberOfLines") {
                values.push(("UINumberOfLines", Value::Int(lines.parse().map_err(|_| format!("not a number: {lines}"))?)));
            }
            values.push(("UITextAlignment", Value::Int(text_alignment(attr(n, "textAlignment")?)?)));
            values.push(("UIDisableUpdateTextColorOnTraitCollectionChange", Value::Bool(declared_color)));
            Object::new("UILabel", values)
        }
        "textField" => {
            only_attributes(n, &[
                ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("contentMode", Some("scaleToFill")),
                ("contentHorizontalAlignment", Some("left")), ("contentVerticalAlignment", Some("center")), ("borderStyle", Some("roundedRect")),
                ("textAlignment", Some("natural")), ("minimumFontSize", None), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "nil", "fontDescription", "textInputTraits"])?;
            let traits = children(n, "textInputTraits").next().ok_or("text field without text input traits")?;
            if traits.attributes().len() != 1 || traits.has_children() {
                return fail("text input traits are not supported by this ibtool");
            }
            if child(n, "nil", "textColor").is_none() {
                return fail("text fields with a text colour are not supported by this ibtool");
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            values.push(("UIClipsToBounds", Value::Bool(true)));
            if n.attribute("translatesAutoresizingMaskIntoConstraints").is_some() {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            values.push(("UIContentHorizontalAlignment", Value::Int(1)));
            values.push(("UIFont", font(nib, child(n, "fontDescription", "fontDescription"), None)?));
            values.push(("UITextAlignment", Value::Int(4)));
            values.push(("UITextColor", nib.system_color("labelColor", &SystemColor::Gray([0.0, 1.0]))));
            values.push(("UIBorderStyle", Value::Int(3)));
            values.push(("UIAdjustsFontSizeToFit", Value::Bool(true)));
            values.push(("UIMinimumFontSize", Value::Float(number(attr(n, "minimumFontSize")?)? as f32)));
            values.push(("UIClearButtonOffset", doubles(&[3.0, 1.0])));
            values.push(("UIDisableTextColorUpdateOnTraitCollectionChange", Value::Bool(true)));
            values.push(("UIRoundedRectBackgroundCornerRadius", Value::Double(0.0)));
            Object::new("UITextField", values)
        }
        "button" => {
            only_attributes(n, &[
                ("opaque", Some("NO")), ("contentMode", Some("scaleToFill")), ("contentHorizontalAlignment", Some("center")),
                ("contentVerticalAlignment", Some("center")), ("buttonType", Some("roundedRect")), ("lineBreakMode", Some("middleTruncation")),
                ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "state", "connections"])?;
            let states: Vec<Node> = children(n, "state").collect();
            if states.len() != 1 || states[0].attribute("key") != Some("normal") || states[0].attributes().len() != 2 || states[0].has_children() {
                return fail("buttons other than one with a normal-state title are not supported by this ibtool");
            }
            for c in children(n, "connections") {
                only_children(c, &["action"])?;
                for a in children(c, "action") {
                    only_attributes(a, &[("selector", None), ("destination", None), ("eventType", Some("touchUpInside")), ("id", None)])?;
                    if attr(a, "destination")? != scene.vc_id {
                        return fail("actions that don't target the scene's view controller are not supported by this ibtool");
                    }
                    scene.actions.push((attr(a, "selector")?.to_string(), name.clone(), 64));
                }
            }
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            if n.attribute("translatesAutoresizingMaskIntoConstraints").is_some() {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            let state = nib.int_number(0);
            let title = nib.string(attr(states[0], "title")?);
            let content = Value::New(Object::new("UIButtonContent", vec![("UITitle", title)]));
            values.push(("UIButtonStatefulContent", nib.array("NSMutableDictionary", vec![state, content])));
            values.push(("UIAdjustsImageWhenHighlighted", Value::Bool(true)));
            values.push(("UIAdjustsImageWhenDisabled", Value::Bool(true)));
            values.push(("UIButtonType", Value::Int(1)));
            values.push(("UIFont", font(nib, None, Some(15.0))?));
            values.push(("UIButtonConfiguration", Value::Nil));
            Object::new("UIButton", values)
        }
        "stackView" => {
            only_attributes(n, &[
                ("opaque", Some("NO")), ("contentMode", Some("scaleToFill")), ("axis", Some("vertical")), ("spacing", None),
                ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
            ])?;
            only_children(n, &["rect", "subviews"])?;
            let mut arranged = Vec::new();
            for s in children(n, "subviews") {
                for c in s.children().filter(|c| c.is_element()) {
                    arranged.push(subview(nib, scene, c)?);
                }
            }
            let refs = arranged_names(&arranged);
            values.push(("UISubviews", nib.array("NSMutableArray", arranged)));
            values.push(("UIDeepDrawRect", Value::Bool(true)));
            values.push(("UIAutoresizeSubviews", Value::Bool(true)));
            values.push(("UIAutoresizingMask", Value::Int(36)));
            if n.attribute("translatesAutoresizingMaskIntoConstraints").is_some() {
                values.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
            }
            values.push(("UIViewLayoutMargins", doubles(&[0.0, 0.0, 0.0, 0.0])));
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            values.push(("UIStackViewArrangedSubviews", nib.array("NSArray", refs)));
            values.push(("UIStackViewAxis", Value::Int(1)));
            values.push(("UIStackViewDistribution", Value::Int(0)));
            values.push(("UIStackViewAlignment", Value::Int(0)));
            values.push(("UIStackViewSpacing", Value::Double(number(attr(n, "spacing")?)?)));
            values.push(("UIStackViewBaselineRelative", Value::Bool(false)));
            values.push(("UIStackViewLayoutMarginsRelative", Value::Bool(false)));
            Object::new("UIStackView", values)
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

/// Where a view's constraints go in the nib, by first attribute: observed
/// for these attributes, whatever the document's order or ids.
fn constraint_rank(attribute: &str) -> Result<usize, String> {
    const ORDER: [&str; 8] = ["centerX", "centerY", "trailing", "leading", "width", "bottom", "top", "height"];
    ORDER.iter().position(|a| *a == attribute).ok_or_else(|| format!("constraints on {attribute} are not supported by this ibtool"))
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

fn root_object(nib: &mut Nib, top: Vec<Value>, objects: Vec<Value>, connections: Vec<Value>) -> Object {
    let top = nib.array("NSArray", top);
    let objects = nib.array("NSArray", objects);
    let connections = nib.array("NSArray", connections);
    let empty = Value::New(Object::new("NSArray", vec![("NSInlinedValue", Value::Bool(true))]).named("empty"));
    Object::new("NSObject", vec![
        ("UINibTopLevelObjectsKey", top),
        ("UINibObjectsKey", objects),
        ("UINibConnectionsKey", connections),
        ("UINibVisibleWindowsKey", empty),
        ("UINibAccessibilityConfigurationsKey", reference("empty")),
        ("UINibTraitStorageListsKey", reference("empty")),
        ("UINibKeyValuePairsKey", reference("empty")),
        ("IBUINibLNEVersionKey", Value::Int(1)),
    ])
}

pub fn compile(xml: &str, minimum_deployment_target: Option<&str>) -> Result<Compiled, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    let platform = match root.attribute("type") {
        Some("com.apple.InterfaceBuilder3.CocoaTouch.Storyboard.XIB") => Platform::Ios,
        Some("com.apple.InterfaceBuilder.AppleTV.Storyboard") => Platform::Tvos,
        _ => return fail("not an iOS or tvOS storyboard"),
    };
    // Without a <device>, Interface Builder lays an iOS scene out on its
    // generic 600x600 canvas below iOS 17, and at the view's frame from
    // iOS 17 on; Xcode's ibtool fails on a tvOS one.
    let has_device = children(root, "device").next().is_some();
    let generic_canvas = if has_device {
        false
    } else if platform == Platform::Tvos {
        return fail("a tvOS storyboard without a device is not compiled by Xcode's ibtool");
    } else {
        let target = minimum_deployment_target.ok_or("a storyboard without a device needs --minimum-deployment-target for this ibtool")?;
        let major: u32 = target.split('.').next().and_then(|m| m.parse().ok()).ok_or_else(|| format!("not a deployment target: {target}"))?;
        major < 17
    };
    let colors = system_colors(&doc, platform)?;
    let initial = attr(root, "initialViewController")?;
    let mut files = Vec::new();
    let mut identifiers = Vec::new();
    let scenes = children(root, "scenes").next().ok_or("storyboard has no scenes")?;
    for scene_node in children(scenes, "scene") {
        only_children(scene_node, &["objects", "point"])?;
        let objects_node = children(scene_node, "objects").next().ok_or("scene without objects")?;
        only_children(objects_node, &["viewController", "placeholder"])?;
        let vc = children(objects_node, "viewController").next().ok_or("scene without a view controller")?;
        only_attributes(vc, &[("id", None), ("customClass", None), ("sceneMemberID", Some("viewController"))])?;
        only_children(vc, &["view", "layoutGuides", "connections"])?;
        let vc_id = attr(vc, "id")?;
        let view = children(vc, "view").next().ok_or("view controller without a view")?;
        only_children(view, &["rect", "autoresizingMask", "viewLayoutGuide", "color", "subviews", "constraints"])?;
        only_attributes(view, &[("key", Some("view")), ("contentMode", Some("scaleToFill")), ("id", None)])?;
        let view_id = attr(view, "id")?;
        let root_name = view_name(view_id);

        let mut scene = Scene { vc_id, colors: &colors, subviews: Vec::new(), actions: Vec::new(), guides: Vec::new() };
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
        let has_safe_area = children(view, "viewLayoutGuide").any(|g| g.attribute("key") == Some("safeArea"));
        let has_subviews = children(view, "subviews").next().is_some();
        if has_safe_area == !scene.guides.is_empty() {
            return fail("a view needs either a safe-area layout guide or legacy top and bottom layout guides for this ibtool");
        }
        if has_safe_area && has_subviews {
            return fail("subviews of a view with a safe-area layout guide are not supported by this ibtool");
        }
        if !has_device && has_subviews {
            return fail("subviews in a storyboard without a device are not supported by this ibtool");
        }

        // Outlets from the view controller into its view.
        let mut vc_outlets = Vec::new();
        for c in children(vc, "connections") {
            only_children(c, &["outlet"])?;
            for o in children(c, "outlet") {
                only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
                vc_outlets.push((attr(o, "property")?.to_string(), view_name(attr(o, "destination")?)));
            }
        }

        let mut nib = Nib::new(platform);
        let f = if generic_canvas { Frame { x: 0.0, y: 0.0, w: 600.0, h: 600.0 } } else { frame(view)? };
        let mask = match children(view, "autoresizingMask").next() {
            Some(m) => autoresizing_mask(m)?,
            None => 0,
        };
        let background = child(view, "color", "backgroundColor").ok_or("view without a background color")?;

        // The view's nib, in encoding order.
        let owner = proxy(&mut nib, "IBFilesOwner", "owner");
        let responder = proxy(&mut nib, "IBFirstResponder", "responder");
        let upstream_used = !vc_outlets.is_empty() || view.descendants().any(|d| d.has_tag_name("action"));
        let mut top = vec![owner, responder];
        let mut objects = vec![reference("owner"), reference("responder")];
        if upstream_used {
            top.push(proxy(&mut nib, "UpstreamPlaceholder-1", "upstream"));
            objects.push(reference("upstream"));
        }

        let mut values = Vec::new();
        geometry(&mut values, &f);
        let mut subview_values = Vec::new();
        for s in children(view, "subviews") {
            for c in s.children().filter(|c| c.is_element()) {
                subview_values.push(subview(&mut nib, &mut scene, c)?);
            }
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
        values.push(("UIBackgroundColor", color(&mut nib, background, &colors)?));
        values.push(("UIDeepDrawRect", Value::Bool(true)));
        values.push(("UIOpaque", Value::Bool(true)));
        values.push(("UIAutoresizeSubviews", Value::Bool(true)));
        values.push(("UIAutoresizingMask", Value::Int(mask)));

        // The view's constraints: the guides' first, then the document's in
        // the order Xcode gives them, by first attribute.
        let item = |id: &str| -> Value {
            match scene.guides.iter().find(|g| g.0 == id) {
                Some((_, kind)) => reference(format!("guide {kind}")),
                None => reference(view_name(id)),
            }
        };
        let mut xml_constraints: Vec<(usize, Node, &str, Option<&str>)> = Vec::new();
        for cs in children(view, "constraints") {
            for c in children(cs, "constraint") {
                only_attributes(c, &[("firstItem", None), ("firstAttribute", None), ("secondItem", None), ("secondAttribute", None), ("constant", None), ("id", None)])?;
                let first = c.attribute("firstItem").unwrap_or(view_id);
                let rank = constraint_rank(attr(c, "firstAttribute")?)?;
                if xml_constraints.iter().any(|(r, ..)| *r == rank) {
                    return fail("two constraints on the same first attribute are not supported by this ibtool");
                }
                xml_constraints.push((rank, c, first, c.attribute("secondItem")));
            }
        }
        xml_constraints.sort_by_key(|c| c.0);
        let mut constraint_values: Vec<Value> = guide_constraints.iter().map(|n| reference(n.clone())).collect();
        let mut constraint_names = Vec::new();
        for (_, c, first, second) in &xml_constraints {
            let second = match second {
                Some(s) => Some((item(s), layout_attribute(attr(*c, "secondAttribute")?)?)),
                None => None,
            };
            let constant = c.attribute("constant").map(number).transpose()?.unwrap_or(0.0);
            let name = nib.fresh();
            constraint_names.push(name.clone());
            match constraint(item(first), layout_attribute(attr(*c, "firstAttribute")?)?, second, constant, None) {
                Value::New(o) => constraint_values.push(Value::New(o.named(name))),
                _ => unreachable!(),
            }
        }
        if !constraint_values.is_empty() {
            values.push(("UIViewAutolayoutConstraints", nib.array("NSMutableArray", constraint_values)));
        }
        values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
        nib.large_content(&mut values);
        top.push(Value::New(Object::new("UIView", values).named(root_name.clone())));

        objects.push(reference(root_name.clone()));
        objects.extend(constraint_names.iter().map(|n| reference(n.clone())));
        objects.extend(scene.guides.iter().map(|g| reference(format!("guide {}", g.1))));
        objects.extend(scene.subviews.iter().map(|n| reference(n.clone())));
        if has_safe_area {
            let identifier = nib.string("");
            objects.push(Value::New(Object::new("UILayoutGuide", vec![
                ("UILayoutGuideOwningView", Value::Nil),
                ("UILayoutGuideIdentifier", identifier),
                ("UILayoutGuideOwningViewIsLocked", Value::Bool(false)),
                ("UILayoutGuideShouldBeArchived", Value::Bool(false)),
                ("UILayoutGuideAllowsNegativeDimensions", Value::Bool(false)),
            ])));
        }

        let mut connections = Vec::new();
        for (label, destination) in &vc_outlets {
            connections.push(outlet(&mut nib, label, "upstream", destination));
        }
        connections.push(outlet(&mut nib, "view", "owner", &root_name));
        for (selector, source, mask) in &scene.actions {
            let label = nib.string(selector);
            connections.push(Value::New(Object::new("UIRuntimeEventConnection", vec![
                ("UILabel", label),
                ("UISource", reference(source.clone())),
                ("UIDestination", reference("upstream")),
                ("UIEventMask", Value::Int(*mask)),
            ])));
        }
        let view_root = root_object(&mut nib, top, objects, connections);

        // The view controller's nib: the scene's placeholders and a class
        // swapper standing in for the controller, which loads the view nib.
        let vc_nib_name = format!("UIViewController-{vc_id}");
        let view_nib_name = format!("{vc_id}-view-{view_id}");
        let mut vnib = Nib::new(platform);
        let owner = proxy(&mut vnib, "IBFilesOwner", "owner");
        let responder = proxy(&mut vnib, "IBFirstResponder", "responder");
        let storyboard = proxy(&mut vnib, "UIStoryboardPlaceholder", "storyboard");
        let external = if upstream_used { vec![vnib.string("UpstreamPlaceholder-1"), reference("controller")] } else { Vec::new() };
        let external = vnib.array("NSDictionary", external);
        let nib_name = vnib.string(&view_nib_name);
        let class_name = vnib.string(vc.attribute("customClass").unwrap_or("UIViewController"));
        let original = vnib.string("UIViewController");
        let swapper = Value::New(Object::new("UIClassSwapper", vec![
            ("UIExternalObjectsTableForViewLoading", external),
            ("UINibName", nib_name),
            ("UIClassName", class_name),
            ("UIOriginalClassName", original),
        ]).named("controller"));
        let connections = vec![outlet(&mut vnib, "storyboard", "controller", "storyboard"), outlet(&mut vnib, "sceneViewController", "owner", "controller")];
        let vc_root = root_object(
            &mut vnib,
            vec![owner, responder, storyboard, swapper],
            vec![reference("owner"), reference("responder"), reference("storyboard"), reference("controller")],
            connections,
        );
        files.push((format!("{vc_nib_name}.nib"), nib::archive(vc_root)));
        files.push((format!("{view_nib_name}.nib"), nib::archive(view_root)));
        identifiers.push((vc_nib_name.clone(), vc_id == initial));
    }
    let entry = identifiers.iter().find(|i| i.1).ok_or("the initial view controller is not in a supported scene")?;
    let info = cf::Value::dict([
        ("UIStoryboardDesignatedEntryPointIdentifier", cf::Value::string(entry.0.clone())),
        ("UIStoryboardVersion", cf::Value::Integer(1)),
        ("UIViewControllerIdentifiersToNibNames", cf::Value::dict(identifiers.iter().map(|(n, _)| (n.clone(), cf::Value::string(n.clone()))))),
    ]);
    files.push(("Info.plist".into(), cf::binary::to_bytes(&info)));
    Ok(Compiled { files })
}
