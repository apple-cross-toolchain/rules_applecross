//! AppKit xibs (macOS, `com.apple.InterfaceBuilder3.Cocoa.XIB`), compiled
//! to the nibs Xcode's ibtool makes of them: see docs/clean-room/ibtool.md
//! (AppKit).
//!
//! A nib's root is an `NSIBObjectData`: the File's Owner as its root, the
//! windows visible at launch, the connections, each object's parent, and
//! each object's and connection's id. Every object is named and defined
//! where the encoder first meets it, as Xcode's are.

use crate::nib::{self, reference, Object, Value};
use crate::storyboard::{attr, child, children, fail, number, only_attributes, only_children, runtime_class_name};
use roxmltree::{Document, Node};
use std::collections::HashMap;

/// A nib being built: the strings and numbers it shares.
struct Nib {
    shared: HashMap<String, String>,
    next: usize,
    /// Whether the document is in `Base.lproj`, where its titles are
    /// localizable.
    localizable: bool,
    /// Whether the document is a storyboard.
    storyboard: bool,
    /// The views constraints refer to (`constrained`).
    constrained: std::collections::HashSet<String>,
    /// Whether the document is a xib without direct instantiation, whose
    /// custom views are `NSCustomView`s and custom objects
    /// `NSCustomObject`s (probes).
    old_custom: bool,
}

impl Nib {
    fn shared(&mut self, key: String, make: impl FnOnce() -> Object) -> Value {
        if let Some(name) = self.shared.get(&key) {
            return reference(name.clone());
        }
        self.next += 1;
        let name = format!("{}{}", nib::SHARED, self.next);
        self.shared.insert(key, name.clone());
        Value::New(make().named(name))
    }

    /// Removes `key` from `values`; a shared object defined there is built
    /// anew where next used, rather than referred to.
    fn remove(&mut self, values: &mut Vec<(&'static str, Value)>, key: &str) {
        for (_, v) in values.iter().filter(|v| v.0 == key) {
            if let Value::New(o) = v {
                if let Some(name) = &o.name {
                    self.shared.retain(|_, n| n != name);
                }
            }
        }
        values.retain(|v| v.0 != key);
    }

    fn string(&mut self, s: &str) -> Value {
        self.shared(format!("s:{s}"), || Object::new("NSString", vec![("NS.bytes", Value::Data(s.as_bytes().to_vec()))]))
    }

    /// A title or other text an object `id` shows: in `Base.lproj` an
    /// `NSLocalizableString` keyed `<id>.<key>`, its development text the
    /// shared string; empty, a plain one.
    fn text(&mut self, id: &str, key: &str, s: &str) -> Value {
        if !self.localizable || s.is_empty() {
            return self.string(s);
        }
        let key = self.string(&format!("{id}.{key}"));
        let dev = self.string(s);
        Value::New(Object::new(
            "NSLocalizableString",
            vec![("NS.bytes", Value::Data(s.as_bytes().to_vec())), ("NSKey", key), ("NSDev", dev)],
        ))
    }

    fn int_number(&mut self, i: i64) -> Value {
        self.shared(format!("n:{i}"), || Object::new("NSNumber", vec![("NS.intval", Value::Int(i))]))
    }

    fn empty_array(&mut self) -> Value {
        self.shared("empty array".into(), || Object::new("NSArray", vec![("NSInlinedValue", Value::Bool(true))]))
    }

    fn array(&mut self, class: &'static str, items: Vec<Value>) -> Value {
        let mut values = vec![("NSInlinedValue", Value::Bool(true))];
        values.extend(items.into_iter().map(|v| ("UINibEncoderEmptyKey", v)));
        Value::New(Object::new(class, values))
    }
}

/// The name an archived document object goes by: shared, so that it is
/// defined where the encoder first meets it.
fn object_name(id: &str) -> String {
    format!("{}o:{id}", nib::SHARED)
}

/// A number as NSStringFromRect and its kin write it.
fn g(x: f64) -> String {
    cf::xml::format_real(x)
}

#[derive(Clone, Copy)]
struct Frame {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn frame(n: Node, key: &str) -> Result<Frame, String> {
    let r = child(n, "rect", key).ok_or_else(|| format!("<{}> without a {key} rect", n.tag_name().name()))?;
    only_attributes(r, &[("key", None), ("x", None), ("y", None), ("width", None), ("height", None)])?;
    Ok(Frame {
        x: number(attr(r, "x")?)?,
        y: number(attr(r, "y")?)?,
        w: number(attr(r, "width")?)?,
        h: number(attr(r, "height")?)?,
    })
}

/// What compiling the document collects beside the objects themselves.
struct Collected {
    /// (object, parent) in the order the nib lists them.
    parents: Vec<(String, String)>,
    /// The windows visible at launch.
    visible: Vec<String>,
    /// The ids of the objects whose connections the nib holds.
    connection_sources: Vec<String>,
    /// The definitions of objects that the objects' list defines.
    definitions: HashMap<String, Value>,
    /// Views placed at their superview's origin whatever their frames say
    /// (a scroll view's document view).
    at_origin: Vec<String>,
    /// Views Xcode lays out whatever their frames say (a box's content
    /// view).
    laid_out: HashMap<String, Frame>,
    /// Views' accessibility identifiers: (view id, identifier).
    accessibility: Vec<(String, String)>,
}

/// An autoresizing mask's bits, as NSView's: flexible left margin 1, width
/// 2, right margin 4, bottom margin 8, height 16, top margin 32.
fn autoresizing_mask(n: Node) -> Result<i64, String> {
    // Without one, flexible right and bottom margins (a constraint probe's
    // subview).
    let Some(m) = child(n, "autoresizingMask", "autoresizingMask") else {
        return Ok(4 | 8);
    };
    only_attributes(
        m,
        &[
            ("key", None),
            ("flexibleMinX", Some("YES")),
            ("widthSizable", Some("YES")),
            ("flexibleMaxX", Some("YES")),
            ("flexibleMinY", Some("YES")),
            ("heightSizable", Some("YES")),
            ("flexibleMaxY", Some("YES")),
        ],
    )?;
    let mut mask = 0;
    for (name, bit) in [
        ("flexibleMinX", 1),
        ("widthSizable", 2),
        ("flexibleMaxX", 4),
        ("flexibleMinY", 8),
        ("heightSizable", 16),
        ("flexibleMaxY", 32),
    ] {
        if m.attribute(name).is_some() {
            mask |= bit;
        }
    }
    Ok(mask)
}

/// A view and its subviews: an `NSView`, or for a top-level custom view an
/// `NSClassSwapper` for its class.
fn view(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str, next_responder: Option<&str>) -> Result<Value, String> {
    // A custom view is an NSClassSwapper for its class, at any depth, as is
    // a plain view of a custom class; a plain view an NSView (constraint
    // probes, templates).
    let swapped = n.has_tag_name("customView") || n.attribute("customClass").is_some();
    // A container view is a plain view; its embed segue's template finds it
    // through its controller (four probes).
    let container = n.has_tag_name("containerView");
    // The context Interface Builder simulates changes Xcode's layout of the
    // text in it (rules_apple's HelloToday example, a probe).
    if n.attribute("simulatedAppContext").is_some() && !layout_relaxed() {
        return fail("a simulated app context is laid out by Xcode with its own metrics, which this ibtool doesn't reproduce; --config=relax_ibtool_layout compiles it with the document's frames");
    }
    if container && (n.attribute("customClass").is_some() || embed_segue(n)?.is_none()) {
        return fail("a container view with a class, or without one embed segue, is not supported by this ibtool");
    }
    only_attributes(
        n,
        &[
            ("key", None),
            ("id", None),
            ("simulatedAppContext", None),
            ("autoresizesSubviews", Some("NO")),
            ("hidden", Some("YES")),
            ("wantsLayer", Some("YES")),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
            ("translatesAutoresizingMaskIntoConstraints", Some("NO")),
        ],
    )?;
    only_children(n, &["rect", "autoresizingMask", "subviews", "constraints", "point", "connections", "accessibility"])?;
    canvas_location_only(n)?;
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), parent.to_string()));
    if children(n, "connections").next().is_some() && !container {
        c.connection_sources.push(attr(n, "id")?.to_string());
    }
    // Without direct instantiation, a custom view is an NSCustomView: the
    // view's keys, then its class (four probes).
    if nib.old_custom && swapped {
        if !n.has_tag_name("customView") {
            return fail("a view of a custom class in a xib without direct instantiation is not supported by this ibtool");
        }
        let class = runtime_class_name(n, module, "NSView")?;
        let mut values = view_values(nib, c, n, module, &name, parent, next_responder)?;
        // Its subviews in reverse (four probes; a plain view's keep their
        // order).
        for v in values.iter_mut().filter(|v| v.0 == "NSSubviews") {
            if let Value::New(o) = &mut v.1 {
                let items: Vec<usize> = (0..o.values.len()).filter(|&i| o.values[i].0 == "UINibEncoderEmptyKey").collect();
                let reversed: Vec<(&'static str, Value)> = items.iter().rev().map(|&i| std::mem::replace(&mut o.values[i], ("UINibEncoderEmptyKey", Value::Nil))).collect();
                for (slot, item) in items.into_iter().zip(reversed) {
                    o.values[slot] = item;
                }
            }
        }
        values.extend(class_values(nib, n, module, Some(&class))?);
        return Ok(Value::New(Object::new("NSCustomView", values).named(name)));
    }
    let mut values = Vec::new();
    let class = runtime_class_name(n, module, "NSView")?;
    if swapped {
        values.push(("NSClassName", nib.string(&class)));
        values.push(("NSOriginalClassName", nib.string("NSView")));
    }
    values.extend(view_values(nib, c, n, module, &name, parent, next_responder)?);
    Ok(Value::New(Object::new(if swapped { "NSClassSwapper" } else { "NSView" }, values).named(name)))
}

/// A control as its class: of another class than its AppKit one, an
/// `NSClassSwapper` naming both before its keys; of its own, itself (corpus
/// xibs' box and button).
fn swapped(nib: &mut Nib, n: Node, module: Option<&str>, class: &'static str, mut values: Vec<(&'static str, Value)>, name: String) -> Result<Value, String> {
    match n.attribute("customClass") {
        Some(c) if c != class => {
            let mut v = vec![("NSClassName", nib.string(&runtime_class_name(n, module, class)?)), ("NSOriginalClassName", nib.string(class))];
            v.append(&mut values);
            Ok(Value::New(Object::new("NSClassSwapper", v).named(name)))
        }
        _ => Ok(Value::New(Object::new(class, values).named(name))),
    }
}

/// A view's superview, its next responder: none at a xib's top level.
fn superview<'p>(n: Node, parent: &'p str) -> Option<&'p str> {
    if n.parent_element().is_some_and(|p| p.has_tag_name("objects")) {
        None
    } else {
        Some(parent)
    }
}

/// The keys every view archives, in order, from its next responder to its
/// clipping; its subviews and constraints join the objects' list.
fn view_values(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, name: &str, parent: &str, next_responder: Option<&str>) -> Result<Vec<(&'static str, Value)>, String> {
    if let Some(a) = children(n, "accessibility").next() {
        only_attributes(a, &[("identifier", None)])?;
        only_children(a, &[])?;
        c.accessibility.push((attr(n, "id")?.to_string(), attr(a, "identifier")?.to_string()));
    }
    let mut values = Vec::new();
    values.push(("NSNextResponder", next_responder.map_or(Value::Nil, |r| reference(r.to_string()))));
    values.push(("NSNibTouchBar", Value::Nil));
    // Hidden, the top bit (a probe's button, scrollers).
    let hidden = if n.attribute("hidden") == Some("YES") { 0x8000_0000_u32 as i32 as i64 } else { 0 };
    // 256: it resizes its subviews, unless it says not (a probe).
    let resizes = if n.attribute("autoresizesSubviews") == Some("NO") { 0 } else { 256 };
    values.push(("NSvFlags", Value::Int(hidden | resizes | autoresizing_mask(n)?)));
    // Its subviews, then its frame.
    let subviews: Vec<Node> = children(n, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())).collect();
    let content = n.has_tag_name("box").then(|| child(n, "view", "contentView")).flatten();
    if let Some(content) = content {
        let content = view(nib, c, content, module, name, Some(name))?;
        values.push(("NSSubviews", nib.array("NSMutableArray", vec![content])));
    } else if !subviews.is_empty() || ["button", "popUpButton", "box", "imageView"].iter().any(|t| n.has_tag_name(*t)) {
        let mut items = Vec::new();
        for s in subviews {
            items.push(match s.tag_name().name() {
                "view" | "customView" | "containerView" => view(nib, c, s, module, name, Some(name))?,
                "textField" => text_field(nib, c, s, module, name)?,
                "button" | "popUpButton" => button(nib, c, s, module, name)?,
                "scrollView" => scroll_view(nib, c, s, module, name)?,
                "box" => box_view(nib, c, s, module, name)?,
                "imageView" => image_view(nib, c, s, module, name)?,
                "wkWebView" => web_view(nib, c, s, module, name)?,
                "collectionView" => collection_view(nib, c, s, module, name)?,
                "slider" => slider(nib, c, s, module, name)?,
                other => return fail(format!("a subview <{other}> is not supported by this ibtool")),
            });
        }
        values.push(("NSSubviews", nib.array("NSMutableArray", items)));
    }
    let mut f = frame(n, "frame")?;
    if let Some(laid_out) = n.attribute("id").and_then(|id| c.laid_out.get(id)) {
        f = Frame { ..*laid_out };
    }
    if n.attribute("id").is_some_and(|id| c.at_origin.iter().any(|o| o == id)) {
        (f.x, f.y) = (0.0, 0.0);
    }
    if f.x == 0.0 && f.y == 0.0 {
        values.push(("NSFrameSize", nib.string(&format!("{{{}, {}}}", g(f.w), g(f.h)))));
    } else {
        values.push(("NSFrame", nib.string(&format!("{{{{{}, {}}}, {{{}, {}}}}}", g(f.x), g(f.y), g(f.w), g(f.h)))));
    }
    if next_responder.is_some() {
        values.push(("NSSuperview", reference(parent.to_string())));
    }
    if n.attribute("wantsLayer").is_some() {
        values.push(("NSViewIsLayerTreeHost", Value::Bool(true)));
    }
    values.push(("NSViewWantsBestResolutionOpenGLSurface", Value::Bool(true)));
    // A view controller's view leaves it out: the controller sizes it (a
    // probe, a corpus storyboard).
    let controller_view = n.attribute("key") == Some("view") && n.parent_element().is_some_and(|p| p.has_tag_name("viewController"));
    if n.attribute("translatesAutoresizingMaskIntoConstraints").is_some() && n.attribute("id").is_some_and(|id| nib.constrained.contains(id)) && !controller_view {
        values.push(("NSDoNotTranslateAutoresizingMask", Value::Bool(true)));
    }
    // Its constraints follow its subviews' in the objects' list.
    if let Some(constraints) = view_constraints(nib, c, n, name)? {
        values.push(("NSViewConstraints", constraints));
    }
    // Content hugging and compression resistance, as strings, when other
    // than the class's own: hugging 250 each, a text field's vertical 750;
    // resistance 750 each (two templates' labels, a probe's).
    let text = n.has_tag_name("textField");
    // A priority the document leaves out is Interface Builder's default
    // (250 hugging, 750 resistance), whatever the class's own (a probe's
    // text field without them).
    let pair = |h: &str, v: &str, absent: &str, (dh, dv): (&str, &str)| -> Option<String> {
        let (h, v) = (n.attribute(h).unwrap_or(absent), n.attribute(v).unwrap_or(absent));
        ((h, v) != (dh, dv)).then(|| format!("{{{h}, {v}}}"))
    };
    if let Some(p) = pair("horizontalHuggingPriority", "verticalHuggingPriority", "250", ("250", if text { "750" } else { "250" })) {
        values.push(("NSHuggingPriority", nib.string(&p)));
    }
    if let Some(p) = pair("horizontalCompressionResistancePriority", "verticalCompressionResistancePriority", "750", ("750", "750")) {
        values.push(("NSAntiCompressionPriority", nib.string(&p)));
    }
    // A storyboard's box content view clips to its bounds (probes).
    if nib.storyboard && n.attribute("key") == Some("contentView") && n.parent_element().is_some_and(|p| p.has_tag_name("box")) {
        values.push(("NSViewClipsToBoundsKeyName", Value::Bool(true)));
    }
    values.push(("IBNSSafeAreaLayoutGuide", Value::Nil));
    values.push(("IBNSLayoutMarginsGuide", Value::Nil));
    values.push(("IBNSClipsToBounds", Value::Int(0)));
    Ok(values)
}

/// The order Xcode lists a view's constraints in (about 50 probes, a
/// corpus storyboard): first the view's own, relating no subview; then each
/// subview's, those whose first item it is or whose second when the view is
/// the first, the subviews nearest the view's top-left corner first (by the
/// distance to the top-left corner of their frames). Within each: those
/// relating the subview to the view, and its size, by the view's attribute
/// (the subview's for a size: centerX, centerY, right, left, trailing,
/// leading, width, bottom, top, height); then those relating it to other
/// subviews, by its own attribute (centerX, trailing, leading, centerY,
/// bottom, top, width, height; a centre or size to the other's same), then
/// the other subview's attribute, then the other subview farthest from it
/// first (by the distance between their centres: four ties in two corpus
/// storyboards, which their corners don't order). Subviews at the same
/// distance, two different constraints on the same attributes of the same
/// views, other attributes (between subviews: left and right, a centre or
/// size to another attribute) and constraints on other views aren't
/// established.
fn constraint_order(c: &Collected, n: Node, list: &[Node]) -> Result<Vec<usize>, String> {
    let rank = |a: &str| -> Result<usize, String> {
        [
            "centerX",
            "centerY",
            "right",
            "left",
            "trailing",
            "leading",
            "width",
            "bottom",
            "top",
            "firstBaseline",
            "baseline",
            "height",
        ]
        .iter()
        .position(|r| *r == a)
        .ok_or_else(|| format!("the order of a view's constraints on {a} is not established by this ibtool"))
    };
    let own = n.attribute("id");
    let subviews: Vec<Node> = children(n, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())).collect();
    let height = match own.and_then(|id| c.laid_out.get(id)) {
        Some(f) => f.h,
        None => frame(n, "frame")?.h,
    };
    // Each subview's squared distance from the view's top-left corner to
    // its own, and its centre.
    let mut distances = Vec::new();
    for s in &subviews {
        let f = frame(*s, "frame")?;
        let (dx, dy) = (f.x, height - (f.y + f.h));
        distances.push((attr(*s, "id")?, dx * dx + dy * dy, (f.x + f.w / 2.0, f.y + f.h / 2.0), f.x));
    }
    let apart = |a: &str, b: &str| -> f64 {
        let centre = |id: &str| distances.iter().find(|d| d.0 == id).map_or((0.0, 0.0), |d| d.2);
        let ((ax, ay), (bx, by)) = (centre(a), centre(b));
        (ax - bx) * (ax - bx) + (ay - by) * (ay - by)
    };
    // Each constraint's (group distance, sibling relation, attribute rank,
    // the other subview's attribute rank, minus its distance), the view's
    // own at -1.
    let mut keys: Vec<(f64, bool, usize, usize, f64)> = Vec::new();
    let mut groups: Vec<&str> = Vec::new();
    let mut group_xs: Vec<f64> = Vec::new();
    let mut group_ids: Vec<&str> = Vec::new();
    for k in list {
        let first = k.attribute("firstItem").filter(|f| Some(*f) != own);
        let second = k.attribute("secondItem").filter(|s| Some(*s) != own);
        let first_attribute = rank(attr(*k, "firstAttribute")?)?;
        let second_attribute = || rank(k.attribute("secondAttribute").ok_or("a constraint with a second item and no attribute")?);
        let subview = |id: &str| -> Result<f64, String> {
            distances
                .iter()
                .find(|d| d.0 == id)
                .map(|d| d.1)
                .ok_or_else(|| "a constraint on a view other than its view or a subview is not supported by this ibtool".to_string())
        };
        if let Some(g) = first.or(second) {
            if !groups.contains(&g) {
                groups.push(g);
            }
        }
        group_xs.push(first.or(second).and_then(|g| distances.iter().find(|d| d.0 == g)).map_or(f64::NEG_INFINITY, |d| d.3));
        group_ids.push(first.or(second).unwrap_or(""));
        let key = match (first, second) {
            (None, None) => (-1.0, false, first_attribute, 0, 0.0),
            (Some(f), None) if k.attribute("secondItem").is_none() => (subview(f)?, false, first_attribute, 0, 0.0),
            // A baseline ranks by the subview's baseline, whatever the
            // view's attribute (five probes).
            (Some(f), None) if is_baseline(attr(*k, "firstAttribute")?) => (subview(f)?, false, first_attribute, 0, 0.0),
            (None, Some(s)) if k.attribute("secondAttribute").is_some_and(is_baseline) => (subview(s)?, false, second_attribute()?, 0, 0.0),
            (Some(f), None) => (subview(f)?, false, second_attribute()?, 0, 0.0),
            (None, Some(s)) => (subview(s)?, false, first_attribute, 0, 0.0),
            (Some(f), Some(s)) if f == s => return fail("a view's constraint relating a subview to itself is not supported by this ibtool"),
            (Some(f), Some(s)) => {
                subview(s)?;
                let names = (attr(*k, "firstAttribute")?, k.attribute("secondAttribute").unwrap_or_default());
                let sibling = |a: &str| {
                    ["centerX", "trailing", "leading", "centerY", "bottom", "top", "firstBaseline", "baseline", "width", "height"]
                        .iter()
                        .position(|r| *r == a)
                };
                let size = |a: &str| matches!(a, "width" | "height" | "centerX" | "centerY");
                if is_baseline(names.0) != is_baseline(names.1) {
                    return fail(format!("the order of constraints between subviews on {} and {} is not established by this ibtool", names.0, names.1));
                }
                let (Some(first_rank), Some(second_rank)) = (sibling(names.0), sibling(names.1)) else {
                    return fail(format!("the order of constraints between subviews on {} and {} is not established by this ibtool", names.0, names.1));
                };
                if (size(names.0) || size(names.1)) && names.0 != names.1 {
                    return fail(format!("the order of constraints between subviews on {} and {} is not established by this ibtool", names.0, names.1));
                }
                (subview(f)?, true, first_rank, second_rank, -apart(f, s))
            }
        };
        // Identical constraints archive the same in either order.
        let same = |o: &Node| {
            o.attributes()
                .filter(|a| a.name() != "id")
                .map(|a| (a.name(), a.value()))
                .eq(k.attributes().filter(|a| a.name() != "id").map(|a| (a.name(), a.value())))
        };
        let gid = *group_ids.last().expect("pushed");
        if keys.iter().zip(list).zip(&group_ids).any(|((other, o), oid)| *other == key && *oid == gid && !same(o)) {
            return fail("a view's constraints whose order isn't established (on the same attributes of the same views, or of subviews as far apart) are not supported by this ibtool");
        }
        keys.push(key);
    }
    // Subviews as far from the corner go leftmost first (four probes), and
    // at the same corner by their ids as bytes (four probes).
    let mut order: Vec<usize> = (0..list.len()).collect();
    order.sort_by(|&a, &b| {
        let (ka, kb) = (keys[a], keys[b]);
        ka.0.total_cmp(&kb.0)
            .then(group_xs[a].total_cmp(&group_xs[b]))
            .then(group_ids[a].cmp(group_ids[b]))
            .then(ka.1.cmp(&kb.1))
            .then(ka.2.cmp(&kb.2))
            .then(ka.3.cmp(&kb.3))
            .then(ka.4.total_cmp(&kb.4))
    });
    Ok(order)
}

/// A constraint's multiplier: a number, or a ratio `a:b`.
fn multiplier(m: &str) -> Result<f64, String> {
    match m.split_once(':') {
        Some((a, b)) => {
            let (a, b) = (number(a)?, number(b)?);
            if b == 0.0 {
                return fail(format!("a multiplier {m} dividing by zero"));
            }
            Ok(a / b)
        }
        None => number(m),
    }
}

/// Whether `relax_ibtool_layout` compiles constrained views with their
/// documents' frames.
fn layout_relaxed() -> bool {
    std::env::var("RULES_APPLECROSS_RELAX_IBTOOL_LAYOUT").is_ok_and(|v| v == "1") || std::env::var("RULES_APPLECROSS_RESOURCE_OUTPUT_MODE").is_ok_and(|v| v == "relaxed")
}

/// Refuses a view's constraints that its subviews' frames in the document
/// don't satisfy: Xcode lays such views out again as it compiles (probes
/// whose frames disagreed moved), while this ibtool keeps the document's
/// frames. Attributes are measured as Auto Layout does, left to right and
/// down from the view's top (a probe whose frames agree, kept): leading and
/// left the frame's left edge, trailing and right its right, top and bottom
/// from the view's top edge.
fn frames_agree(c: &Collected, n: Node, list: &[Node]) -> Result<(), String> {
    let own = n.attribute("id");
    let bounds = match own.and_then(|id| c.laid_out.get(id)) {
        Some(f) => *f,
        None => frame(n, "frame")?,
    };
    let height = bounds.h;
    let rect = |id: Option<&str>| -> Result<Frame, String> {
        match id.filter(|i| Some(*i) != own) {
            None => Ok(Frame {
                x: 0.0,
                y: 0.0,
                w: bounds.w,
                h: bounds.h,
            }),
            Some(id) => {
                let s = children(n, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())).find(|s| s.attribute("id") == Some(id));
                frame(s.ok_or("a constraint on a view other than its view or a subview is not supported by this ibtool")?, "frame")
            }
        }
    };
    let value = |f: &Frame, a: &str| -> Result<f64, String> {
        Ok(match a {
            "leading" | "left" => f.x,
            "trailing" | "right" => f.x + f.w,
            "centerX" => f.x + f.w / 2.0,
            "width" => f.w,
            "top" => height - (f.y + f.h),
            "bottom" => height - f.y,
            "centerY" => height - (f.y + f.h / 2.0),
            "height" => f.h,
            other => return fail(format!("a constraint on {other} is not supported by this ibtool")),
        })
    };
    for k in list {
        let first = value(&rect(k.attribute("firstItem"))?, attr(*k, "firstAttribute")?)?;
        let second = match k.attribute("secondItem") {
            Some(s) => value(&rect(Some(s))?, k.attribute("secondAttribute").ok_or("a constraint with a second item and no attribute")?)?,
            // A size alone, or a second attribute of the view's own.
            None => match k.attribute("secondAttribute") {
                Some(a) => value(&rect(None)?, a)?,
                None => 0.0,
            },
        };
        let factor = k.attribute("multiplier").map(multiplier).transpose()?.unwrap_or(1.0);
        let wanted = factor * second + k.attribute("constant").map(number).transpose()?.unwrap_or(0.0);
        let agrees = match k.attribute("relation") {
            None => (first - wanted).abs() < 0.001,
            Some("greaterThanOrEqual") => first > wanted - 0.001,
            Some("lessThanOrEqual") => first < wanted + 0.001,
            Some(other) => return fail(format!("constraint relation {other} is not supported by this ibtool")),
        };
        if !agrees {
            return fail(format!(
                "a constraint ({}) that its views' frames in the document don't satisfy, so Xcode lays them out again as it compiles, which this ibtool doesn't reproduce; --config=relax_ibtool_layout compiles them with the document's frames",
                attr(*k, "id")?
            ));
        }
    }
    Ok(())
}

fn is_baseline(a: &str) -> bool {
    matches!(a, "firstBaseline" | "baseline")
}

/// The constraints a view holds, as `NSLayoutConstraint`s, in
/// [`constraint_order`]: a missing first item is the view; attributes by
/// NSLayoutAttribute, relations by NSLayoutRelation (-1, 0, 1); the
/// constant twice; the priority when not required.
fn view_constraints(nib: &mut Nib, c: &mut Collected, n: Node, name: &str) -> Result<Option<Value>, String> {
    let list: Vec<Node> = children(n, "constraints").flat_map(|l| l.children().filter(|c| c.is_element())).collect();
    if list.is_empty() {
        return Ok(None);
    }
    let ids: Vec<&str> = list.iter().map(|k| attr(*k, "id")).collect::<Result<_, _>>()?;
    let order = constraint_order(c, n, &list)?;
    if !layout_relaxed() {
        frames_agree(c, n, &list)?;
    }
    let mut items = Vec::new();
    for i in order {
        let k = list[i];
        if !k.has_tag_name("constraint") {
            return fail(format!("<{}> in a view's constraints is not supported by this ibtool", k.tag_name().name()));
        }
        only_attributes(
            k,
            &[
                ("firstItem", None),
                ("firstAttribute", None),
                ("relation", None),
                ("secondItem", None),
                ("secondAttribute", None),
                ("constant", None),
                ("symbolic", Some("YES")),
                ("multiplier", None),
                ("priority", None),
                ("id", None),
            ],
        )?;
        let attribute = |a: &str| -> Result<i64, String> {
            Ok(match a {
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
                "baseline" | "lastBaseline" => 11,
                "firstBaseline" => 12,
                other => return fail(format!("constraint attribute {other} is not supported by this ibtool")),
            })
        };
        let constraint_name = object_name(ids[i]);
        c.parents.push((constraint_name.clone(), name.to_string()));
        let first = k.attribute("firstItem").map_or(name.to_string(), object_name);
        let first_attribute = attribute(attr(k, "firstAttribute")?)?;
        let mut values = vec![
            ("NSFirstItem", reference(first)),
            ("NSFirstAttributeV2", Value::Int(first_attribute)),
            ("NSFirstAttribute", Value::Int(first_attribute)),
        ];
        match k.attribute("relation") {
            None => {}
            Some("greaterThanOrEqual") => values.push(("NSRelation", Value::Int(1))),
            Some("lessThanOrEqual") => values.push(("NSRelation", Value::Int(-1))),
            Some(other) => return fail(format!("constraint relation {other} is not supported by this ibtool")),
        }
        // A multiplier, rounded to a float (four probes; a ratio as its
        // quotient).
        if let Some(m) = k.attribute("multiplier") {
            values.push(("NSMultiplier", Value::Double(multiplier(m)? as f32 as f64)));
        }
        if let Some(second) = k.attribute("secondItem") {
            let a = attribute(attr(k, "secondAttribute")?)?;
            values.push(("NSSecondItem", reference(object_name(second))));
            values.push(("NSSecondAttributeV2", Value::Int(a)));
            values.push(("NSSecondAttribute", Value::Int(a)));
        } else if k.attribute("secondAttribute").is_some() {
            return fail("a constraint's second attribute without a second item");
        }
        // Without a constant, neither constant key (a probe); a symbolic
        // one, the standard space, in their place (a probe).
        let constant = k.attribute("constant").map(number).transpose()?.filter(|_| k.attribute("symbolic").is_none());
        if let Some(constant) = constant {
            values.push(("NSConstantV2", Value::Double(constant)));
        }
        if k.attribute("symbolic").is_some() {
            values.push(("NSSymbolicConstant", nib.string("NSSpace")));
        }
        values.push(("NSShouldBeArchived", Value::Bool(true)));
        if let Some(p) = k.attribute("priority") {
            values.push(("NSPriority", Value::Int(p.parse().map_err(|_| format!("not a priority: {p}"))?)));
        }
        if let Some(constant) = constant {
            values.push(("NSConstant", Value::Double(constant)));
        }
        items.push(Value::New(Object::new("NSLayoutConstraint", values).named(constraint_name)));
    }
    Ok(Some(nib.array("NSArray", items)))
}

/// A window's position flags in `NSWTFlags` (bits 19 to 22), from its
/// initial position's struts: without one on an axis, or with both, both
/// of the axis's bits; with one, the other's (a left strut keeps 0x80000, a
/// right one 0x100000, a top one 0x200000, a bottom one 0x400000); with a
/// left and a bottom one alone, none. Springs change nothing (64 probes).
fn position_flags(n: Node) -> Result<i64, String> {
    let Some(m) = child(n, "windowPositionMask", "initialPositionMask") else {
        return Ok(0x78_0000);
    };
    only_attributes(
        m,
        &[
            ("key", None),
            ("leftStrut", Some("YES")),
            ("rightStrut", Some("YES")),
            ("topStrut", Some("YES")),
            ("bottomStrut", Some("YES")),
            ("horizontalStrut", Some("YES")),
            ("verticalStrut", Some("YES")),
        ],
    )?;
    let has = |s: &str| m.attribute(s).is_some();
    let x = match (has("leftStrut"), has("rightStrut")) {
        (true, false) => 0x8_0000,
        (false, true) => 0x10_0000,
        _ => 0x18_0000,
    };
    let y = match (has("topStrut"), has("bottomStrut")) {
        (true, false) => 0x20_0000,
        (false, true) => 0x40_0000,
        _ => 0x60_0000,
    };
    Ok(if (x, y) == (0x8_0000, 0x40_0000) { 0 } else { x | y })
}

/// A window, as an `NSWindowTemplate`.
fn window(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("title", None),
            ("allowsToolTipsWhenApplicationIsInactive", Some("NO")),
            ("autorecalculatesKeyViewLoop", Some("NO")),
            ("releasedWhenClosed", Some("NO")),
            ("deferred", Some("NO")),
            ("frameAutosaveName", None),
            ("animationBehavior", Some("default")),
            ("visibleAtLaunch", Some("NO")),
            // Not one-shot, or without a toolbar button, changes nothing (a
            // probe of each).
            ("oneShot", Some("NO")),
            ("showsToolbarButton", Some("NO")),
            ("restorable", Some("NO")),
            ("titlebarAppearsTransparent", Some("YES")),
            ("titleVisibility", Some("hidden")),
            ("key", Some("window")),
            ("id", None),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
        ],
    )?;
    only_children(
        n,
        &["windowStyleMask", "windowPositionMask", "windowCollectionBehavior", "rect", "value", "view", "point", "connections"],
    )?;
    for p in children(n, "point") {
        if p.attribute("key") != Some("canvasLocation") {
            return fail("a window's point other than its canvas location is not supported by this ibtool");
        }
    }
    // Its minimum and maximum sizes: its content's as given, and its
    // frame's with the title bar's 32 points when titled (seven probes).
    let mut sizes = [None, None];
    for v in children(n, "value") {
        only_attributes(v, &[("key", None), ("type", Some("size")), ("width", None), ("height", None)])?;
        let at = match attr(v, "key")? {
            "minSize" => 0,
            "maxSize" => 1,
            other => return fail(format!("a window's {other} is not supported by this ibtool")),
        };
        sizes[at] = Some((number(attr(v, "width")?)?, number(attr(v, "height")?)?));
    }
    let id = attr(n, "id")?;
    let name = object_name(id);
    c.parents.push((name.clone(), parent.to_string()));
    // A storyboard's windows are its window controllers', which show them,
    // and are never visible at launch (Xcode's game templates).
    if n.attribute("visibleAtLaunch").is_none() && !nib.storyboard {
        c.visible.push(name.clone());
    }
    let style = child(n, "windowStyleMask", "styleMask").ok_or("a window without a style mask")?;
    only_attributes(
        style,
        &[
            ("key", None),
            ("titled", Some("YES")),
            ("closable", Some("YES")),
            ("miniaturizable", Some("YES")),
            ("resizable", Some("YES")),
        ],
    )?;
    let mut mask = 0;
    for (a, bit) in [("titled", 1), ("closable", 2), ("miniaturizable", 4), ("resizable", 8)] {
        if style.attribute(a).is_some() {
            mask |= bit;
        }
    }
    // Flags: deferred (0x20000000, the default), not released when closed
    // (0x40000000), recalculating the key view loop (0x800) and tooltips
    // while inactive (0x2000), both defaults, and the position's (28
    // probes).
    let mut flags = position_flags(n)?;
    if n.attribute("deferred").is_none() {
        flags |= 0x2000_0000;
    }
    if n.attribute("releasedWhenClosed").is_some() {
        flags |= 0x4000_0000;
    }
    if n.attribute("autorecalculatesKeyViewLoop").is_none() {
        flags |= 0x800;
    }
    if n.attribute("allowsToolTipsWhenApplicationIsInactive").is_none() {
        flags |= 0x2000;
    }
    // Its content rectangle, centred on the screen along an axis without a
    // strut (16 probes).
    let content = frame(n, "contentRect")?;
    let screen = frame(n, "screenRect")?;
    let struts = child(n, "windowPositionMask", "initialPositionMask");
    let has = |s: &str| struts.is_some_and(|m| m.attribute(s).is_some());
    let x = if has("leftStrut") || has("rightStrut") {
        content.x
    } else {
        screen.x + (screen.w - content.w) / 2.0
    };
    let y = if has("topStrut") || has("bottomStrut") {
        content.y
    } else {
        screen.y + (screen.h - content.h) / 2.0
    };
    let contents = children(n, "view").collect::<Vec<_>>();
    let content_view = match contents.as_slice() {
        [v] if v.attribute("key") == Some("contentView") => Some(*v),
        // A storyboard's window without one has a plain view of its
        // content's size (Xcode's app template).
        [] if nib.storyboard => None,
        _ => return fail("a window's views other than one content view are not supported by this ibtool"),
    };
    let titlebar = if mask & 1 != 0 { 32.0 } else { 0.0 };
    let size = |nib: &mut Nib, (w, h): (f64, f64), extra: f64| nib.string(&format!("{{{}, {}}}", g(w), g(h + extra)));
    let mut values = vec![
        ("NSWindowStyleMask", Value::Int(mask)),
        ("NSWindowBacking", Value::Int(2)),
        ("NSWindowRect", nib.string(&format!("{{{{{}, {}}}, {{{}, {}}}}}", g(x), g(y), g(content.w), g(content.h)))),
        ("NSWTFlags", Value::Int(flags)),
        ("NSWindowTitle", nib.text(id, "title", n.attribute("title").unwrap_or(""))),
        ("NSWindowSubtitle", nib.string("")),
        ("NSWindowClass", nib.string(&runtime_class_name(n, module, "NSWindow")?)),
        ("NSViewClass", Value::Nil),
        ("NSUserInterfaceItemIdentifier", Value::Nil),
    ];
    if let Some(max) = sizes[1] {
        values.push(("NSWindowContentMaxSize", size(nib, max, 0.0)));
    }
    if let Some(min) = sizes[0] {
        values.push(("NSWindowContentMinSize", size(nib, min, 0.0)));
    }
    let content = match content_view {
        Some(v) => view(nib, c, v, module, &name, None)?,
        None => {
            let view_name = format!("{name} content view");
            c.parents.push((view_name.clone(), name.clone()));
            let values = vec![
                ("NSNextResponder", Value::Nil),
                ("NSNibTouchBar", Value::Nil),
                ("NSvFlags", Value::Int(256)),
                ("NSFrameSize", size(nib, (content.w, content.h), 0.0)),
                ("NSViewWantsBestResolutionOpenGLSurface", Value::Bool(true)),
                ("IBNSSafeAreaLayoutGuide", Value::Nil),
                ("IBNSLayoutMarginsGuide", Value::Nil),
                ("IBNSClipsToBounds", Value::Int(0)),
            ];
            Value::New(Object::new("NSView", values).named(view_name))
        }
    };
    values.push(("NSWindowView", content));
    values.push(("NSScreenRect", nib.string(&format!("{{{{{}, {}}}, {{{}, {}}}}}", g(screen.x), g(screen.y), g(screen.w), g(screen.h)))));
    if let Some(min) = sizes[0] {
        values.push(("NSMinSize", size(nib, min, titlebar)));
    }
    values.push((
        "NSMaxSize",
        match sizes[1] {
            Some(max) => size(nib, max, titlebar),
            None => nib.string("{10000000000000, 10000000000000}"),
        },
    ));
    if let Some(name) = n.attribute("frameAutosaveName") {
        values.push(("NSFrameAutosaveName", nib.string(name)));
    }
    // Its collection behaviour, NSWindowCollectionBehavior's bits (13
    // probes; Interface Builder ignores the others).
    if let Some(behavior) = child(n, "windowCollectionBehavior", "collectionBehavior") {
        let mut bits = 0;
        for a in behavior.attributes().filter(|a| a.name() != "key") {
            if a.value() != "YES" {
                return fail(format!("a window collection behaviour {}=\"{}\" is not supported by this ibtool", a.name(), a.value()));
            }
            bits |= match a.name() {
                "canJoinAllSpaces" => 1,
                "moveToActiveSpace" => 2,
                "managed" => 4,
                "transient" => 8,
                "stationary" => 16,
                "participatesInCycle" => 32,
                "ignoresCycle" => 64,
                "fullScreenPrimary" => 128,
                "fullScreenAuxiliary" => 256,
                "fullScreenNone" => 512,
                "fullScreenAllowsTiling" => 2048,
                "fullScreenDisallowsTiling" => 4096,
                other => return fail(format!("the window collection behaviour {other} is not supported by this ibtool")),
            };
        }
        values.push(("NSWindowCollectionBehavior", Value::Int(bits)));
    }
    // Restorable unless it says not (a probe).
    values.push(("NSWindowIsRestorable", Value::Bool(n.attribute("restorable").is_none())));
    // A xib's window has full-screen content sizes; a storyboard's doesn't
    // (Xcode's app template).
    if !nib.storyboard {
        values.push(("NSMinFullScreenContentSize", nib.string("{0, 0}")));
        values.push(("NSMaxFullScreenContentSize", nib.string("{0, 0}")));
    }
    // A hidden title and a transparent title bar, after the full-screen
    // sizes (two probes).
    if n.attribute("titleVisibility").is_some() {
        values.push(("NSWindowTitleVisibility", Value::Int(1)));
    }
    if n.attribute("titlebarAppearsTransparent").is_some() {
        values.push(("NSTitlebarAppearsTransparent", Value::Bool(true)));
    }
    // A custom class's reference last (three corpus xibs).
    if n.attribute("customClass").is_some() {
        let class = class_values(nib, n, module, None)?
            .into_iter()
            .find(|v| v.0 == "IBClassReference")
            .map(|v| v.1)
            .ok_or("a custom class's reference")?;
        values.push(("IBClassReference", class));
    }
    Ok(Value::New(Object::new("NSWindowTemplate", values).named(name)))
}

/// A menu: its title, items and, for a system menu, the name AppKit finds
/// it by. Its items, and their submenus, follow it in the objects' list.
fn menu(nib: &mut Nib, c: &mut Collected, n: Node, parent: &str) -> Result<Value, String> {
    only_attributes(n, &[("key", None), ("title", None), ("systemMenu", None), ("id", None)])?;
    only_children(n, &["items", "point"])?;
    canvas_location_only(n)?;
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), parent.to_string()));
    let mut items = Vec::new();
    for list in children(n, "items") {
        only_children(list, &["menuItem"])?;
        for i in children(list, "menuItem") {
            items.push(menu_item(nib, c, i, &name, None)?);
        }
    }
    let title = nib.text(attr(n, "id")?, "title", n.attribute("title").unwrap_or(""));
    let mut values = vec![("NSTitle", title), ("NSMenuItems", nib.array("NSMutableArray", items))];
    if let Some(system) = n.attribute("systemMenu") {
        let system = match system {
            "main" => "_NSMainMenu",
            "apple" => "_NSAppleMenu",
            "services" => "_NSServicesMenu",
            "recentDocuments" => "_NSRecentDocumentsMenu",
            "font" => "_NSFontMenu",
            "window" => "_NSWindowsMenu",
            "help" => "_NSHelpMenu",
            other => return fail(format!("system menu {other} is not supported by this ibtool")),
        };
        values.push(("NSName", nib.string(system)));
    }
    Ok(Value::New(Object::new("NSMenu", values).named(name)))
}

/// A pop-up button's menu, and its selected item and that item's index: the
/// cell holds the item, defined there, and the menu refers to it (five
/// probes).
fn popup_menu(nib: &mut Nib, c: &mut Collected, n: Node, cell_name: &str, selected: Option<&str>) -> Result<(Value, Option<(Value, usize)>), String> {
    only_attributes(n, &[("key", None), ("id", None)])?;
    only_children(n, &["items"])?;
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), cell_name.to_string()));
    let mut items = Vec::new();
    let mut chosen = None;
    for list in children(n, "items") {
        only_children(list, &["menuItem"])?;
        for i in children(list, "menuItem") {
            if Some(attr(i, "id")?) == selected {
                chosen = Some(items.len());
            }
            items.push(menu_item(nib, c, i, &name, Some(cell_name))?);
        }
    }
    let chosen = match (selected, chosen) {
        (Some(id), Some(at)) => Some((std::mem::replace(&mut items[at], reference(object_name(id))), at)),
        (Some(_), None) => return fail("a pop-up button's selected item outside its menu"),
        (None, _) => None,
    };
    let title = nib.string("");
    let menu = Value::New(Object::new("NSMenu", vec![("NSTitle", title), ("NSMenuItems", nib.array("NSMutableArray", items))]).named(name));
    Ok((menu, chosen))
}

/// A title: its attribute, or a `<string key="title">` (one with leading
/// white space, as Xcode's main menu template's writing directions).
fn title(n: Node) -> Result<String, String> {
    let element = children(n, "string").find(|s| s.attribute("key") == Some("title"));
    match (n.attribute("title"), element) {
        (Some(t), None) => Ok(t.to_string()),
        (None, Some(e)) => {
            only_attributes(e, &[("key", None)])?;
            Ok(e.text().unwrap_or("").to_string())
        }
        (None, None) => Ok(String::new()),
        (Some(_), Some(_)) => fail("a title both as an attribute and an element"),
    }
}

/// Refuses points but a canvas location, which is the editor's alone.
fn canvas_location_only(n: Node) -> Result<(), String> {
    for p in children(n, "point") {
        if p.attribute("key") != Some("canvasLocation") {
            return fail(format!("a point other than a canvas location in <{}> is not supported by this ibtool", n.tag_name().name()));
        }
    }
    Ok(())
}

/// An image AppKit provides, as a nib names it: `NSCustomResource`.
fn system_image(nib: &mut Nib, name: &str, size: &str) -> Value {
    let key = format!("resource:{name}");
    let class = nib.string("NSImage");
    let resource = nib.string(name);
    let size = nib.string(size);
    let size = Value::New(Object::new("NSValue", vec![("NS.special", Value::Int(2)), ("NS.sizeval", size)]));
    nib.shared(key, || {
        Object::new(
            "NSCustomResource",
            vec![
                ("NSClassName", class),
                ("NSResourceName", resource),
                ("IBNamespaceID", Value::Nil),
                ("IBDesignSize", size),
                ("IBDesignImageConfiguration", Value::Nil),
            ],
        )
    })
}

/// AppKit's named images, which Interface Builder names in the `system`
/// namespace, and the design sizes Xcode gives them from the Mac's images
/// (macOS 27), whatever the document's resources say: NSImage.h's
/// `NSImageName` constants as their values ("NS" and the constant's suffix),
/// but for the retired `NSDotMac` (a probe of each).
const SYSTEM_IMAGES: [(&str, &str); 138] = [
    ("NSActionTemplate", "{19, 19}"),
    ("NSAddTemplate", "{18, 16}"),
    ("NSAdvanced", "{32, 32}"),
    ("NSApplicationIcon", "{32, 32}"),
    ("NSBluetoothTemplate", "{16, 20}"),
    ("NSBonjour", "{32, 32}"),
    ("NSBookmarksTemplate", "{22, 17}"),
    ("NSCaution", "{32, 32}"),
    ("NSColorPanel", "{32, 32}"),
    ("NSColumnViewTemplate", "{24, 17}"),
    ("NSComputer", "{32, 32}"),
    ("NSEnterFullScreenTemplate", "{20, 18}"),
    ("NSEveryone", "{32, 32}"),
    ("NSExitFullScreenTemplate", "{20, 18}"),
    ("NSFlowViewTemplate", "{18, 10}"),
    ("NSFolder", "{32, 32}"),
    ("NSFolderBurnable", "{32, 32}"),
    ("NSFolderSmart", "{32, 32}"),
    ("NSFollowLinkFreestandingTemplate", "{19, 19}"),
    ("NSFontPanel", "{32, 32}"),
    ("NSGoBackTemplate", "{12, 16}"),
    ("NSGoForwardTemplate", "{12, 16}"),
    ("NSGoLeftTemplate", "{12, 16}"),
    ("NSGoRightTemplate", "{12, 16}"),
    ("NSHomeTemplate", "{24, 20}"),
    ("NSIChatTheaterTemplate", "{15, 13}"),
    ("NSIconViewTemplate", "{19, 17}"),
    ("NSInfo", "{32, 32}"),
    ("NSInvalidDataFreestandingTemplate", "{19, 19}"),
    ("NSLeftFacingTriangleTemplate", "{12, 16}"),
    ("NSListViewTemplate", "{21, 14}"),
    ("NSLockLockedTemplate", "{16, 18}"),
    ("NSLockUnlockedTemplate", "{21, 18}"),
    ("NSMenuMixedStateTemplate", "{18, 4}"),
    ("NSMenuOnStateTemplate", "{18, 16}"),
    ("NSMobileMe", "{32, 32}"),
    ("NSMultipleDocuments", "{32, 32}"),
    ("NSNetwork", "{32, 32}"),
    ("NSPathTemplate", "{25, 14}"),
    ("NSPreferencesGeneral", "{32, 32}"),
    ("NSQuickLookTemplate", "{27, 16}"),
    ("NSRefreshFreestandingTemplate", "{19, 19}"),
    ("NSRefreshTemplate", "{17, 20}"),
    ("NSRemoveTemplate", "{18, 4}"),
    ("NSRevealFreestandingTemplate", "{19, 19}"),
    ("NSRightFacingTriangleTemplate", "{12, 16}"),
    ("NSShareTemplate", "{19, 22}"),
    ("NSSlideshowTemplate", "{24, 17}"),
    ("NSSmartBadgeTemplate", "{14, 14}"),
    ("NSStatusAvailable", "{16, 16}"),
    ("NSStatusNone", "{16, 16}"),
    ("NSStatusPartiallyAvailable", "{16, 16}"),
    ("NSStatusUnavailable", "{16, 16}"),
    ("NSStopProgressFreestandingTemplate", "{19, 19}"),
    ("NSStopProgressTemplate", "{17, 15}"),
    ("NSTouchBarAddDetailTemplate", "{19, 19}"),
    ("NSTouchBarAddTemplate", "{18, 16}"),
    ("NSTouchBarAlarmTemplate", "{18, 19}"),
    ("NSTouchBarAudioInputMuteTemplate", "{17, 20}"),
    ("NSTouchBarAudioInputTemplate", "{16, 20}"),
    ("NSTouchBarAudioOutputMuteTemplate", "{17, 20}"),
    ("NSTouchBarAudioOutputVolumeHighTemplate", "{26, 18}"),
    ("NSTouchBarAudioOutputVolumeLowTemplate", "{19, 17}"),
    ("NSTouchBarAudioOutputVolumeMediumTemplate", "{23, 17}"),
    ("NSTouchBarAudioOutputVolumeOffTemplate", "{16, 17}"),
    ("NSTouchBarBookmarksTemplate", "{22, 17}"),
    ("NSTouchBarColorPickerFill", "{18, 30}"),
    ("NSTouchBarColorPickerFont", "{13, 30}"),
    ("NSTouchBarColorPickerStroke", "{18, 30}"),
    ("NSTouchBarCommunicationAudioTemplate", "{20, 18}"),
    ("NSTouchBarCommunicationVideoTemplate", "{24, 16}"),
    ("NSTouchBarComposeTemplate", "{21, 20}"),
    ("NSTouchBarDeleteTemplate", "{18, 21}"),
    ("NSTouchBarDownloadTemplate", "{19, 19}"),
    ("NSTouchBarEnterFullScreenTemplate", "{20, 18}"),
    ("NSTouchBarExitFullScreenTemplate", "{20, 18}"),
    ("NSTouchBarFastForwardTemplate", "{24, 15}"),
    ("NSTouchBarFolderCopyToTemplate", "{25, 18}"),
    ("NSTouchBarFolderMoveToTemplate", "{23, 17}"),
    ("NSTouchBarFolderTemplate", "{23, 17}"),
    ("NSTouchBarGetInfoTemplate", "{19, 19}"),
    ("NSTouchBarGoBackTemplate", "{12, 16}"),
    ("NSTouchBarGoDownTemplate", "{18, 10}"),
    ("NSTouchBarGoForwardTemplate", "{12, 16}"),
    ("NSTouchBarGoUpTemplate", "{18, 10}"),
    ("NSTouchBarHistoryTemplate", "{19, 19}"),
    ("NSTouchBarIconViewTemplate", "{19, 17}"),
    ("NSTouchBarListViewTemplate", "{21, 14}"),
    ("NSTouchBarMailTemplate", "{24, 16}"),
    ("NSTouchBarNewFolderTemplate", "{25, 19}"),
    ("NSTouchBarNewMessageTemplate", "{21, 18}"),
    ("NSTouchBarOpenInBrowserTemplate", "{19, 19}"),
    ("NSTouchBarPauseTemplate", "{14, 16}"),
    ("NSTouchBarPlayPauseTemplate", "{28, 15}"),
    ("NSTouchBarPlayTemplate", "{14, 16}"),
    ("NSTouchBarPlayheadTemplate", "{7, 30}"),
    ("NSTouchBarQuickLookTemplate", "{27, 16}"),
    ("NSTouchBarRecordStartTemplate", "{19, 19}"),
    ("NSTouchBarRecordStopTemplate", "{18, 16}"),
    ("NSTouchBarRefreshTemplate", "{17, 20}"),
    ("NSTouchBarRemoveTemplate", "{18, 4}"),
    ("NSTouchBarRewindTemplate", "{24, 15}"),
    ("NSTouchBarRotateLeftTemplate", "{19, 20}"),
    ("NSTouchBarRotateRightTemplate", "{19, 20}"),
    ("NSTouchBarSearchTemplate", "{20, 19}"),
    ("NSTouchBarShareTemplate", "{19, 22}"),
    ("NSTouchBarSidebarTemplate", "{24, 17}"),
    ("NSTouchBarSkipAhead15SecondsTemplate", "{19, 21}"),
    ("NSTouchBarSkipAhead30SecondsTemplate", "{19, 21}"),
    ("NSTouchBarSkipAheadTemplate", "{30, 15}"),
    ("NSTouchBarSkipBack15SecondsTemplate", "{19, 21}"),
    ("NSTouchBarSkipBack30SecondsTemplate", "{19, 21}"),
    ("NSTouchBarSkipBackTemplate", "{30, 15}"),
    ("NSTouchBarSkipToEndTemplate", "{19, 15}"),
    ("NSTouchBarSkipToStartTemplate", "{19, 15}"),
    ("NSTouchBarSlideshowTemplate", "{24, 17}"),
    ("NSTouchBarTagIconTemplate", "{23, 21}"),
    ("NSTouchBarTextBoldTemplate", "{14, 14}"),
    ("NSTouchBarTextBoxTemplate", "{25, 17}"),
    ("NSTouchBarTextCenterAlignTemplate", "{21, 16}"),
    ("NSTouchBarTextItalicTemplate", "{13, 14}"),
    ("NSTouchBarTextJustifiedAlignTemplate", "{21, 16}"),
    ("NSTouchBarTextLeftAlignTemplate", "{21, 16}"),
    ("NSTouchBarTextListTemplate", "{21, 14}"),
    ("NSTouchBarTextRightAlignTemplate", "{21, 16}"),
    ("NSTouchBarTextStrikethroughTemplate", "{18, 15}"),
    ("NSTouchBarTextUnderlineTemplate", "{14, 17}"),
    ("NSTouchBarUserAddTemplate", "{22, 20}"),
    ("NSTouchBarUserGroupTemplate", "{27, 18}"),
    ("NSTouchBarUserTemplate", "{18, 17}"),
    ("NSTouchBarVolumeDownTemplate", "{19, 17}"),
    ("NSTouchBarVolumeUpTemplate", "{26, 18}"),
    ("NSTrashEmpty", "{32, 32}"),
    ("NSTrashFull", "{32, 32}"),
    ("NSUser", "{32, 32}"),
    ("NSUserAccounts", "{32, 32}"),
    ("NSUserGroup", "{32, 32}"),
    ("NSUserGuest", "{32, 32}"),
];

/// An image named in a control: an `NSCustomResource` of the size the
/// document's resources give, in the `system` namespace for AppKit's named
/// images. Other names starting with NS aren't established.
fn catalog_image(nib: &mut Nib, n: Node, image: &str) -> Result<Value, String> {
    let system = SYSTEM_IMAGES.iter().find(|i| i.0 == image).map(|i| i.1);
    if image.starts_with("NS") && system.is_none() {
        return fail(format!("the image {image} in a control is not supported by this ibtool"));
    }
    let resource = n
        .document()
        .descendants()
        .find(|d| d.has_tag_name("image") && d.parent_element().is_some_and(|p| p.has_tag_name("resources")) && d.attribute("name") == Some(image))
        .ok_or_else(|| format!("the image {image} without its size in the document's resources"))?;
    let size = format!("{{{}, {}}}", g(number(attr(resource, "width")?)?), g(number(attr(resource, "height")?)?));
    let Some(system_size) = system else {
        return Ok(system_image(nib, image, &size));
    };
    let class = nib.string("NSImage");
    let resource = nib.string(image);
    let namespace = nib.string("system");
    let size = nib.string(system_size);
    let size = Value::New(Object::new("NSValue", vec![("NS.special", Value::Int(2)), ("NS.sizeval", size)]));
    Ok(nib.shared(format!("resource:system:{image}"), || {
        Object::new(
            "NSCustomResource",
            vec![
                ("NSClassName", class),
                ("NSResourceName", resource),
                ("IBNamespaceID", namespace),
                ("IBDesignSize", size),
                ("IBDesignImageConfiguration", Value::Nil),
            ],
        )
    }))
}

/// A key equivalent's modifiers, as NSEvent's flags: shift 1 << 17,
/// control 1 << 18, option 1 << 19, command 1 << 20; without a mask, command
/// for an item with a key equivalent.
fn modifier_mask(n: Node) -> Result<i64, String> {
    let Some(m) = child(n, "modifierMask", "keyEquivalentModifierMask") else {
        return Ok(if n.attribute("keyEquivalent").is_some_and(|k| !k.is_empty()) { 1 << 20 } else { 0 });
    };
    only_attributes(m, &[("key", None), ("shift", Some("YES")), ("control", Some("YES")), ("option", Some("YES")), ("command", Some("YES"))])?;
    let mut mask = 0;
    for (a, bit) in [("shift", 1 << 17), ("control", 1 << 18), ("option", 1 << 19), ("command", 1 << 20)] {
        if m.attribute(a).is_some() {
            mask |= bit;
        }
    }
    Ok(mask)
}

/// A menu item: its menu (by a conditional reference), title, key
/// equivalent and the system's check and mixed-state images; one with a
/// submenu opens it with `submenuAction:`, targeting it.
fn menu_item(nib: &mut Nib, c: &mut Collected, n: Node, menu_name: &str, popup_cell: Option<&str>) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("title", None),
            ("keyEquivalent", None),
            ("tag", None),
            ("isSeparatorItem", Some("YES")),
            ("enabled", Some("NO")),
            ("state", Some("on")),
            ("indentationLevel", None),
            ("id", None),
        ],
    )?;
    if popup_cell.is_none() && n.attribute("state").is_some() {
        return fail("a menu item's state outside a pop-up button is not supported by this ibtool");
    }
    only_children(n, &["modifierMask", "menu", "connections", "string"])?;
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), menu_name.to_string()));
    c.connection_sources.push(n.attribute("id").unwrap_or_default().to_string());
    let separator = n.attribute("isSeparatorItem").is_some();
    let mut values = vec![("NSMenu", Value::Conditional(menu_name.to_string()))];
    // In a storyboard, an item opening a submenu is allowed in limited app
    // mode (Xcode's main menu templates' 20 such items; a xib's aren't).
    if nib.storyboard && children(n, "menu").next().is_some() {
        values.push(("NSAllowedForLimitedAppMode", Value::Bool(true)));
    }
    if separator || n.attribute("enabled").is_some() {
        values.push(("NSIsDisabled", Value::Bool(true)));
    }
    if separator {
        values.push(("NSIsSeparator", Value::Bool(true)));
    }
    // Its indentation, after its disabled flag (three probes).
    if let Some(level) = n.attribute("indentationLevel") {
        values.push(("NSIndent", Value::Int(level.parse().map_err(|_| format!("not an indentation level: {level}"))?)));
    }
    values.push(("NSAllowsKeyEquivalentLocalization", Value::Bool(true)));
    values.push(("NSAllowsKeyEquivalentMirroring", Value::Bool(true)));
    let title = title(n)?;
    values.push(("NSTitle", nib.text(attr(n, "id")?, "title", &title)));
    values.push(("NSKeyEquiv", nib.string(n.attribute("keyEquivalent").unwrap_or(""))));
    // A pop-up button's items have command without a mask, key equivalent
    // or not (four probes).
    let mask = match (popup_cell, child(n, "modifierMask", "keyEquivalentModifierMask")) {
        (Some(_), None) => 1 << 20,
        _ => modifier_mask(n)?,
    };
    if mask != 0 {
        values.push(("NSKeyEquivModMask", Value::Int(mask)));
    }
    values.push(("NSMnemonicLoc", Value::Int(0x7fff_ffff)));
    if n.attribute("state").is_some() {
        values.push(("NSState", Value::Int(1)));
    }
    values.push(("NSOnImage", system_image(nib, "NSMenuCheckmark", "{18, 16}")));
    values.push(("NSMixedImage", system_image(nib, "NSMenuMixedState", "{18, 4}")));
    if let Some(tag) = n.attribute("tag") {
        values.push(("NSTag", Value::Int(tag.parse().map_err(|_| format!("not a tag: {tag}"))?)));
    }
    let submenus: Vec<Node> = children(n, "menu").collect();
    if let [submenu] = submenus.as_slice() {
        let submenu_name = object_name(attr(*submenu, "id")?);
        values.push(("NSAction", nib.string("submenuAction:")));
        values.push(("NSTarget", menu(nib, c, *submenu, &name)?));
        values.push(("NSSubmenu", reference(submenu_name)));
    } else if submenus.len() > 1 {
        return fail("a menu item with several submenus");
    }
    // A pop-up button's items select themselves through its cell.
    if let Some(cell) = popup_cell {
        if !submenus.is_empty() {
            return fail("a pop-up button's submenu is not supported by this ibtool");
        }
        values.push(("NSAction", nib.string("_popUpItemAction:")));
        values.push(("NSTarget", reference(cell.to_string())));
    }
    values.push(("NSHiddenInRepresentation", Value::Bool(false)));
    Ok(Value::New(Object::new("NSMenuItem", values).named(name)))
}

/// A placeholder or custom object's class: `IBClassReference` and
/// `NSClassName` (the class the runtime looks up).
fn class_values(nib: &mut Nib, n: Node, module: Option<&str>, runtime: Option<&str>) -> Result<Vec<(&'static str, Value)>, String> {
    let Some(class) = n.attribute("customClass") else {
        return Ok(vec![("NSClassName", nib.string(runtime.unwrap_or("NSObject")))]);
    };
    let provider = n.attribute("customModuleProvider").filter(|p| !p.is_empty());
    let module_name = match (provider, module) {
        (Some(_), Some(m)) => Some(m.to_string()),
        _ => n.attribute("customModule").map(str::to_string),
    };
    let reference = Object::new(
        "IBClassReference",
        vec![
            ("IBClassName", nib.string(class)),
            ("IBModuleName", module_name.map_or(Value::Nil, |m| nib.string(&m))),
            ("IBModuleProvider", provider.map_or(Value::Nil, |p| nib.string(p))),
        ],
    );
    let runtime = match runtime {
        Some(r) => r.to_string(),
        None => runtime_class_name(n, module, "NSObject")?,
    };
    Ok(vec![("IBClassReference", Value::New(reference)), ("NSClassName", nib.string(&runtime))])
}

/// AppKit classes whose custom objects are `NSCustomObject`s.
const APPKIT_OBJECTS: [&str; 1] = ["NSFontManager"];

/// A custom object. An AppKit class's is an NSCustomObject of its class, as
/// a placeholder's (Xcode's main menu templates' font manager); the app's
/// own classes are swapped in for NSObject, made with init when the
/// document says so (its app delegate; an installer pane template).
fn custom_object(nib: &mut Nib, c: &mut Collected, t: Node, module: Option<&str>, parent: &str, direct: bool) -> Result<Value, String> {
    only_attributes(t, &[("id", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("userLabel", None)])?;
    only_children(t, &["connections"])?;
    let class = t.attribute("customClass").ok_or("a custom object without a class is not supported by this ibtool")?;
    let name = object_name(attr(t, "id")?);
    c.parents.push((name.clone(), parent.to_string()));
    c.connection_sources.push(attr(t, "id")?.to_string());
    // Without direct instantiation, any custom object is an
    // NSCustomObject of its class (two probes).
    let appkit = APPKIT_OBJECTS.contains(&class);
    if appkit || !direct {
        let values = class_values(nib, t, module, None)?;
        return Ok(Value::New(Object::new("NSCustomObject", values).named(name)));
    }
    // Other AppKit classes are made with init as the app's are (a probe of
    // NSCollectionViewItem); the rest aren't established.
    if class.starts_with("NS") && class != "NSCollectionViewItem" {
        return fail(format!("a custom object of class {class} is not supported by this ibtool"));
    }
    let values = vec![
        ("NSClassName", nib.string(&runtime_class_name(t, module, "NSObject")?)),
        ("NSOriginalClassName", nib.string("NSObject")),
        ("NSInitializeWithInit", Value::Bool(true)),
    ];
    Ok(Value::New(Object::new("NSClassSwapper", values).named(name)))
}

/// Connections: outlets, then actions (whose target is left out when it is
/// the first responder), each by their source's id (as bytes; Xcode's main
/// menu template's 87, and two corpus xibs), in document order for one
/// source.
fn connections(nib: &mut Nib, doc: &Document, sources: &[String], first_responder: &str) -> Result<Vec<(String, Value)>, String> {
    let by_id: HashMap<&str, Node> = doc.descendants().filter(|d| d.is_element()).filter_map(|d| d.attribute("id").map(|id| (id, d))).collect();
    let mut sorted: Vec<&str> = sources.iter().map(String::as_str).collect();
    sorted.sort();
    // A view at the top level is collected as one twice.
    sorted.dedup();
    let mut out = Vec::new();
    for kind in ["outlet", "action"] {
        for id in &sorted {
            let id = *id;
            let source = by_id.get(id).ok_or("a connection's source")?;
            for list in children(*source, "connections") {
                only_children(list, &["outlet", "action"])?;
                for o in list.children().filter(|c| c.has_tag_name(kind)) {
                    let connector = if o.has_tag_name("outlet") {
                        only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
                        Object::new(
                            "NSNibOutletConnector",
                            vec![
                                ("NSSource", reference(object_name(id))),
                                ("NSDestination", reference(object_name(attr(o, "destination")?))),
                                ("NSLabel", nib.string(attr(o, "property")?)),
                                ("NSChildControllerCreationSelectorName", Value::Nil),
                            ],
                        )
                    } else {
                        only_attributes(o, &[("selector", None), ("target", None), ("id", None)])?;
                        let mut values = vec![("NSSource", reference(object_name(id)))];
                        let target = attr(o, "target")?;
                        if target != first_responder {
                            values.push(("NSDestination", reference(object_name(target))));
                        }
                        // An old storyboard's toggleSourceList: is AppKit's
                        // toggleSidebar: (three corpus storyboards).
                        let selector = match attr(o, "selector")? {
                            "toggleSourceList:" => "toggleSidebar:",
                            s => s,
                        };
                        values.push(("NSLabel", nib.string(selector)));
                        Object::new("NSNibControlConnector", values)
                    };
                    let name = object_name(attr(o, "id")?);
                    out.push((name.clone(), Value::New(connector.named(name))));
                }
            }
        }
    }
    Ok(out)
}

/// The nib of a root object (the File's Owner, or a storyboard nib's
/// NSObject), its objects with their parents (each defined where the
/// encoder first meets it: those `c.definitions` holds stand in the
/// objects' list) and connections.
fn assemble(nib: &mut Nib, root_object: Object, root_name: &str, mut c: Collected, connections: Vec<(String, Value)>) -> Result<Vec<u8>, String> {
    let visible: Vec<Value> = c.visible.iter().map(|v| reference(v.clone())).collect();
    let visible = nib.array("NSMutableSet", visible);
    let mut oids = vec![reference(root_name.to_string())];
    let mut keys = Vec::new();
    let mut parents = Vec::new();
    for (object, parent) in &c.parents {
        oids.push(reference(object.clone()));
        parents.push(reference(parent.clone()));
        keys.push(c.definitions.remove(object).unwrap_or_else(|| reference(object.clone())));
    }
    let mut connection_values = Vec::new();
    for (name, value) in connections {
        oids.push(reference(name));
        connection_values.push(value);
    }
    let numbers: Vec<Value> = (1..=oids.len() as i64).map(|i| nib.int_number(i)).collect();
    // Accessibility identifiers: a connector each, by the view's id as
    // bytes, numbered after the other objects (a rules_apple example, three
    // probes).
    c.accessibility.sort();
    let mut ax_connectors = Vec::new();
    let mut ax_keys = Vec::new();
    let mut ax_numbers = Vec::new();
    for (i, (id, identifier)) in c.accessibility.iter().enumerate() {
        let name = format!("{} accessibility", object_name(id));
        let kind = Value::New(Object::new("NSMutableString", vec![("NS.bytes", Value::Data(b"AXIdentifier".to_vec()))]));
        let value = nib.string(identifier);
        ax_connectors.push(Value::New(
            Object::new(
                "NSNibAXAttributeConnector",
                vec![
                    ("AXDestinationArchiveKey", reference(object_name(id))),
                    ("AXAttributeTypeArchiveKey", kind),
                    ("AXAttributeValueArchiveKey", value),
                ],
            )
            .named(name.clone()),
        ));
        ax_keys.push(reference(name));
        ax_numbers.push(nib.int_number(numbers.len() as i64 + 1 + i as i64));
    }
    let (ax_keys, ax_values) = if ax_keys.is_empty() {
        let empty = nib.empty_array();
        (empty, nib.empty_array())
    } else {
        (nib.array("NSArray", ax_keys), nib.array("NSArray", ax_numbers))
    };
    let data = Object::new(
        "NSIBObjectData",
        vec![
            ("NSRoot", Value::New(root_object)),
            ("NSVisibleWindows", visible),
            ("NSConnections", nib.array("NSMutableArray", connection_values)),
            ("NSObjectsKeys", nib.array("NSArray", keys)),
            ("NSObjectsValues", nib.array("NSArray", parents)),
            ("NSOidsKeys", nib.array("NSArray", oids)),
            ("NSOidsValues", nib.array("NSArray", numbers)),
            ("NSAccessibilityConnectors", nib.array("NSMutableArray", ax_connectors)),
            ("NSAccessibilityOidsKeys", ax_keys),
            ("NSAccessibilityOidsValues", ax_values),
        ],
    );
    nib::archive_appkit(Object::new("NSObject", vec![("IB.objectdata", Value::New(data)), ("IB.systemFontUpdateVersion", Value::Int(1))]))
}

fn new_nib(doc: &Document, localizable: bool, storyboard: bool) -> Nib {
    Nib {
        shared: HashMap::new(),
        next: 0,
        localizable,
        storyboard,
        constrained: constrained(doc),
        old_custom: !storyboard && doc.root_element().attribute("customObjectInstantitationMethod") != Some("direct"),
    }
}

/// The ids of the views constraints position: their items, and the view
/// holding one without a first item. Only such a view that doesn't
/// translate its mask says so (`NSDoNotTranslateAutoresizingMask`; a
/// constrained probe's subview, a text field's template unconstrained).
fn constrained(doc: &Document) -> std::collections::HashSet<String> {
    let mut ids = std::collections::HashSet::new();
    for k in doc.descendants().filter(|d| d.has_tag_name("constraint")) {
        let owner = k.parent().and_then(|p| p.parent()).and_then(|v| v.attribute("id"));
        ids.extend(k.attribute("firstItem").or(owner).map(str::to_string));
        ids.extend(k.attribute("secondItem").map(str::to_string));
    }
    ids
}

fn new_collected(sources: Vec<String>) -> Collected {
    Collected {
        parents: Vec::new(),
        visible: Vec::new(),
        connection_sources: sources,
        definitions: HashMap::new(),
        at_origin: Vec::new(),
        laid_out: HashMap::new(),
        accessibility: Vec::new(),
    }
}

/// Compiles an AppKit xib to its nib.
pub fn compile_xib(xml: &str, module: Option<&str>, localizable: bool) -> Result<Vec<u8>, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    if root.attribute("targetRuntime") != Some("MacOSX.Cocoa") {
        return fail("an AppKit xib for another runtime than MacOSX.Cocoa is not supported by this ibtool");
    }
    only_children(root, &["dependencies", "objects", "resources"])?;
    let objects_node = children(root, "objects").next().ok_or("a xib without objects")?;
    let mut nib = new_nib(&doc, localizable, false);
    let mut c = new_collected(vec!["-2".into()]);
    let mut owner = None;
    let mut application = None;
    let mut tops = Vec::new();
    for o in objects_node.children().filter(|c| c.is_element()) {
        match (o.tag_name().name(), o.attribute("id")) {
            ("customObject", Some("-2")) => owner = Some(o),
            ("customObject", Some("-1")) => {
                only_attributes(o, &[("id", None), ("userLabel", None), ("customClass", Some("FirstResponder"))])?;
                only_children(o, &[])?;
            }
            ("customObject", Some("-3")) => application = Some(o),
            ("customView" | "window" | "customObject" | "menu" | "view" | "box" | "textField" | "button" | "popUpButton" | "imageView" | "scrollView" | "collectionViewItem", _) => tops.push(o),
            (tag, _) => return fail(format!("a xib's top-level <{tag}> is not supported by this ibtool")),
        }
    }
    let owner = owner.ok_or("a xib without a File's Owner")?;
    only_attributes(
        owner,
        &[("id", None), ("userLabel", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None)],
    )?;
    only_children(owner, &["connections"])?;
    let owner_name = object_name("-2");
    let root_object = Object::new("NSCustomObject", class_values(&mut nib, owner, module, None)?).named(owner_name.clone());

    // The application placeholder: its class, NSApplication at run time.
    let application = application.ok_or("a xib without an Application placeholder")?;
    application_placeholder(&mut nib, &mut c, application, module, &owner_name)?;

    let direct = root.attribute("customObjectInstantitationMethod") == Some("direct");
    for t in &tops {
        let value = match t.tag_name().name() {
            "window" => {
                c.connection_sources.push(attr(*t, "id")?.to_string());
                window(&mut nib, &mut c, *t, module, &owner_name)?
            }
            "menu" => menu(&mut nib, &mut c, *t, &owner_name)?,
            "customObject" => custom_object(&mut nib, &mut c, *t, module, &owner_name, direct)?,
            "collectionViewItem" => collection_item(&mut nib, &mut c, *t, module, &owner_name)?,
            "box" | "textField" | "button" | "popUpButton" | "imageView" | "scrollView" => match t.tag_name().name() {
                "box" => box_view(&mut nib, &mut c, *t, module, &owner_name)?,
                "textField" => text_field(&mut nib, &mut c, *t, module, &owner_name)?,
                "button" | "popUpButton" => button(&mut nib, &mut c, *t, module, &owner_name)?,
                "imageView" => image_view(&mut nib, &mut c, *t, module, &owner_name)?,
                _ => scroll_view(&mut nib, &mut c, *t, module, &owner_name)?,
            },
            _ => {
                c.connection_sources.push(attr(*t, "id")?.to_string());
                view(&mut nib, &mut c, *t, module, &owner_name, None)?
            }
        };
        c.definitions.insert(object_name(attr(*t, "id")?), value);
    }
    let connections = connections(&mut nib, &doc, &c.connection_sources.clone(), "-1")?;
    assemble(&mut nib, root_object, &owner_name, c, connections)
}

/// A xib's Application placeholder (`-3`): of class NSObject, or without a
/// class, it refers to NSObject; of NSApplication, it names that alone
/// (templates).
fn application_placeholder(nib: &mut Nib, c: &mut Collected, application: Node, module: Option<&str>, parent: &str) -> Result<(), String> {
    only_attributes(application, &[("id", None), ("userLabel", None), ("customClass", None)])?;
    // Its outlets, as an older main menu's delegate (a corpus xib).
    only_children(application, &["connections"])?;
    if children(application, "connections").next().is_some() {
        c.connection_sources.push(attr(application, "id")?.to_string());
    }
    let name = object_name(attr(application, "id")?);
    let values = match application.attribute("customClass") {
        Some("NSObject") => class_values(nib, application, module, Some("NSApplication"))?,
        Some("NSApplication") => vec![("NSClassName", nib.string("NSApplication"))],
        // Without a class, as NSObject's (Xcode's application UI template).
        None => {
            let class = nib.string("NSObject");
            let reference = Object::new("IBClassReference", vec![("IBClassName", class), ("IBModuleName", Value::Nil), ("IBModuleProvider", Value::Nil)]);
            vec![("IBClassReference", Value::New(reference)), ("NSClassName", nib.string("NSApplication"))]
        }
        _ => return fail("an Application placeholder of a class other than NSObject or NSApplication is not supported by this ibtool"),
    };
    c.parents.push((name.clone(), parent.to_string()));
    c.definitions.insert(name.clone(), Value::New(Object::new("NSCustomObject", values).named(name)));
    Ok(())
}

/// Compiles an AppKit storyboard: its files in the `.storyboardc`. The
/// application scene's main menu is `MainMenu.nib`; the Info.plist names it
/// whether or not there is one (Xcode's storyboard template).
pub fn compile_storyboard(xml: &str, module: Option<&str>, localizable: bool) -> Result<Vec<(String, Vec<u8>)>, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    if root.attribute("targetRuntime") != Some("MacOSX.Cocoa") {
        return fail("an AppKit storyboard for another runtime than MacOSX.Cocoa is not supported by this ibtool");
    }
    // Inferred metrics' tie breakers are the editor's (a corpus storyboard
    // compiles the same without them).
    only_children(root, &["dependencies", "scenes", "resources", "inferredMetricsTieBreakers"])?;
    let mut files = Vec::new();
    let mut nib_names: Vec<(String, String)> = Vec::new();
    let mut uuids: Vec<(String, String)> = Vec::new();
    // Identified view controllers' identifiers.
    let mut identified: Vec<String> = Vec::new();
    let mut entry = None;
    // The controllers windows' content segues lead to.
    let content_controllers: Vec<&str> = doc
        .descendants()
        .filter(|d| d.has_tag_name("segue") && d.attribute("relationship") == Some("window.shadowedContentViewController"))
        .filter_map(|d| d.attribute("destination"))
        .collect();
    // The controllers embed segues lead to.
    let embedded: Vec<&str> = doc
        .descendants()
        .filter(|d| d.has_tag_name("segue") && d.attribute("kind") == Some("embed"))
        .filter_map(|d| d.attribute("destination"))
        .collect();
    for s in doc.descendants().filter(|d| d.has_tag_name("segue")) {
        let content = s.attribute("relationship") == Some("window.shadowedContentViewController") && s.attribute("kind") == Some("relationship");
        let embed = s.attribute("kind") == Some("embed") && s.parent().and_then(|l| l.parent()).is_some_and(|v| v.has_tag_name("containerView"));
        if !content && !embed {
            return fail("segues other than a window's content relationship or a container view's embedding are not supported by this ibtool");
        }
    }
    for scenes in children(root, "scenes") {
        for scene in children(scenes, "scene") {
            only_attributes(scene, &[("sceneID", None)])?;
            only_children(scene, &["objects", "point"])?;
            canvas_location_only(scene)?;
            let objects = children(scene, "objects").next().ok_or("a scene without objects")?;
            let members: Vec<Node> = objects.children().filter(|c| c.is_element()).collect();
            let first_responder = members
                .iter()
                .find(|m| m.attribute("sceneMemberID") == Some("firstResponder"))
                .ok_or("a scene without a first responder")?;
            only_attributes(*first_responder, &[("id", None), ("userLabel", None), ("customClass", Some("NSResponder")), ("sceneMemberID", None)])?;
            only_children(*first_responder, &[])?;
            let controller = members.iter().find(|m| m.attribute("sceneMemberID") == Some("viewController")).ok_or("a scene without a controller")?;
            let others: Vec<Node> = members.iter().copied().filter(|m| m != first_responder && m != controller).collect();
            match controller.tag_name().name() {
                "windowController" => {
                    let id = attr(*controller, "id")?;
                    // Named by its identifier when it has one, which its
                    // nib name's and UUID's keys share (Xcode's document app
                    // template).
                    let identifier = match controller.attribute("storyboardIdentifier") {
                        Some(i) => {
                            identified.push(i.to_string());
                            i.to_string()
                        }
                        None => format!("NSWindowController-{id}"),
                    };
                    let (bytes, uuid) = window_controller_nib(&doc, xml, *controller, &others, attr(*first_responder, "id")?, module, localizable)?;
                    files.push((format!("{identifier}.nib"), bytes));
                    nib_names.push((identifier.clone(), identifier.clone()));
                    uuids.push((identifier.clone(), uuid));
                    if root.attribute("initialViewController") == Some(id) {
                        entry = Some(identifier);
                    }
                }
                // A window's content controller: its view's nib (the window
                // controller's holds the controller).
                "viewController" if content_controllers.contains(&attr(*controller, "id")?) => {
                    if !others.is_empty() || controller.attribute("storyboardIdentifier").is_some() {
                        return fail("a content controller with an identifier or objects beside it is not supported by this ibtool");
                    }
                    let (name, bytes) = view_nib(&doc, *controller, attr(*first_responder, "id")?, module, localizable)?;
                    files.push((format!("{name}.nib"), bytes));
                }
                // An embedded view controller: its nib, named for its id, and
                // its view's (four probes).
                "viewController" if embedded.contains(&attr(*controller, "id")?) => {
                    let id = attr(*controller, "id")?;
                    if !others.is_empty() || controller.attribute("storyboardIdentifier").is_some() || content_controllers.contains(&id) {
                        return fail("an embedded view controller with an identifier, objects beside it or a window is not supported by this ibtool");
                    }
                    let identifier = format!("NSViewController-{id}");
                    files.push((format!("{identifier}.nib"), controller_nib(&doc, xml, *controller, module, localizable, false)?));
                    let (name, bytes) = view_nib(&doc, *controller, attr(*first_responder, "id")?, module, localizable)?;
                    files.push((format!("{name}.nib"), bytes));
                    nib_names.push((identifier.clone(), identifier.clone()));
                    uuids.push((identifier, uuid(xml, id)));
                }
                // The initial view controller: its nib, named for its id, as
                // an identified one's (Xcode's notification content
                // extension template).
                "viewController" if root.attribute("initialViewController") == controller.attribute("id") && controller.attribute("storyboardIdentifier").is_none() => {
                    let id = attr(*controller, "id")?;
                    if !others.is_empty() || content_controllers.contains(&id) || embedded.contains(&id) {
                        return fail("an initial view controller with objects beside it, a window or a container is not supported by this ibtool");
                    }
                    let identifier = format!("NSViewController-{id}");
                    files.push((format!("{identifier}.nib"), controller_nib(&doc, xml, *controller, module, localizable, true)?));
                    let (name, bytes) = view_nib(&doc, *controller, attr(*first_responder, "id")?, module, localizable)?;
                    files.push((format!("{name}.nib"), bytes));
                    nib_names.push((identifier.clone(), identifier.clone()));
                    uuids.push((identifier.clone(), uuid(xml, id)));
                    entry = Some(identifier);
                }
                // An identified view controller: its nib and its view's.
                "viewController" if controller.attribute("storyboardIdentifier").is_some() => {
                    if !others.is_empty() {
                        return fail("objects beside a view controller are not supported by this ibtool");
                    }
                    let identifier = attr(*controller, "storyboardIdentifier")?;
                    files.push((format!("{identifier}.nib"), controller_nib(&doc, xml, *controller, module, localizable, true)?));
                    let (name, bytes) = view_nib(&doc, *controller, attr(*first_responder, "id")?, module, localizable)?;
                    files.push((format!("{name}.nib"), bytes));
                    nib_names.push((identifier.to_string(), identifier.to_string()));
                    uuids.push((identifier.to_string(), uuid(xml, attr(*controller, "id")?)));
                    identified.push(identifier.to_string());
                }
                "application" => {
                    let nib = main_menu(&doc, *controller, &others, attr(*first_responder, "id")?, module, localizable)?;
                    files.push(("MainMenu.nib".to_string(), nib));
                    nib_names.push(("MainMenu".into(), "MainMenu".into()));
                }
                // A controller nothing loads (no identifier, not initial,
                // no segue to it) compiles to nothing (Xcode's storyboard
                // template).
                _ if controller.attribute("storyboardIdentifier").is_none()
                    && root.attribute("initialViewController") != controller.attribute("id")
                    && !doc.descendants().any(|d| d.has_tag_name("segue") && d.attribute("destination") == controller.attribute("id")) =>
                {
                    if !others.is_empty() {
                        return fail("objects beside a controller nothing loads are not supported by this ibtool");
                    }
                }
                tag => return fail(format!("an AppKit storyboard's <{tag}> scene is not supported by this ibtool")),
            }
        }
    }
    if root.attribute("initialViewController").is_some() && entry.is_none() {
        return fail("an initial controller other than a window or view controller is not supported by this ibtool");
    }
    // Its keys in the order CoreFoundation's table gives them inserted so
    // (three probes' and templates' key sets).
    let mut info = vec![("NSStoryboardMainMenu", cf::Value::string("MainMenu"))];
    if !uuids.is_empty() {
        info.push(("NSViewControllerIdentifiersToUUIDs", cf::Value::dict(uuids.into_iter().map(|(k, v)| (k, cf::Value::string(v))))));
    }
    if let Some(e) = &entry {
        info.push(("NSStoryboardDesignatedEntryPointIdentifier", cf::Value::string(e.clone())));
    }
    info.push(("NSStoryboardVersion", cf::Value::Integer(1)));
    info.push(("NSViewControllerIdentifiersToNibNames", cf::Value::dict(nib_names.into_iter().map(|(k, v)| (k, cf::Value::string(v))))));
    let info = cf::Value::dict(info);
    // The main menu's nib name is the string naming the main menu (a
    // template); an identified controller's identifier is one string for
    // its nib name's and UUID's keys (a probe), a window controller's
    // generated name a new one each time.
    let share = |_: &cf::Value, at: &cf::binary::Position| {
        use cf::binary::Step;
        match at {
            [Step::Value(k)] => k == "NSStoryboardMainMenu",
            [Step::Value(k), Step::Key(id)] if k == "NSViewControllerIdentifiersToNibNames" => id == "MainMenu" || identified.contains(id),
            [Step::Value(k), Step::Key(id)] if k == "NSViewControllerIdentifiersToUUIDs" => identified.contains(id),
            _ => false,
        }
    };
    files.push(("Info.plist".to_string(), cf::binary::to_bytes_with(&info, &share, cf::binary::Dictionaries::Grown)));
    Ok(files)
}

/// The application scene's nib: an NSObject root, the application (its
/// main menu, NSApplication at run time), the scene's objects, an
/// Application placeholder, then the menu.
fn main_menu(doc: &Document, application: Node, others: &[Node], first_responder: &str, module: Option<&str>, localizable: bool) -> Result<Vec<u8>, String> {
    only_attributes(application, &[("id", None), ("sceneMemberID", None)])?;
    only_children(application, &["menu", "connections"])?;
    let mut nib = new_nib(doc, localizable, true);
    let app_id = attr(application, "id")?;
    let mut c = new_collected(vec![app_id.to_string()]);
    let root_name = object_name("root");
    let root_object = Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSObject"))]).named(root_name.clone());
    let app_name = object_name(app_id);
    c.parents.push((app_name.clone(), root_name.clone()));
    let direct = true;
    for o in others {
        if !o.has_tag_name("customObject") {
            return fail(format!("an application scene's <{}> is not supported by this ibtool", o.tag_name().name()));
        }
        let value = custom_object(&mut nib, &mut c, *o, module, &root_name, direct)?;
        c.definitions.insert(object_name(attr(*o, "id")?), value);
    }
    let placeholder = object_name("application placeholder");
    c.parents.push((placeholder.clone(), root_name.clone()));
    let value = Value::New(Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSApplication"))]).named(placeholder.clone()));
    c.definitions.insert(placeholder, value);
    let menus: Vec<Node> = children(application, "menu").collect();
    let [main] = menus.as_slice() else {
        return fail("an application without one main menu is not supported by this ibtool");
    };
    if main.attribute("key") != Some("mainMenu") {
        return fail("an application's menu other than its main menu is not supported by this ibtool");
    }
    let menu_value = menu(&mut nib, &mut c, *main, &root_name)?;
    let app = Object::new("NSCustomObject", vec![("IBMainMenu", menu_value), ("NSClassName", nib.string("NSApplication"))]).named(app_name.clone());
    c.definitions.insert(app_name, Value::New(app));
    let connections = connections(&mut nib, doc, &c.connection_sources.clone(), first_responder)?;
    assemble(&mut nib, root_object, &root_name, c, connections)
}

/// A UUID for a controller's `uniqueIdentifierForStoryboardCompilation`,
/// which Xcode makes anew on every run: this ibtool derives one from the
/// document and the controller's id, so that its output is reproducible.
fn uuid(xml: &str, id: &str) -> String {
    let mut h: u128 = 0x6c62272e07bb014262b821756295c58d;
    for b in xml.bytes().chain([0]).chain(id.bytes()) {
        h ^= b as u128;
        h = h.wrapping_mul(0x0000000001000000000000000000013b);
    }
    let hex = format!("{h:032X}");
    format!("{}-{}-4{}-8{}-{}", &hex[0..8], &hex[8..12], &hex[13..16], &hex[17..20], &hex[20..32])
}

/// A storyboard placeholder, as a controller's `storyboard` outlet names it.
fn storyboard_placeholder(nib: &mut Nib, c: &mut Collected, name: &str, parent: &str) {
    c.parents.push((name.to_string(), parent.to_string()));
    let identifier = nib.string("NSStoryboardPlaceholder");
    c.definitions.insert(
        name.to_string(),
        Value::New(Object::new("NSNibExternalObjectPlaceholder", vec![("NSExternalObjectPlaceholderIdentifier", identifier)]).named(name.to_string())),
    );
}

fn outlet(name: String, source: &str, destination: &str, label: Value) -> (String, Value) {
    let connector = Object::new(
        "NSNibOutletConnector",
        vec![
            ("NSSource", reference(source.to_string())),
            ("NSDestination", reference(destination.to_string())),
            ("NSLabel", label),
            ("NSChildControllerCreationSelectorName", Value::Nil),
        ],
    );
    (name.clone(), Value::New(connector.named(name)))
}

/// A window controller's nib: an NSObject root (its scene controller), the
/// controller holding its window and the window's content controller, each
/// with a storyboard placeholder, and the content controller set on the
/// window as a runtime attribute. Xcode lists the connections in an order
/// that varies from run to run, and gives each controller a new UUID; this
/// ibtool keeps one order of each.
fn window_controller_nib(doc: &Document, xml: &str, wc: Node, others: &[Node], first_responder: &str, module: Option<&str>, localizable: bool) -> Result<(Vec<u8>, String), String> {
    only_attributes(
        wc,
        &[
            ("id", None),
            ("sceneMemberID", None),
            ("showSeguePresentationStyle", Some("single")),
            ("storyboardIdentifier", None),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
        ],
    )?;
    // Shown as a single window, presentation style 1 (a corpus storyboard).
    let presentation = if wc.attribute("showSeguePresentationStyle").is_some() { 1 } else { 0 };
    only_children(wc, &["window", "connections"])?;
    if !others.is_empty() {
        return fail("objects beside a window controller are not supported by this ibtool");
    }
    let mut nib = new_nib(doc, localizable, true);
    let wc_id = attr(wc, "id")?;
    let window_node = children(wc, "window").next().ok_or("a window controller without a window")?;
    let segues: Vec<Node> = children(wc, "connections").flat_map(|l| l.children().filter(|c| c.is_element())).collect();
    let [segue] = segues.as_slice() else {
        return fail("a window controller without one content segue is not supported by this ibtool");
    };
    let vc = doc
        .descendants()
        .find(|d| d.attribute("id") == segue.attribute("destination") && d.attribute("sceneMemberID") == Some("viewController"))
        .ok_or("a content segue's destination")?;
    if !vc.has_tag_name("viewController") {
        return fail("a window's content controller other than a view controller is not supported by this ibtool");
    }
    let root_name = object_name("root");
    let root_object = Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSObject"))]).named(root_name.clone());
    let mut c = new_collected(vec![attr(window_node, "id")?.to_string()]);
    let wc_name = object_name(wc_id);
    let vc_name = object_name(attr(vc, "id")?);
    let vc_placeholder = format!("{vc_name} storyboard");
    let wc_placeholder = format!("{wc_name} storyboard");
    storyboard_placeholder(&mut nib, &mut c, &vc_placeholder, &root_name);
    c.parents.push((wc_name.clone(), root_name.clone()));
    let window_value = window(&mut nib, &mut c, window_node, module, &wc_name)?;
    let window_name = object_name(attr(window_node, "id")?);
    c.parents.push((vc_name.clone(), window_name.clone()));
    let placeholder = object_name("application placeholder");
    c.parents.push((placeholder.clone(), root_name.clone()));
    let value = Value::New(Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSApplication"))]).named(placeholder.clone()));
    c.definitions.insert(placeholder, value);
    storyboard_placeholder(&mut nib, &mut c, &wc_placeholder, &root_name);

    let (vc_value, templates) = content_controller(&mut nib, xml, vc, module, &vc_name, false)?;
    let wc_uuid = uuid(xml, wc_id);
    let mut wc_values = vec![("IBWindowTemplate", window_value), ("IBWindowTemplateContentViewController", vc_value)];
    // Its identifier, when it has one (Xcode's document app template).
    if let Some(identifier) = wc.attribute("storyboardIdentifier") {
        wc_values.push(("explicitStoryboardIdentifier", nib.string(identifier)));
    }
    wc_values.push(("showSeguePresentationStyle", Value::Int(presentation)));
    wc_values.push(("uniqueIdentifierForStoryboardCompilation", nib.string(&wc_uuid)));
    // Shown as a single window: its UUID names the single instance (Xcode's
    // Safari extension templates).
    if presentation == 1 {
        let key = nib.string("NSSingleInstancePresentationIdentifier");
        let id = nib.string(&wc_uuid);
        wc_values.push((
            "NSStoryboardSegueDestinationOptions",
            Value::New(Object::new(
                "NSDictionary",
                vec![("NSInlinedValue", Value::Bool(true)), ("UINibEncoderEmptyKey", key), ("UINibEncoderEmptyKey", id)],
            )),
        ));
    }
    // Of a custom class, swapped in for NSWindowController (a corpus
    // storyboard).
    let wc_object = swapped(&mut nib, wc, module, "NSWindowController", wc_values, wc_name.clone())?;
    c.definitions.insert(wc_name.clone(), wc_object);

    // The scene controller first, then the window's own connections, the
    // storyboard outlets, the window outlet and the content controller.
    let label = nib.string("sceneController");
    let mut connections = vec![outlet(format!("{wc_name} scene controller"), &root_name, &wc_name, label)];
    connections.extend(self::connections(&mut nib, doc, &c.connection_sources.clone(), first_responder)?);
    // The window's delegate is its controller, connected or not (Xcode's
    // game templates leave it out of the document).
    let has_delegate = children(window_node, "connections")
        .flat_map(|l| children(l, "outlet"))
        .any(|o| o.attribute("property") == Some("delegate"));
    if !has_delegate {
        let label = nib.string("delegate");
        connections.push(outlet(format!("{window_name} delegate outlet"), &window_name, &wc_name, label));
    }
    add_templates(&mut nib, &mut c, &mut connections, &templates, &vc_name, &vc_placeholder);
    let label = nib.string("storyboard");
    connections.push(outlet(format!("{vc_name} storyboard outlet"), &vc_name, &vc_placeholder, label));
    let label = nib.string("storyboard");
    connections.push(outlet(format!("{wc_name} storyboard outlet"), &wc_name, &wc_placeholder, label));
    let label = nib.string("window");
    connections.push(outlet(format!("{wc_name} window outlet"), &wc_name, &window_name, label));
    let values = nib.array("NSArray", vec![reference(vc_name.clone())]);
    let key_path = nib.string("contentViewController");
    let key_paths = nib.array("NSArray", vec![key_path]);
    let runtime = format!("{wc_name} content controller");
    let connector = Object::new(
        "NSIBUserDefinedRuntimeAttributesConnector",
        vec![("NSObject", reference(window_name)), ("NSValues", values), ("NSKeyPaths", key_paths)],
    )
    .named(runtime.clone());
    connections.push((runtime, Value::New(connector)));
    Ok((assemble(&mut nib, root_object, &root_name, c, connections)?, wc_uuid))
}

/// A view controller as a window controller's or its own nib holds it:
/// its class swapped in for NSViewController, naming its view's nib and its
/// identifier. In its own nib it is always swapped, requiring the swapper
/// for connections; as a window's content without a class, it is a plain
/// NSViewController (eight probes, a template, rules_apple's example).
fn content_controller(nib: &mut Nib, xml: &str, vc: Node, module: Option<&str>, name: &str, own_nib: bool) -> Result<(Value, Vec<String>), String> {
    only_attributes(
        vc,
        &[
            ("id", None),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
            ("sceneMemberID", None),
            ("storyboardIdentifier", None),
            ("title", None),
            ("showSeguePresentationStyle", Some("single")),
        ],
    )?;
    let single = vc.attribute("showSeguePresentationStyle").is_some();
    if single && own_nib {
        return fail("an identified view controller shown as a single window is not supported by this ibtool");
    }
    let view = controller_view(vc).ok_or("a view controller without a view")?;
    let nib_name = format!("{}-view-{}", attr(vc, "id")?, attr(view, "id")?);
    // A window's content controller without a class is a plain
    // NSViewController (rules_apple's example, two probes).
    let swapped = own_nib || vc.attribute("customClass").is_some();
    let mut values = Vec::new();
    if swapped {
        values.push(("NSClassName", nib.string(&runtime_class_name(vc, module, "NSViewController")?)));
        values.push(("NSOriginalClassName", nib.string("NSViewController")));
    }
    values.push(("NSNibName", nib.string(&nib_name)));
    // Its embed segues' templates, made as its view loads (four probes).
    let segues = embeds(vc);
    let mut templates = Vec::new();
    if !segues.is_empty() && (vc.attribute("storyboardIdentifier").is_some() || vc.attribute("title").is_some()) {
        return fail("an identified or titled view controller's embed segues are not supported by this ibtool");
    }
    for segue in &segues {
        let template_name = object_name(&format!("{} template", attr(*segue, "id")?));
        let mut t = Vec::new();
        if let Some(identifier) = segue.attribute("identifier") {
            t.push(("NSIdentifier", nib.string(identifier)));
        }
        t.push(("NSTrigger", nib.string("IBSegueTriggerViewDidLoad")));
        t.push(("NSDestinationControllerIdentifier", nib.string(&format!("NSViewController-{}", attr(*segue, "destination")?))));
        t.push(("NSContainerView", Value::Nil));
        templates.push((template_name.clone(), Value::New(Object::new("NSStoryboardEmbedSegueTemplate", t).named(template_name))));
    }
    let template_names: Vec<String> = templates.iter().map(|t| t.0.clone()).collect();
    let template_names_out = template_names.clone();
    if !templates.is_empty() {
        let list = templates.into_iter().map(|t| t.1).collect();
        values.push(("NSStoryboardSegueTemplates", nib.array("NSArray", list)));
    }
    if let Some(identifier) = vc.attribute("storyboardIdentifier") {
        values.push(("NSStoryboardIdentifier", nib.string(identifier)));
    }
    // Its title after its identifier (a probe).
    if let Some(title) = vc.attribute("title") {
        values.push(("NSTitle", nib.text(attr(vc, "id")?, "title", title)));
    }
    // Its view's nib names it as its upstream placeholder when connections
    // need it there (Xcode's SpriteKit template).
    let mut table = vec![("NSInlinedValue", Value::Bool(true))];
    let mut upstream: Vec<String> = Vec::new();
    if uses_upstream(vc) {
        upstream.push(name.to_string());
    }
    upstream.extend(template_names);
    for (i, target) in upstream.into_iter().enumerate() {
        table.push(("UINibEncoderEmptyKey", nib.string(&format!("UpstreamPlaceholder-{}", i + 1))));
        table.push(("UINibEncoderEmptyKey", reference(target)));
    }
    values.push(("NSExternalObjectsTableForViewLoading", Value::New(Object::new("NSDictionary", table))));
    // Shown as a single window: style 1, its UUID naming the single
    // instance, as a window controller's (rules_apple's Buttons example).
    values.push(("showSeguePresentationStyle", Value::Int(if single { 1 } else { 0 })));
    let vc_uuid = uuid(xml, attr(vc, "id")?);
    values.push(("uniqueIdentifierForStoryboardCompilation", nib.string(&vc_uuid)));
    if single {
        let key = nib.string("NSSingleInstancePresentationIdentifier");
        let id = nib.string(&vc_uuid);
        values.push((
            "NSStoryboardSegueDestinationOptions",
            Value::New(Object::new(
                "NSDictionary",
                vec![("NSInlinedValue", Value::Bool(true)), ("UINibEncoderEmptyKey", key), ("UINibEncoderEmptyKey", id)],
            )),
        ));
    }
    if own_nib {
        values.push(("connectionsRequireClassSwapperForStoryboardCompilation", Value::Bool(true)));
    }
    let class = if swapped { "NSClassSwapper" } else { "NSViewController" };
    Ok((Value::New(Object::new(class, values).named(name.to_string())), template_names_out))
}

/// A controller's embed segue templates in its nib: listed after it, in
/// reverse order (three probes), each with outlets to the controller and to
/// its storyboard placeholder.
fn add_templates(nib: &mut Nib, c: &mut Collected, connections: &mut Vec<(String, Value)>, templates: &[String], vc_name: &str, storyboard: &str) {
    let at = c.parents.iter().position(|p| p.0 == vc_name).map_or(c.parents.len(), |i| i + 1);
    for t in templates {
        c.parents.insert(at, (t.clone(), vc_name.to_string()));
        let label = nib.string("controller");
        connections.push(outlet(format!("{t} controller outlet"), t, vc_name, label));
        let label = nib.string("storyboard");
        connections.push(outlet(format!("{t} storyboard outlet"), t, storyboard, label));
    }
}

/// A view controller's own nib: one with a storyboard identifier, named by
/// it, or one a container view embeds, named `NSViewController-<id>`
/// (`identified` false), holding the controller (its scene controller) and
/// a storyboard placeholder (probes).
fn controller_nib(doc: &Document, xml: &str, vc: Node, module: Option<&str>, localizable: bool, identified: bool) -> Result<Vec<u8>, String> {
    let mut nib = new_nib(doc, localizable, true);
    let root_name = object_name("root");
    let root_object = Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSObject"))]).named(root_name.clone());
    let mut c = new_collected(Vec::new());
    let vc_name = object_name(attr(vc, "id")?);
    c.parents.push((vc_name.clone(), root_name.clone()));
    let (value, templates) = content_controller(&mut nib, xml, vc, module, &vc_name, identified)?;
    c.definitions.insert(vc_name.clone(), value);
    let placeholder = object_name("application placeholder");
    c.parents.push((placeholder.clone(), root_name.clone()));
    let value = Value::New(Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSApplication"))]).named(placeholder.clone()));
    c.definitions.insert(placeholder, value);
    let storyboard = format!("{vc_name} storyboard");
    storyboard_placeholder(&mut nib, &mut c, &storyboard, &root_name);
    let label = nib.string("sceneController");
    let mut connections = vec![outlet(format!("{vc_name} scene controller"), &root_name, &vc_name, label)];
    add_templates(&mut nib, &mut c, &mut connections, &templates, &vc_name, &storyboard);
    let label = nib.string("storyboard");
    connections.push(outlet(format!("{vc_name} storyboard outlet"), &vc_name, &storyboard, label));
    assemble(&mut nib, root_object, &root_name, c, connections)
}

/// A view controller's view nib: an NSObject root (the controller) whose
/// `view` outlet is the view.
fn view_nib(doc: &Document, vc: Node, first_responder: &str, module: Option<&str>, localizable: bool) -> Result<(String, Vec<u8>), String> {
    only_children(vc, &["view", "sceneView", "connections"])?;
    // The controller's own connections, and those to it, go through an
    // upstream placeholder standing for it (Xcode's SpriteKit template).
    let upstream = uses_upstream(vc);
    if children(vc, "connections").flat_map(|l| l.children().filter(|c| c.is_element())).any(|c| !c.has_tag_name("outlet")) {
        return fail("a view controller's connections other than outlets are not supported by this ibtool");
    }
    let mut nib = new_nib(doc, localizable, true);
    let view_node = controller_view(vc).ok_or("a view controller without a view")?;
    let root_name = object_name("root");
    let root_object = Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSObject"))]).named(root_name.clone());
    let mut c = new_collected(vec![attr(vc, "id")?.to_string()]);
    let view_name = object_name(attr(view_node, "id")?);
    let value = if view_node.has_tag_name("sceneView") {
        scene_view(&mut nib, &mut c, view_node, module, &root_name)?
    } else {
        view(&mut nib, &mut c, view_node, module, &root_name, None)?
    };
    c.definitions.insert(view_name.clone(), value);
    let placeholder = object_name("application placeholder");
    c.parents.push((placeholder.clone(), root_name.clone()));
    let value = Value::New(Object::new("NSCustomObject", vec![("NSClassName", nib.string("NSApplication"))]).named(placeholder.clone()));
    c.definitions.insert(placeholder, value);
    // Upstream placeholders: the controller's, then each embed segue's
    // template, numbered from 1, after the Application placeholder; each
    // template's container view is its outlet (four probes).
    let mut placeholders = Vec::new();
    if upstream {
        placeholders.push((object_name(attr(vc, "id")?), None));
    }
    for segue in embeds(vc) {
        let container = segue.parent().and_then(|l| l.parent()).ok_or("an embed segue outside a container view")?;
        placeholders.push((object_name(&format!("{} template", attr(segue, "id")?)), Some(object_name(attr(container, "id")?))));
    }
    let mut embed_outlets = Vec::new();
    for (i, (name, container)) in placeholders.into_iter().enumerate() {
        c.parents.push((name.clone(), root_name.clone()));
        let identifier = nib.string(&format!("UpstreamPlaceholder-{}", i + 1));
        c.definitions.insert(
            name.clone(),
            Value::New(Object::new("NSNibExternalObjectPlaceholder", vec![("NSExternalObjectPlaceholderIdentifier", identifier)]).named(name.clone())),
        );
        if let Some(container) = container {
            let label = nib.string("containerView");
            embed_outlets.push(outlet(format!("{name} container outlet"), &name, &container, label));
        }
    }
    let label = nib.string("view");
    let mut connections = vec![outlet(format!("{view_name} view outlet"), &root_name, &view_name, label)];
    connections.extend(embed_outlets);
    connections.extend(self::connections(&mut nib, doc, &c.connection_sources.clone(), first_responder)?);
    let name = format!("{}-view-{}", attr(vc, "id")?, attr(view_node, "id")?);
    Ok((name, assemble(&mut nib, root_object, &root_name, c, connections)?))
}

/// The ICC profile of the Mac's Generic Gray Gamma 2.2 or sRGB colour space,
/// which AppKit's colours embed: copied into the SDK archive's toolchain
/// from the Mac that packaged it (tools/package-sdks.sh), as actool reads it.
fn profile(gray: bool) -> Result<Vec<u8>, String> {
    let developer = std::env::var("DEVELOPER_DIR").unwrap_or_else(|_| "/Applications/Xcode.app/Contents/Developer".into());
    let name = if gray { "generic_gray_gamma_2_2.icc" } else { "srgb.icc" };
    let path = std::path::Path::new(&developer).join("Toolchains/XcodeDefault.xctoolchain/usr/share/colorsync").join(name);
    std::fs::read(&path).map_err(|e| format!("{}: {e} (an SDK archive packaged before it kept the Mac's colour profiles)", path.display()))
}

/// A colour. A System catalog colour names itself and carries the colour
/// it resolves to on the Mac, in Generic Gray Gamma 2.2 or sRGB with the
/// profile embedded (the corpus's 15 System colours, each always the
/// same).
fn color(nib: &mut Nib, n: Node) -> Result<Value, String> {
    // Calibrated and device white and RGB: their components as `%.10g`
    // text, alpha only when not 1 (eight probes).
    let space = match n.attribute("colorSpace") {
        Some("calibratedWhite") => Some((3, true)),
        Some("deviceWhite") => Some((4, true)),
        Some("calibratedRGB") => Some((1, false)),
        Some("deviceRGB") => Some((2, false)),
        _ => None,
    };
    if let Some((space, white)) = space {
        let keys: &[&str] = if white { &["white"] } else { &["red", "green", "blue"] };
        let mut allowed: Vec<(&str, Option<&str>)> = vec![("key", None), ("alpha", None), ("colorSpace", None)];
        allowed.extend(keys.iter().map(|k| (*k, None)));
        only_attributes(n, &allowed)?;
        let mut text: Vec<String> = keys.iter().map(|k| Ok(general(number(attr(n, k)?)?, 10))).collect::<Result<_, String>>()?;
        let alpha = number(attr(n, "alpha")?)?;
        if alpha != 1.0 {
            text.push(general(alpha, 10));
        }
        let mut bytes = text.join(" ").into_bytes();
        bytes.push(0);
        let key = if white { "NSWhite" } else { "NSRGB" };
        return Ok(nib.shared(format!("color:{space}:{}", String::from_utf8_lossy(&bytes)), || {
            Object::new("NSColor", vec![("NSColorSpace", Value::Int(space)), (key, Value::Data(bytes))])
        }));
    }
    if n.attribute("colorSpace") == Some("custom") && n.attribute("customColorSpace") == Some("sRGB") {
        return srgb_color(nib, n);
    }
    // Gamma 2.2 grey and Display P3 also archive `NSLinearExposure`, which
    // for grey follows no rule found yet (60 probes).
    if n.attribute("colorSpace") == Some("custom") {
        return fail(format!(
            "a colour in the {} colour space is not supported by this ibtool",
            n.attribute("customColorSpace").unwrap_or("custom")
        ));
    }
    only_attributes(n, &[("key", None), ("name", None), ("catalog", Some("System")), ("colorSpace", Some("catalog"))])?;
    system_color(nib, attr(n, "name")?)
}

/// `x` as C's `%.{digits}g` prints it: that many significant digits,
/// trailing zeros dropped, in exponent form below 1e-4 or from 1e{digits}.
fn general(x: f64, digits: i32) -> String {
    if x == 0.0 {
        return "0".into();
    }
    let sci = format!("{x:.*e}", (digits - 1) as usize);
    let (mantissa, exponent) = sci.split_once('e').expect("an exponent");
    let exponent: i32 = exponent.parse().expect("an exponent");
    let trim = |s: String| if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    if exponent < -4 || exponent >= digits {
        let sign = if exponent < 0 { '-' } else { '+' };
        format!("{}e{sign}{:02}", trim(mantissa.to_string()), exponent.abs())
    } else {
        trim(format!("{x:.*}", (digits - 1 - exponent) as usize))
    }
}

/// Generic Gray Gamma 2.2's or sRGB's colour space, its profile embedded,
/// one object in a nib.
fn color_space(nib: &mut Nib, gray: bool) -> Result<Value, String> {
    if let Some(existing) = nib.shared.get(&format!("colorspace:{gray}")) {
        return Ok(reference(existing.clone()));
    }
    let icc = profile(gray)?;
    let data = Value::New(Object::new("NSData", vec![("NS.bytes", Value::Data(icc))]));
    let values = if gray {
        vec![("NSID", Value::Int(9)), ("NSModel", Value::Int(0)), ("NSICC", data)]
    } else {
        vec![("NSID", Value::Int(7)), ("NSICC", data)]
    };
    Ok(nib.shared(format!("colorspace:{gray}"), || Object::new("NSColorSpace", values)))
}

/// A custom sRGB colour (about 60 probes): its components converted to
/// calibrated RGB as `%.10g` text (alpha when not 1), the sRGB colour space,
/// its own components (alpha always), and `NSLinearExposure`: `1` when all
/// four components are multiples of 1/255, as Interface Builder's colour
/// picker makes them, `0` otherwise.
fn srgb_color(nib: &mut Nib, n: Node) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("key", None),
            ("red", None),
            ("green", None),
            ("blue", None),
            ("alpha", None),
            ("colorSpace", Some("custom")),
            ("customColorSpace", Some("sRGB")),
        ],
    )?;
    let c = [number(attr(n, "red")?)?, number(attr(n, "green")?)?, number(attr(n, "blue")?)?, number(attr(n, "alpha")?)?];
    let rgb = crate::colorsync::srgb_to_generic([c[0], c[1], c[2]])?;
    let mut text: Vec<String> = rgb.iter().map(|&x| general(x, 10)).collect();
    if c[3] != 1.0 {
        text.push(general(c[3], 10));
    }
    let mut rgb_bytes = text.join(" ").into_bytes();
    rgb_bytes.push(0);
    let components: Vec<String> = c.iter().map(|&x| general(x, 10)).collect();
    let eight_bit = c.iter().all(|&x| ((x * 255.0).round() / 255.0) as f32 == x as f32);
    let space = color_space(nib, false)?;
    let key = format!("color:srgb:{c:?}");
    let values = vec![
        ("NSColorSpace", Value::Int(1)),
        ("NSRGB", Value::Data(rgb_bytes)),
        ("NSCustomColorSpace", space),
        ("NSComponents", Value::Data(components.join(" ").into_bytes())),
        ("NSLinearExposure", Value::Data(if eight_bit { b"1".to_vec() } else { b"0".to_vec() })),
    ];
    Ok(nib.shared(key, || Object::new("NSColor", values)))
}

/// A System catalog colour by name.
fn system_color(nib: &mut Nib, name: &str) -> Result<Value, String> {
    let (white, components): (Option<&str>, &str) = match name {
        "textBackgroundColor" | "headerColor" => (Some("1"), "1 1"),
        "labelColor" | "controlTextColor" | "headerTextColor" | "selectedTextColor" | "textColor" => (Some("0"), "0 1"),
        "controlBackgroundColor" | "controlColor" | "selectedTextBackgroundColor" => (Some("0.602715373"), "0.6666666667 1"),
        "windowBackgroundColor" | "gridColor" => (Some("0.4246723652"), "0.5 1"),
        _ => return fail(format!("the System colour {name} is not supported by this ibtool")),
    };
    let key = format!("color:{name}");
    if let Some(existing) = nib.shared.get(&key) {
        return Ok(reference(existing.clone()));
    }
    let gray = white.is_some();
    let space = color_space(nib, gray)?;
    let nul = |s: &str| {
        let mut b = s.as_bytes().to_vec();
        b.push(0);
        Value::Data(b)
    };
    // Colours resolving to the same one share it (a corpus xib's label and
    // control text colours).
    let fallback = nib.shared(format!("resolved:{white:?}:{components}"), || {
        Object::new(
            "NSColor",
            vec![
                ("NSColorSpace", Value::Int(3)),
                ("NSWhite", nul(white.unwrap_or_default())),
                ("NSCustomColorSpace", space),
                ("NSComponents", Value::Data(components.as_bytes().to_vec())),
                ("NSLinearExposure", Value::Data(b"1".to_vec())),
            ],
        )
    });
    let catalog = nib.string("System");
    let color_name = nib.string(name);
    Ok(nib.shared(key, || {
        Object::new(
            "NSColor",
            vec![("NSColorSpace", Value::Int(6)), ("NSCatalogName", catalog), ("NSColorName", color_name), ("NSColor", fallback)],
        )
    }))
}

/// A font: a system font by its metafont (AppKit's private names, its
/// default size and `NSfFlags`, 26 probes), or a font by name (flags 16).
/// A control's font, or with `usesAppearanceFont` the system font: a
/// button's of its control size (`button`: 13, 11 or 9), a text field's at
/// 13 whatever its size, sized by the appearance
/// (`NSFontUsesAppearanceFontSize`; six probes).
fn appearance_font(nib: &mut Nib, n: Node, button: Option<f64>) -> Result<Value, String> {
    if n.attribute("usesAppearanceFont").is_none() {
        return font(nib, n);
    }
    only_attributes(n, &[("key", None), ("usesAppearanceFont", Some("YES"))])?;
    let name = nib.string(".AppleSystemUIFont");
    Ok(match button {
        Some(size) => nib.shared(format!("font:.AppleSystemUIFont:{size}:1044"), || {
            Object::new("NSFont", vec![("NSName", name), ("NSSize", Value::Double(size)), ("NSfFlags", Value::Int(0x414))])
        }),
        None => nib.shared("font:appearance".to_string(), || {
            Object::new(
                "NSFont",
                vec![
                    ("NSName", name),
                    ("NSFontUsesAppearanceFontSize", Value::Bool(true)),
                    ("NSSize", Value::Double(13.0)),
                    ("NSfFlags", Value::Int(0x414)),
                ],
            )
        }),
    })
}

fn font(nib: &mut Nib, n: Node) -> Result<Value, String> {
    font_as(nib, n, None)
}

/// A font, with a metafont read as another (`meta`: a pop-up button's menu
/// font is the message font, two probes).
fn font_as(nib: &mut Nib, n: Node, meta: Option<(&str, &str)>) -> Result<Value, String> {
    only_attributes(n, &[("key", None), ("metaFont", None), ("size", None), ("name", None), ("family", None)])?;
    let size = n.attribute("size").map(number).transpose()?;
    let metafont = n.attribute("metaFont").map(|m| match meta {
        Some((from, to)) if m == from => to,
        _ => m,
    });
    let (name, default, flags): (&str, f64, i64) = match (metafont, n.attribute("name")) {
        (Some(meta), None) => match meta {
            "system" => (".AppleSystemUIFont", 13.0, 0x414),
            "systemBold" => (".AppleSystemUIFontBold", 13.0, 0x818),
            "smallSystem" => (".AppleSystemUIFont", 11.0, 0xc1c),
            "smallSystemBold" => (".AppleSystemUIFontBold", 11.0, 0xd1d),
            "miniSystem" => (".AppleSystemUIFont", 9.0, 0xe1e),
            "label" => (".AppleSystemUIFont", 10.0, 0xb1b),
            "menu" => (".AppleSystemUIFont", 13.0, 0x515),
            "message" => (".AppleSystemUIFont", 13.0, 0x616),
            "palette" => (".AppleSystemUIFont", 11.0, 0x717),
            "toolTip" => (".AppleSystemUIFont", 11.0, 0x919),
            "controlContent" => (".AppleSystemUIFont", 12.0, 0xa1a),
            "menuBar" => (".AppleSystemUIFont", 14.0, 0x1010),
            "cellTitle" => (".AppleSystemUIFont", 12.0, 0x1313),
            "systemUltraLight" => (".AppleSystemUIFontUltraLight", 13.0, 0x1414),
            "systemThin" => (".AppleSystemUIFontThin", 13.0, 0x1514),
            "systemLight" => (".AppleSystemUIFontLight", 13.0, 0x1614),
            "systemMedium" => (".AppleSystemUIFontMedium", 13.0, 0x1714),
            "systemSemibold" => (".AppleSystemUIFontDemi", 13.0, 0x1814),
            "systemHeavy" => (".AppleSystemUIFontHeavy", 13.0, 0x1918),
            "systemBlack" => (".AppleSystemUIFontBlack", 13.0, 0x1a18),
            other => return fail(format!("metaFont {other} is not supported by this ibtool")),
        },
        (None, Some(name)) => (name, size.ok_or("a named font without a size")?, 16),
        _ => return fail("a font with both a metafont and a name, or neither"),
    };
    let size = size.unwrap_or(default);
    let font_name = nib.string(name);
    Ok(nib.shared(format!("font:{name}:{size}:{flags}"), || {
        Object::new("NSFont", vec![("NSName", font_name), ("NSSize", Value::Double(size)), ("NSfFlags", Value::Int(flags))])
    }))
}

/// Refuses a control Auto Layout sizes by its content, when constraints
/// place it: Xcode lays such views out again as it compiles, with the Mac's
/// control metrics and text measurement, so a document whose frames are
/// older than those differs (133 of 227 text fields' and buttons' frames in
/// the corpus). `--config=relax_ibtool_layout` keeps the document's frames,
/// which only the first layout at run time uses.
fn content_sized(nib: &Nib, n: Node) -> Result<(), String> {
    let constrained = n.attribute("translatesAutoresizingMaskIntoConstraints").is_some() && n.attribute("id").is_some_and(|id| nib.constrained.contains(id));
    let relaxed = layout_relaxed();
    if constrained && !relaxed {
        return fail(format!(
            "a constrained <{}> is laid out again by Xcode's Auto Layout as it compiles, which this ibtool doesn't reproduce; --config=relax_ibtool_layout compiles it with the document's frame",
            n.tag_name().name()
        ));
    }
    Ok(())
}

/// A text field and its cell (28 probes of the cell's attributes): the
/// cell's flags and the control's alignment, line breaking and size.
fn text_field(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("id", None),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
            ("horizontalHuggingPriority", None),
            ("verticalHuggingPriority", None),
            ("horizontalCompressionResistancePriority", None),
            ("verticalCompressionResistancePriority", None),
            ("translatesAutoresizingMaskIntoConstraints", Some("NO")),
            ("fixedFrame", Some("YES")),
            ("allowsCharacterPickerTouchBarItem", None),
            ("preferredMaxLayoutWidth", None),
        ],
    )?;
    only_children(n, &["rect", "autoresizingMask", "textFieldCell", "constraints", "point", "accessibility"])?;
    canvas_location_only(n)?;
    let id = attr(n, "id")?;
    content_sized(nib, n)?;
    let name = object_name(id);
    c.parents.push((name.clone(), parent.to_string()));
    // Its cell comes before its own constraints in the objects' list (a
    // corpus xib).
    let cell = child(n, "textFieldCell", "cell").ok_or("a text field without a cell")?;
    c.parents.push((object_name(attr(cell, "id")?), name.clone()));
    let mut values = view_values(nib, c, n, module, &name, parent, superview(n, parent))?;
    only_attributes(
        cell,
        &[
            ("key", None),
            ("id", None),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
            ("title", None),
            ("placeholderString", None),
            ("lineBreakMode", None),
            ("alignment", None),
            ("scrollable", Some("YES")),
            ("sendsActionOnEndEditing", Some("YES")),
            ("editable", Some("YES")),
            ("selectable", Some("YES")),
            ("drawsBackground", Some("YES")),
            ("borderStyle", Some("bezel")),
            ("controlSize", None),
            ("state", Some("on")),
            ("focusRingType", Some("none")),
            ("allowsUndo", Some("NO")),
            ("truncatesLastVisibleLine", Some("YES")),
            ("usesSingleLineMode", Some("YES")),
        ],
    )?;
    only_children(cell, &["font", "color", "string"])?;
    let cell_id = attr(cell, "id")?;
    let cell_name = object_name(cell_id);
    let has = |a: &str| cell.attribute(a).is_some();
    let mut flags: i64 = 0x0400_0000;
    let mut flags2: i64 = 0;
    // Without a line break mode, word wrapping (a probe).
    let line_break = cell.attribute("lineBreakMode").unwrap_or("wordWrapping");
    // NSLineBreakMode, in flags2's bits 9 to 11 too.
    let control_break = match line_break {
        "wordWrapping" => 0,
        "charWrapping" => 1,
        "clipping" => 2,
        "truncatingHead" => 3,
        "truncatingTail" => 4,
        "truncatingMiddle" => 5,
        other => return fail(format!("line break mode {other} is not supported by this ibtool")),
    };
    flags2 |= control_break << 9;
    if !line_break.ends_with("Wrapping") {
        flags |= 0x40;
    }
    let (control_alignment, alignment_bits) = match cell.attribute("alignment") {
        None | Some("natural") => (4, 4),
        Some("left") => (0, 0),
        Some("center") => (1, 2),
        Some("right") => (2, 1),
        Some("justified") => (3, 3),
        Some(other) => return fail(format!("alignment {other} is not supported by this ibtool")),
    };
    flags2 |= alignment_bits << 26;
    for (a, bit) in [("scrollable", 0x10_0000), ("editable", 0x1020_0001), ("selectable", 0x20_0001), ("borderStyle", 0x40_0000)] {
        if has(a) {
            flags |= bit;
        }
    }
    if has("state") {
        flags |= 0x8000_0000;
    }
    for (a, bit) in [
        ("sendsActionOnEndEditing", 0x40_0000),
        ("focusRingType", 0x8000),
        ("allowsUndo", 0x1000),
        ("truncatesLastVisibleLine", 0x100),
        ("usesSingleLineMode", 0x40),
    ] {
        if has(a) {
            flags2 |= bit;
        }
    }
    let control_size = match cell.attribute("controlSize") {
        None | Some("regular") => 0,
        Some("small") => {
            flags2 |= 0x2_0000;
            1
        }
        Some("mini") => {
            flags2 |= 0x4_0000;
            2
        }
        Some(other) => return fail(format!("control size {other} is not supported by this ibtool")),
    };
    // A set top bit is a negative int.
    let signed = |v: i64| v as u32 as i32 as i64;
    // A cell of a custom class is swapped in for NSTextFieldCell (Xcode's
    // action extension template).
    let mut cell_values = Vec::new();
    if cell.attribute("customClass").is_some() {
        cell_values.push(("NSClassName", nib.string(&runtime_class_name(cell, module, "NSTextFieldCell")?)));
        cell_values.push(("NSOriginalClassName", nib.string("NSTextFieldCell")));
    }
    cell_values.push(("NSCellFlags", Value::Int(signed(flags))));
    cell_values.push(("NSCellFlags2", Value::Int(signed(flags2))));
    cell_values.push(("NSContents", nib.text(cell_id, "title", &title(cell)?)));
    let font_node = child(cell, "font", "font").ok_or("a text field cell without a font")?;
    cell_values.push(("NSSupport", appearance_font(nib, font_node, None)?));
    // Its placeholder, after its font (a probe).
    if let Some(p) = cell.attribute("placeholderString") {
        cell_values.push(("NSPlaceholderString", nib.text(cell_id, "placeholderString", p)));
    }
    cell_values.push(("NSControlView", reference(name.clone())));
    if has("drawsBackground") {
        cell_values.push(("NSDrawsBackground", Value::Bool(true)));
    }
    let bg = child(cell, "color", "backgroundColor").ok_or("a text field cell without a background colour")?;
    cell_values.push(("NSBackgroundColor", color(nib, bg)?));
    let fg = child(cell, "color", "textColor").ok_or("a text field cell without a text colour")?;
    cell_values.push(("NSTextColor", color(nib, fg)?));
    values.push(("NSEnabled", Value::Bool(true)));
    // The Touch Bar's character picker when allowed, last; not allowing it
    // changes nothing (two probes).
    match n.attribute("allowsCharacterPickerTouchBarItem") {
        None | Some("NO") => {}
        Some("YES") => cell_values.push(("NSCharacterPickerEnabled", Value::Bool(true))),
        Some(other) => return fail(format!("allowsCharacterPickerTouchBarItem=\"{other}\" is not supported by this ibtool")),
    }
    let cell_class = if cell.attribute("customClass").is_some() { "NSClassSwapper" } else { "NSTextFieldCell" };
    values.push(("NSCell", Value::New(Object::new(cell_class, cell_values).named(cell_name))));
    // A localizable document's controls allow a logical (right-to-left)
    // layout direction (the corpus's 358 controls, exactly those in
    // `Base.lproj`).
    values.push(("NSAllowsLogicalLayoutDirection", Value::Bool(nib.localizable)));
    values.push(("NSControlSize", Value::Int(control_size)));
    values.push(("NSControlContinuous", Value::Bool(false)));
    values.push(("NSControlRefusesFirstResponder", Value::Bool(false)));
    values.push(("NSControlUsesSingleLineMode", Value::Bool(has("usesSingleLineMode"))));
    values.push(("NSControlTextAlignment", Value::Int(control_alignment)));
    values.push(("NSControlLineBreakMode", Value::Int(control_break)));
    values.push(("NSControlWritingDirection", Value::Int(-1)));
    values.push(("NSControlSendActionMask", Value::Int(4)));
    // Its preferred width for wrapping, when given (Xcode's document app
    // template).
    if let Some(w) = n.attribute("preferredMaxLayoutWidth") {
        values.push(("NSPreferredMaxLayoutWidth", Value::Double(number(w)?)));
    }
    values.push(("NSTextFieldAlignmentRectInsetsVersion", Value::Int(2)));
    values.push(("NSAllowsWritingTools", Value::Bool(true)));
    values.push(("NSTextFieldAllowsWritingToolsAffordance", Value::Bool(false)));
    values.push(("NS.resolvesNaturalAlignmentWithBaseWritingDirection", Value::Bool(false)));
    // A storyboard's text field that isn't editable has the standard line
    // break strategy, after its alignment rect insets' version (the
    // corpus's 94 storyboard text fields; a xib's never has).
    if nib.storyboard && !has("editable") {
        let at = values.iter().position(|v| v.0 == "NSTextFieldAlignmentRectInsetsVersion").expect("the insets' version") + 1;
        values.insert(at, ("NSTextFieldLineBreakStrategyVersion", Value::Int(2)));
        values.insert(at + 1, ("NSLineBreakStrategy", Value::Int(0xffff)));
    }
    swapped(nib, n, module, "NSTextField", values, name)
}

/// A button and its cell (85 probes): the cell's flags as a text field's,
/// `NSButtonFlags` (its behaviour, image position in bits 19 to 22,
/// border 0x800000, inset from bit 13, transparency 0x8000, 0x100 for a
/// check box's or radio button's image), `NSButtonFlags2` (the bezel style,
/// image scaling in bits 6 and 7, the key equivalent's modifiers from bit
/// 8), and the button type its behaviour makes. A button's `type` itself
/// changes nothing.
fn button(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("id", None),
            ("hidden", Some("YES")),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
            ("horizontalHuggingPriority", None),
            ("verticalHuggingPriority", None),
            ("horizontalCompressionResistancePriority", None),
            ("verticalCompressionResistancePriority", None),
            ("translatesAutoresizingMaskIntoConstraints", Some("NO")),
            ("fixedFrame", Some("YES")),
        ],
    )?;
    only_children(
        n,
        &["rect", "autoresizingMask", "buttonCell", "popUpButtonCell", "constraints", "connections", "point", "accessibility"],
    )?;
    canvas_location_only(n)?;
    content_sized(nib, n)?;
    // A pop-up button is a button whose cell holds a menu (about 10
    // probes).
    let popup = n.has_tag_name("popUpButton");
    let id = attr(n, "id")?;
    let name = object_name(id);
    c.parents.push((name.clone(), parent.to_string()));
    if children(n, "connections").next().is_some() {
        c.connection_sources.push(id.to_string());
    }
    let cell = child(n, if popup { "popUpButtonCell" } else { "buttonCell" }, "cell").ok_or("a button without a cell")?;
    // Its cell comes before its own constraints in the objects' list (a
    // probe).
    c.parents.push((object_name(attr(cell, "id")?), name.clone()));
    only_attributes(
        cell,
        &[
            ("key", None),
            ("id", None),
            ("type", None),
            ("bezelStyle", None),
            ("title", None),
            ("alternateTitle", None),
            ("alignment", None),
            ("borderStyle", Some(if popup { "borderAndBezel" } else { "border" })),
            ("imagePosition", None),
            ("imageScaling", None),
            ("inset", None),
            ("state", Some("on")),
            ("controlSize", None),
            ("keyEquivalent", None),
            ("lineBreakMode", None),
            ("enabled", Some("NO")),
            ("selectedItem", None),
            ("arrowPosition", None),
            ("image", None),
        ],
    )?;
    if !popup && ["enabled", "selectedItem", "arrowPosition"].iter().any(|a| cell.attribute(*a).is_some()) {
        return fail("a button cell's enabled, selectedItem or arrowPosition attribute is not supported by this ibtool");
    }
    only_children(cell, &["behavior", "font", "modifierMask", "string", "menu", "connections"])?;
    if children(cell, "connections").next().is_some() {
        c.connection_sources.push(attr(cell, "id")?.to_string());
    }
    if popup != child(cell, "menu", "menu").is_some() {
        return fail("a pop-up button without a menu, or a button with one, is not supported by this ibtool");
    }
    let cell_id = attr(cell, "id")?;
    let cell_name = object_name(cell_id);
    let has = |a: &str| cell.attribute(a).is_some();
    let bezel_name = cell.attribute("bezelStyle").ok_or("a button without a bezel style")?;
    let (bezel, bezel_bits) = match bezel_name {
        "rounded" => (1, 1),
        "regularSquare" => (2, 2),
        "disclosure" => (5, 5),
        "shadowlessSquare" => (6, 6),
        "circular" => (7, 7),
        "texturedSquare" => (8, 0x20),
        "helpButton" => (9, 0x21),
        "smallSquare" => (10, 0x22),
        "texturedRounded" => (11, 0x23),
        "recessed" => (13, 0x2d),
        "roundedDisclosure" => (14, 0x26),
        // An inline button's font is its own (a probe), not established.
        other => return fail(format!("bezel style {other} is not supported by this ibtool")),
    };
    // Its behaviour's bits, and the button type it makes.
    let behavior = child(cell, "behavior", "behavior").ok_or("a button without a behaviour")?;
    let mut behaviors: Vec<&str> = behavior.attributes().filter(|a| a.name() != "key").map(|a| a.name()).collect();
    behaviors.sort();
    if behavior.attributes().any(|a| a.name() != "key" && a.value() != "YES") {
        return fail("a button behaviour other than YES is not supported by this ibtool");
    }
    let mut flags_b: i64 = 0;
    for b in &behaviors {
        flags_b |= match *b {
            "pushIn" => 0x8000_0000,
            "changeContents" => 0x4000_0000,
            "changeBackground" => 0x2000_0000,
            "changeGray" => 0x1000_0000,
            "lightByContents" => 0x0800_0000,
            "lightByBackground" => 0x0400_0000,
            "lightByGray" => 0x0200_0000,
            "doesNotDimImage" => 0x1000,
            other => return fail(format!("button behaviour {other} is not supported by this ibtool")),
        };
    }
    let kind = cell.attribute("type").unwrap_or("push");
    let (aux, image_source) = match (behaviors.as_slice(), kind) {
        (["lightByBackground", "lightByGray", "pushIn"], _) => (7, None),
        (["lightByBackground", "lightByGray"], "push" | "bevel") if popup => (0, None),
        // AppKit's button types for other behaviours, whatever the type
        // (six probes).
        (["lightByContents"], _) if !popup => (5, None),
        (["changeBackground", "changeGray", "lightByBackground", "lightByGray"], _) if !popup => (6, None),
        (["changeBackground", "changeGray", "lightByBackground", "lightByGray", "pushIn"], _) if !popup => (1, None),
        (["changeContents", "lightByContents"] | ["lightByBackground", "lightByGray"] | ["lightByContents", "pushIn"], kind) if !popup && kind != "check" && kind != "radio" => (0, None),
        (["changeContents", "doesNotDimImage", "lightByContents"], "check") => (3, Some("NSSwitch")),
        (["changeContents", "doesNotDimImage", "lightByContents"], "radio") => (4, Some("NSRadioButton")),
        _ => return fail(format!("a {kind} button behaving as {} is not supported by this ibtool", behaviors.join(" "))),
    };
    let position = match cell.attribute("imagePosition") {
        None => 0,
        Some("above") => 1,
        Some("below") => 3,
        Some("right" | "trailing") => 5,
        Some("left" | "leading") => 7,
        Some("only") => 8,
        Some("overlaps") => 9,
        Some(other) => return fail(format!("image position {other} is not supported by this ibtool")),
    };
    flags_b |= position << 19;
    if has("borderStyle") {
        flags_b |= 0x80_0000;
    }
    if let Some(inset) = cell.attribute("inset") {
        flags_b |= inset.parse::<i64>().map_err(|_| format!("not an inset: {inset}"))? << 13;
    }
    if image_source.is_some() {
        flags_b |= 0x100;
    }
    let scaling = match cell.attribute("imageScaling") {
        None | Some("none") => 0,
        Some("proportionallyUpOrDown") => 1,
        Some("proportionallyDown") => 2,
        Some("axesIndependently") => 3,
        Some(other) => return fail(format!("image scaling {other} is not supported by this ibtool")),
    };
    let key_equivalent = match (cell.attribute("keyEquivalent"), children(cell, "string").find(|s| s.attribute("key") == Some("keyEquivalent"))) {
        (Some(k), None) => k.to_string(),
        (None, Some(e)) => match e.attribute("base64-UTF8") {
            None => e.text().unwrap_or("").to_string(),
            // Characters XML can't hold (escape, a corpus xib's Cancel).
            Some("YES") => cf::xml::decode_base64(e.text().unwrap_or(""))
                .and_then(|b| String::from_utf8(b).ok())
                .ok_or("a key equivalent that isn't base64 of UTF-8")?,
            Some(other) => return fail(format!("base64-UTF8=\"{other}\" is not supported by this ibtool")),
        },
        (None, None) => String::new(),
        _ => return fail("a key equivalent both as an attribute and an element"),
    };
    // A button's key equivalent has no modifiers without a mask.
    let modifiers = match child(cell, "modifierMask", "keyEquivalentModifierMask") {
        Some(_) => modifier_mask(cell)?,
        None => 0,
    };
    let flags_b2 = bezel_bits | scaling << 6 | modifiers << 8;

    // The cell's flags, as a text field's.
    let mut flags: i64 = 0x0400_0000;
    let mut flags2: i64 = 0;
    let line_break = match cell.attribute("lineBreakMode") {
        None | Some("wordWrapping") => 0,
        Some("charWrapping") => 1,
        Some("clipping") => 2,
        Some("truncatingHead") => 3,
        Some("truncatingTail") => 4,
        Some("truncatingMiddle") => 5,
        Some(other) => return fail(format!("line break mode {other} is not supported by this ibtool")),
    };
    if line_break >= 2 {
        flags |= 0x40;
    }
    flags2 |= line_break << 9;
    let (control_alignment, alignment_bits) = match cell.attribute("alignment") {
        None | Some("natural") => (4, 4),
        Some("left") => (0, 0),
        Some("center") => (1, 2),
        Some("right") => (2, 1),
        Some("justified") => (3, 3),
        Some(other) => return fail(format!("alignment {other} is not supported by this ibtool")),
    };
    flags2 |= alignment_bits << 26;
    if has("state") {
        flags |= 0x8000_0000;
    }
    // A pop-up button's border and bezel, and its cell disabled.
    if popup && has("borderStyle") {
        flags |= 0x40_0000;
    }
    if has("enabled") {
        flags |= 0x2000_0000;
    }
    let control_size = match cell.attribute("controlSize") {
        None | Some("regular") => 0,
        Some("small") => {
            flags2 |= 0x2_0000;
            1
        }
        Some("mini") => {
            flags2 |= 0x4_0000;
            2
        }
        Some(other) => return fail(format!("control size {other} is not supported by this ibtool")),
    };
    let signed = |v: i64| v as u32 as i32 as i64;
    let font_node = child(cell, "font", "font").ok_or("a button cell without a font")?;
    let mut cell_values = vec![
        ("NSCellFlags", Value::Int(signed(flags))),
        ("NSCellFlags2", Value::Int(signed(flags2))),
        // A pop-up button's title is its selected item's, not localized
        // itself (a xib and a storyboard in Base.lproj).
        (
            "NSContents",
            if popup {
                nib.string(cell.attribute("title").unwrap_or(""))
            } else {
                nib.text(cell_id, "title", cell.attribute("title").unwrap_or(""))
            },
        ),
        (
            "NSSupport",
            if popup && font_node.attribute("usesAppearanceFont").is_none() {
                font_as(nib, font_node, Some(("menu", "message")))?
            } else {
                appearance_font(nib, font_node, Some([13.0, 11.0, 9.0][control_size as usize]))?
            },
        ),
        ("NSControlView", reference(name.clone())),
        ("NSButtonFlags", Value::Int(signed(flags_b))),
        ("NSButtonFlags2", Value::Int(signed(flags_b2))),
        ("NSBezelStyle", Value::Int(bezel)),
    ];
    // Its image, before its alternate's (three probes).
    if let Some(image) = cell.attribute("image") {
        if image_source.is_some() {
            return fail("a check box's or radio button's image is not supported by this ibtool");
        }
        cell_values.push(("NSNormalImage", catalog_image(nib, cell, image)?));
    }
    if let Some(image) = image_source {
        let image_name = nib.string(image);
        cell_values.push(("NSAlternateImage", Value::New(Object::new("NSButtonImageSource", vec![("NSImageName", image_name)]))));
    }
    cell_values.push(("NSAlternateContents", nib.text(cell_id, "alternateTitle", cell.attribute("alternateTitle").unwrap_or(""))));
    cell_values.push(("NSKeyEquivalent", nib.string(&key_equivalent)));
    cell_values.push(("NSPeriodicDelay", Value::Int(400)));
    cell_values.push(("NSPeriodicInterval", Value::Int(75)));
    cell_values.push(("NSAuxButtonType", Value::Int(aux)));
    if popup {
        let menu_node = child(cell, "menu", "menu").expect("checked");
        let (menu, selected) = popup_menu(nib, c, menu_node, &cell_name, cell.attribute("selectedItem"))?;
        let index = selected.as_ref().map_or(-1, |s| s.1 as i64);
        cell_values.push(("NSMenuItem", selected.map_or(Value::Nil, |s| s.0)));
        cell_values.push(("NSMenuItemRespectAlignment", Value::Bool(true)));
        cell_values.push(("NSMenu", menu));
        // The selected item's index unless the first.
        if index != 0 {
            cell_values.push(("NSSelectedIndex", Value::Int(index)));
        }
        cell_values.push(("NSPreferredEdge", Value::Int(1)));
        cell_values.push(("NSUsesItemFromMenu", Value::Bool(true)));
        cell_values.push(("NSAltersState", Value::Bool(true)));
        let arrow = match cell.attribute("arrowPosition") {
            None | Some("arrowAtBottom") => 2,
            Some("arrowAtCenter") => 1,
            Some(other) => return fail(format!("arrow position {other} is not supported by this ibtool")),
        };
        cell_values.push(("NSArrowPosition", Value::Int(arrow)));
    }

    // Content hugging: a bordered button of a rounded kind has its own, 250
    // and 750, and archives any other, even 250 each (probes); square and
    // borderless kinds as a view does.
    let square = matches!(bezel_name, "regularSquare" | "shadowlessSquare" | "smallSquare" | "texturedSquare");
    let mut values = view_values(nib, c, n, module, &name, parent, superview(n, parent))?;
    if has("borderStyle") && !square {
        let hugging = format!(
            "{{{}, {}}}",
            n.attribute("horizontalHuggingPriority").unwrap_or("250"),
            n.attribute("verticalHuggingPriority").unwrap_or("250")
        );
        let at = values.iter().position(|v| v.0 == "IBNSSafeAreaLayoutGuide").expect("the safe-area key");
        nib.remove(&mut values, "NSHuggingPriority");
        let at = at.min(values.iter().position(|v| v.0 == "IBNSSafeAreaLayoutGuide").expect("the safe-area key"));
        let at = if values.iter().any(|v| v.0 == "NSAntiCompressionPriority") { at - 1 } else { at };
        if hugging != "{250, 750}" {
            values.insert(at, ("NSHuggingPriority", nib.string(&hugging)));
        }
    }
    values.push(("NSEnabled", Value::Bool(true)));
    values.push((
        "NSCell",
        Value::New(Object::new(if popup { "NSPopUpButtonCell" } else { "NSButtonCell" }, cell_values).named(cell_name)),
    ));
    values.push(("NSAllowsLogicalLayoutDirection", Value::Bool(nib.localizable)));
    values.push(("NSControlSize", Value::Int(control_size)));
    values.push(("NSControlContinuous", Value::Bool(false)));
    values.push(("NSControlRefusesFirstResponder", Value::Bool(false)));
    values.push(("NSControlUsesSingleLineMode", Value::Bool(false)));
    values.push(("NSControlTextAlignment", Value::Int(control_alignment)));
    values.push(("NSControlLineBreakMode", Value::Int(line_break)));
    values.push(("NSControlWritingDirection", Value::Int(-1)));
    values.push(("NSControlSendActionMask", Value::Int(4)));
    values.push(("IBNSShadowedSymbolConfiguration", Value::Nil));
    swapped(nib, n, module, if popup { "NSPopUpButton" } else { "NSButton" }, values, name)
}

/// A scroll view, as Xcode archives one it doesn't tile again: no border,
/// scrollers hidden and hiding themselves, no content insets, a document
/// view no larger than its clip view (20 probes: with a border, a scroller
/// showing, insets or a larger document, Xcode lays it out with AppKit's
/// scroller metrics, which this ibtool doesn't reproduce). `NSsFlags`:
/// 0x20800, hiding scrollers 0x200, horizontal and vertical scrollers 0x20
/// and 0x10 (the defaults), predominant-axis scrolling 0x10000 (the
/// default), content insets given 0x100000.
fn scroll_view(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("id", None),
            ("wantsLayer", Some("YES")),
            ("borderType", Some("none")),
            ("autohidesScrollers", Some("YES")),
            ("horizontalLineScroll", None),
            ("horizontalPageScroll", None),
            ("verticalLineScroll", None),
            ("verticalPageScroll", None),
            ("usesPredominantAxisScrolling", Some("NO")),
            ("hasHorizontalScroller", Some("NO")),
            ("hasVerticalScroller", Some("NO")),
            ("verticalScrollElasticity", None),
            ("horizontalScrollElasticity", None),
            ("translatesAutoresizingMaskIntoConstraints", Some("NO")),
            ("fixedFrame", Some("YES")),
        ],
    )?;
    only_children(n, &["rect", "autoresizingMask", "clipView", "edgeInsets", "scroller", "constraints", "accessibility"])?;
    if n.attribute("borderType").is_none() || n.attribute("autohidesScrollers").is_none() {
        return fail("a scroll view with a border or scrollers that don't hide themselves is laid out by Xcode, which this ibtool doesn't reproduce");
    }
    let id = attr(n, "id")?;
    let name = object_name(id);
    c.parents.push((name.clone(), parent.to_string()));
    let frame_n = frame(n, "frame")?;
    let clip = child(n, "clipView", "contentView").ok_or("a scroll view without a clip view")?;
    only_attributes(
        clip,
        &[("key", None), ("id", None), ("drawsBackground", Some("NO")), ("copiesOnScroll", Some("NO")), ("ambiguous", Some("YES"))],
    )?;
    only_children(clip, &["rect", "autoresizingMask", "subviews"])?;
    let clip_frame = frame(clip, "frame")?;
    if (clip_frame.x, clip_frame.y, clip_frame.w, clip_frame.h) != (0.0, 0.0, frame_n.w, frame_n.h) {
        return fail("a clip view other than its scroll view's bounds is not supported by this ibtool");
    }
    let docs: Vec<Node> = children(clip, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())).collect();
    let [doc] = docs.as_slice() else {
        return fail("a clip view without one document view is not supported by this ibtool");
    };
    // Laying a constrained scroll view out again, Xcode may turn off a
    // collection view's horizontal scroller (a corpus xib); one without it
    // is established (eleven corpus xibs, a probe).
    let constrained = n.attribute("translatesAutoresizingMaskIntoConstraints").is_some() && nib.constrained.contains(attr(n, "id")?);
    if doc.has_tag_name("collectionView") && constrained && n.attribute("hasHorizontalScroller").is_none() {
        return fail("a constrained scroll view's collection view with a horizontal scroller is laid out by Xcode, which this ibtool doesn't reproduce");
    }
    if !matches!(doc.tag_name().name(), "view" | "customView" | "collectionView") {
        return fail(format!("a scroll view's document <{}> is not supported by this ibtool", doc.tag_name().name()));
    }
    let doc_frame = frame(*doc, "frame")?;
    if doc_frame.w > clip_frame.w || doc_frame.h > clip_frame.h {
        return fail("a document view larger than its clip view makes Xcode show a scroller, which this ibtool doesn't reproduce");
    }
    if let Some(insets) = child(n, "edgeInsets", "contentInsets") {
        only_attributes(insets, &[("key", None), ("left", Some("0.0")), ("right", Some("0.0")), ("top", Some("0.0")), ("bottom", Some("0.0"))])?;
    }
    let mut scrollers = (None, None);
    for sc in children(n, "scroller") {
        match sc.attribute("key") {
            Some("horizontalScroller") => scrollers.0 = Some(sc),
            Some("verticalScroller") => scrollers.1 = Some(sc),
            _ => return fail("a scroll view's scroller other than its two"),
        }
    }
    let (Some(h), Some(v)) = scrollers else {
        return fail("a scroll view without both scrollers is not supported by this ibtool");
    };

    // The clip view: its flags without its mask, the document view, the
    // control background colour, drawing it (4) and copying on scroll
    // (unless not, 2).
    let clip_name = object_name(attr(clip, "id")?);
    c.parents.push((clip_name.clone(), name.clone()));
    let doc_name = object_name(attr(*doc, "id")?);
    // The document view is at the clip view's origin (a probe).
    c.at_origin.push(attr(*doc, "id")?.to_string());
    let mut clip_values = view_values(nib, c, clip, module, &clip_name, &name, Some(&name))?;
    for v in clip_values.iter_mut() {
        if v.0 == "NSvFlags" {
            v.1 = Value::Int(256);
        }
    }
    let at = clip_values.iter().position(|v| v.0 == "NSSuperview").expect("its superview") + 1;
    clip_values.insert(at, ("NSNextKeyView", reference(doc_name.clone())));
    clip_values.push(("NSDocView", reference(doc_name)));
    clip_values.push(("NSBGColor", system_color(nib, "controlBackgroundColor")?));
    let mut cv_flags = 0;
    if clip.attribute("drawsBackground").is_none() {
        cv_flags |= 4;
    }
    if clip.attribute("copiesOnScroll").is_some() {
        cv_flags |= 2;
    }
    if cv_flags != 0 {
        clip_values.push(("NScvFlags", Value::Int(cv_flags)));
    }
    clip_values.push(("NSAutomaticallyAdjustsContentInsets", Value::Bool(true)));
    let clip_value = Value::New(Object::new("NSClipView", clip_values).named(clip_name.clone()));
    let h_value = scroller(nib, c, h, module, &name, true)?;
    let v_value = scroller(nib, c, v, module, &name, false)?;
    // A storyboard whose document view is off its clip view's origin has
    // the vertical scroller first among its subviews, though not in the
    // objects' list (probes; a xib doesn't).
    let scrollers = if nib.storyboard && (doc_frame.x, doc_frame.y) != (0.0, 0.0) {
        vec![clip_value, v_value, h_value]
    } else {
        vec![clip_value, h_value, v_value]
    };

    let mut values = view_values(nib, c, n, module, &name, parent, superview(n, parent))?;
    let at = values.iter().position(|v| v.0 == "NSFrame" || v.0 == "NSFrameSize").expect("its frame");
    values.insert(at, ("NSSubviews", nib.array("NSMutableArray", scrollers)));
    let at = values.iter().position(|v| v.0 == "NSSuperview").expect("its superview") + 1;
    values.insert(at, ("NSNextKeyView", reference(clip_name.clone())));
    let mut flags = 0x20800 | 0x200;
    if n.attribute("hasHorizontalScroller").is_none() {
        flags |= 0x20;
    }
    if n.attribute("hasVerticalScroller").is_none() {
        flags |= 0x10;
    }
    if n.attribute("usesPredominantAxisScrolling").is_none() {
        flags |= 0x1_0000;
    }
    if child(n, "edgeInsets", "contentInsets").is_some() {
        flags |= 0x10_0000;
    }
    // Scroll elasticity, vertical in bits 12 and 13, horizontal in 14 and
    // 15: automatic 0, none 1, allowed 2 (five probes).
    for (key, shift) in [("verticalScrollElasticity", 12), ("horizontalScrollElasticity", 14)] {
        flags |= match n.attribute(key) {
            None | Some("automatic") => 0,
            Some("none") => 1,
            Some("allowed") => 2,
            Some(other) => return fail(format!("{key}=\"{other}\" is not supported by this ibtool")),
        } << shift;
    }
    values.push(("NSsFlags", Value::Int(flags)));
    values.push(("NSVScroller", reference(object_name(attr(v, "id")?))));
    values.push(("NSHScroller", reference(object_name(attr(h, "id")?))));
    values.push(("NSContentView", reference(clip_name)));
    // Its line and page scrolls, as big-endian floats: vertical line and
    // page, then horizontal (a probe), when given.
    let amounts = ["verticalLineScroll", "verticalPageScroll", "horizontalLineScroll", "horizontalPageScroll"];
    // Left out when all are AppKit's 10 (a probe).
    let defaults = amounts.iter().all(|a| n.attribute(*a).is_some_and(|v| number(v).is_ok_and(|v| v == 10.0)));
    if amounts.iter().any(|a| n.attribute(*a).is_some()) && !defaults {
        let mut data = Vec::new();
        for a in amounts {
            let v = n.attribute(a).ok_or_else(|| format!("a scroll view without its {a} is not supported by this ibtool"))?;
            data.extend_from_slice(&(number(v)? as f32).to_be_bytes());
        }
        values.push(("NSScrollAmts", Value::Data(data)));
    }
    values.push(("NSMinMagnification", Value::Double(0.25)));
    values.push(("NSMaxMagnification", Value::Double(4.0)));
    values.push(("NSMagnification", Value::Double(1.0)));
    if child(n, "edgeInsets", "contentInsets").is_some() {
        values.push(("NSContentInsets", nib.string("{0, 0, 0, 0}")));
    }
    Ok(Value::New(Object::new("NSScrollView", values).named(name)))
}

/// A scroll view's hidden scroller, targeting it with `_doScroller:`.
fn scroller(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str, horizontal: bool) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("key", None),
            ("id", None),
            ("hidden", Some("YES")),
            ("wantsLayer", Some("YES")),
            ("verticalHuggingPriority", Some("750")),
            ("doubleValue", None),
            ("horizontal", None),
        ],
    )?;
    only_children(n, &["rect", "autoresizingMask"])?;
    if n.attribute("hidden").is_none() {
        return fail("a scroller showing is laid out by Xcode, which this ibtool doesn't reproduce");
    }
    if n.attribute("horizontal") != Some(if horizontal { "YES" } else { "NO" }) {
        return fail("a scroller of the other orientation");
    }
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), parent.to_string()));
    let mut values = view_values(nib, c, n, module, &name, parent, superview(n, parent))?;
    nib.remove(&mut values, "NSHuggingPriority");
    // Layer-backed whether or not it says so (a probe).
    if !values.iter().any(|v| v.0 == "NSViewIsLayerTreeHost") {
        let at = values.iter().position(|v| v.0 == "NSViewWantsBestResolutionOpenGLSurface").expect("its OpenGL key");
        values.insert(at, ("NSViewIsLayerTreeHost", Value::Bool(true)));
    }
    let action = nib.string("_doScroller:");
    values.push(("NSAllowsLogicalLayoutDirection", Value::Bool(nib.localizable)));
    values.push(("NSControlAction", action));
    values.push(("NSControlTarget", reference(parent.to_string())));
    values.push(("NSControlSize", Value::Int(0)));
    values.push(("NSControlContinuous", Value::Bool(false)));
    values.push(("NSControlRefusesFirstResponder", Value::Bool(false)));
    values.push(("NSControlUsesSingleLineMode", Value::Bool(false)));
    values.push(("NSControlTextAlignment", Value::Int(0)));
    values.push(("NSControlLineBreakMode", Value::Int(0)));
    values.push(("NSControlWritingDirection", Value::Int(0)));
    values.push(("NSControlSendActionMask", Value::Int(4)));
    if horizontal {
        values.push(("NSsFlags", Value::Int(1)));
    }
    values.push(("NSTarget", reference(parent.to_string())));
    let action = nib.string("_doScroller:");
    values.push(("NSAction", action));
    if let Some(d) = n.attribute("doubleValue") {
        values.push(("NSCurValue", Value::Double(number(d)?)));
    }
    Ok(Value::New(Object::new("NSScroller", values).named(name)))
}

/// A box: a separator, or a custom box and its content view (about 30
/// probes). Xcode lays a custom box's content view out again, inset from
/// the box's bounds by its border width (1 unless given, none without a
/// border), whatever its frame says. A primary or secondary box places its
/// content below its title, measured with the Mac's fonts, which this
/// ibtool doesn't reproduce.
fn box_view(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("id", None),
            ("autoresizesSubviews", Some("NO")),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
            ("boxType", None),
            ("borderType", None),
            ("borderWidth", None),
            ("cornerRadius", None),
            ("title", None),
            ("titlePosition", None),
            ("transparent", Some("YES")),
            ("horizontalHuggingPriority", None),
            ("verticalHuggingPriority", None),
            ("horizontalCompressionResistancePriority", None),
            ("verticalCompressionResistancePriority", None),
            ("translatesAutoresizingMaskIntoConstraints", Some("NO")),
            ("fixedFrame", Some("YES")),
        ],
    )?;
    only_children(n, &["rect", "autoresizingMask", "view", "color", "constraints", "point", "accessibility"])?;
    canvas_location_only(n)?;
    let id = attr(n, "id")?;
    let name = object_name(id);
    let separator = match n.attribute("boxType") {
        Some("separator") => true,
        Some("custom") => false,
        _ => return fail("a primary or secondary box is laid out by Xcode with the Mac's fonts, which this ibtool doesn't reproduce"),
    };
    let border_type = match n.attribute("borderType") {
        None | Some("groove") => 3,
        Some("bezel") => 2,
        Some("line") => 1,
        Some("none") => 0,
        Some(other) => return fail(format!("border type {other} is not supported by this ibtool")),
    };
    let title_position = match n.attribute("titlePosition") {
        Some("noTitle") => 0,
        Some("aboveTop") => 1,
        None | Some("atTop") => 2,
        Some("belowTop") => 3,
        Some("aboveBottom") => 4,
        Some("atBottom") => 5,
        Some("belowBottom") => 6,
        Some(other) => return fail(format!("title position {other} is not supported by this ibtool")),
    };
    let content = child(n, "view", "contentView");
    if children(n, "view").any(|v| v.attribute("key") != Some("contentView")) {
        return fail("a box's view other than its content view is not supported by this ibtool");
    }
    if separator && (content.is_some() || children(n, "color").next().is_some() || ["borderWidth", "cornerRadius", "transparent"].iter().any(|a| n.attribute(*a).is_some())) {
        return fail("a separator box's content view, colours, border or transparency are not supported by this ibtool");
    }
    let content = match (separator, content) {
        (false, None) => return fail("a custom box without a content view is not supported by this ibtool"),
        (_, content) => content,
    };
    c.parents.push((name.clone(), parent.to_string()));
    if let Some(content) = content {
        let f = frame(n, "frame")?;
        let border = n.attribute("borderWidth").map(number).transpose()?.unwrap_or(1.0);
        let inset = if border_type == 0 { 0.0 } else { border };
        c.laid_out.insert(
            attr(content, "id")?.to_string(),
            Frame {
                x: inset,
                y: inset,
                w: f.w - 2.0 * inset,
                h: f.h - 2.0 * inset,
            },
        );
    }
    let mut values = view_values(nib, c, n, module, &name, parent, superview(n, parent))?;
    let offsets = if separator { "{5, 5}" } else { "{0, 0}" };
    values.push(("NSOffsets", nib.string(offsets)));
    // The title's cell: the small system font, the text background and
    // label colours.
    // Without a title, a plain "Title", even in Base.lproj (a corpus
    // storyboard's separator).
    let title = match n.attribute("title") {
        Some(t) => nib.text(id, "title", t),
        None => nib.string("Title"),
    };
    let font_name = nib.string(".AppleSystemUIFont");
    let font = nib.shared("font:.AppleSystemUIFont:11:3100".to_string(), || {
        Object::new("NSFont", vec![("NSName", font_name), ("NSSize", Value::Double(11.0)), ("NSfFlags", Value::Int(0xc1c))])
    });
    let background = system_color(nib, "textBackgroundColor")?;
    let text = system_color(nib, "labelColor")?;
    // A storyboard's leaves out its control view (probes).
    let mut cell = vec![
        ("NSCellFlags", Value::Int(0x400_0000)),
        ("NSCellFlags2", Value::Int(0x800_0000)),
        ("NSContents", title),
        ("NSSupport", font),
    ];
    if !nib.storyboard {
        cell.push(("NSControlView", Value::Nil));
    }
    cell.push(("NSBackgroundColor", background));
    cell.push(("NSTextColor", text));
    values.push(("NSTitleCell", Value::New(Object::new("NSTextFieldCell", cell))));
    if let Some(content) = content {
        values.push(("NSContentView", reference(object_name(attr(content, "id")?))));
    }
    values.push(("NSBorderType", Value::Int(border_type)));
    values.push(("NSBoxType", Value::Int(if separator { 2 } else { 4 })));
    values.push(("NSTitlePosition", Value::Int(title_position)));
    values.push(("NSTransparent", Value::Bool(false)));
    if n.attribute("transparent").is_some() {
        values.push(("NSFullyTransparent", Value::Bool(true)));
    }
    if let Some(width) = n.attribute("borderWidth") {
        // Numbers are shared by value: an object id of the same integer
        // refers to it (a probe's 2).
        let width = number(width)?;
        let key = if width.fract() == 0.0 { format!("n:{}", width as i64) } else { format!("d:{width}") };
        values.push(("NSBorderWidth2", nib.shared(key, || Object::new("NSNumber", vec![("NS.dblval", Value::Double(width))]))));
    }
    if let Some(radius) = n.attribute("cornerRadius") {
        values.push(("NSCornerRadius2", Value::Double(number(radius)?)));
    }
    for (key, archived) in [("borderColor", "NSBorderColor2"), ("fillColor", "NSFillColor2")] {
        if let Some(color_node) = child(n, "color", key) {
            values.push((archived, color(nib, color_node)?));
        }
    }
    if children(n, "color").any(|k| !matches!(k.attribute("key"), Some("borderColor" | "fillColor"))) {
        return fail("a box's colour other than its border and fill is not supported by this ibtool");
    }
    if content.is_some() {
        values.push(("IBNSBoxIsUsingDocumentContentView", Value::Bool(true)));
    }
    swapped(nib, n, module, "NSBox", values, name)
}

/// An image view and its cell (about 15 probes): an editable control
/// accepting eight drag types, its cell's flags as a text field's, the
/// image's alignment, scaling and frame style by AppKit's enums, and a named
/// image from the app's catalogs as an `NSCustomResource` of its document
/// size. AppKit's own images (namespace `system`) aren't established.
fn image_view(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("id", None),
            ("customClass", None),
            ("customModule", None),
            ("customModuleProvider", None),
            ("hidden", Some("YES")),
            ("horizontalHuggingPriority", None),
            ("verticalHuggingPriority", None),
            ("horizontalCompressionResistancePriority", None),
            ("verticalCompressionResistancePriority", None),
            ("translatesAutoresizingMaskIntoConstraints", Some("NO")),
            ("fixedFrame", Some("YES")),
            ("userLabel", None),
        ],
    )?;
    only_children(n, &["rect", "autoresizingMask", "imageCell", "constraints", "point", "accessibility"])?;
    canvas_location_only(n)?;
    // Its image sizes it.
    content_sized(nib, n)?;
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), parent.to_string()));
    let cell = child(n, "imageCell", "cell").ok_or("an image view without a cell")?;
    only_attributes(
        cell,
        &[
            ("key", None),
            ("id", None),
            ("refusesFirstResponder", Some("YES")),
            ("alignment", None),
            ("imageScaling", None),
            ("imageAlignment", None),
            ("imageFrameStyle", None),
            ("image", None),
        ],
    )?;
    only_children(cell, &[])?;
    let cell_name = object_name(attr(cell, "id")?);
    // Its cell comes before its own constraints in the objects' list.
    c.parents.push((cell_name.clone(), name.clone()));
    let mut values = view_values(nib, c, n, module, &name, parent, superview(n, parent))?;
    let types = [
        "Apple PDF pasteboard type",
        "Apple PICT pasteboard type",
        "Apple PNG pasteboard type",
        "NSFilenamesPboardType",
        "NeXT TIFF v4.0 pasteboard type",
        "com.apple.NSFilePromiseItemMetaData",
        "com.apple.pasteboard.promised-file-content-type",
        "dyn.ah62d4rv4gu8yc6durvwwa3xmrvw1gkdusm1044pxqyuha2pxsvw0e55bsmwca7d3sbwu",
    ];
    let types: Vec<Value> = types.iter().map(|t| nib.string(t)).collect();
    let at = values.iter().position(|v| v.0 == "NSFrame" || v.0 == "NSFrameSize").expect("its frame");
    values.insert(at, ("NSDragTypes", nib.array("NSMutableSet", types)));
    let refuses = cell.attribute("refusesFirstResponder").is_some();
    let (control_alignment, alignment_bits) = match cell.attribute("alignment") {
        None | Some("natural") => (4, 4),
        Some("left") => (0, 0),
        Some("center") => (1, 2),
        Some("right") => (2, 1),
        Some("justified") => (3, 3),
        Some(other) => return fail(format!("alignment {other} is not supported by this ibtool")),
    };
    let flags2 = alignment_bits << 26 | if refuses { 0x200_0000 } else { 0 };
    let mut cell_values = vec![("NSCellFlags", Value::Int(0)), ("NSCellFlags2", Value::Int(flags2))];
    if let Some(image) = cell.attribute("image") {
        cell_values.push(("NSContents", catalog_image(nib, n, image)?));
    }
    cell_values.push(("NSControlView", reference(name.clone())));
    let align = match cell.attribute("imageAlignment") {
        None | Some("center") => 0,
        Some("top") => 1,
        Some("topLeft") => 2,
        Some("topRight") => 3,
        Some("left") => 4,
        Some("bottom") => 5,
        Some("bottomLeft") => 6,
        Some("bottomRight") => 7,
        Some("right") => 8,
        Some(other) => return fail(format!("image alignment {other} is not supported by this ibtool")),
    };
    let scale = match cell.attribute("imageScaling") {
        None | Some("proportionallyDown") => 0,
        Some("axesIndependently") => 1,
        Some("none") => 2,
        Some("proportionallyUpOrDown") => 3,
        Some(other) => return fail(format!("image scaling {other} is not supported by this ibtool")),
    };
    let style = match cell.attribute("imageFrameStyle") {
        None => 0,
        Some("grayBezel") => 2,
        Some(other) => return fail(format!("image frame style {other} is not supported by this ibtool")),
    };
    cell_values.push(("NSAlign", Value::Int(align)));
    cell_values.push(("NSScale", Value::Int(scale)));
    cell_values.push(("NSStyle", Value::Int(style)));
    // A storyboard's animates its images, a xib's doesn't (probes).
    cell_values.push(("NSAnimates", Value::Bool(nib.storyboard)));
    cell_values.push(("NSImageAnimation", Value::Int(-1)));
    values.push(("NSEnabled", Value::Bool(true)));
    values.push(("NSCell", Value::New(Object::new("NSImageCell", cell_values).named(cell_name))));
    values.push(("NSAllowsLogicalLayoutDirection", Value::Bool(nib.localizable)));
    values.push(("NSControlSize", Value::Int(0)));
    values.push(("NSControlContinuous", Value::Bool(false)));
    values.push(("NSControlRefusesFirstResponder", Value::Bool(refuses)));
    values.push(("NSControlUsesSingleLineMode", Value::Bool(false)));
    values.push(("NSControlTextAlignment", Value::Int(control_alignment)));
    values.push(("NSControlLineBreakMode", Value::Int(0)));
    values.push(("NSControlWritingDirection", Value::Int(-1)));
    values.push(("NSControlSendActionMask", Value::Int(4)));
    values.push(("NSEditable", Value::Bool(true)));
    values.push(("NSImageViewPlaceholderPrecedence", Value::Int(0)));
    values.push(("IBNSShadowedSymbolConfiguration", Value::Nil));
    swapped(nib, n, module, "NSImageView", values, name)
}

/// A view controller's view: a view, or a SceneKit view (Xcode's SceneKit
/// game templates).
fn controller_view<'a, 'i>(vc: Node<'a, 'i>) -> Option<Node<'a, 'i>> {
    vc.children().find(|c| (c.has_tag_name("view") || c.has_tag_name("sceneView")) && c.attribute("key") == Some("view"))
}

/// A web view as Xcode's Safari extension templates make it. Xcode
/// archives a live WKWebView, so its keys are WebKit's on the Mac that ran
/// it (macOS 27): 17 drag types, its own flipped view and two
/// hidden colour extension views as subviews outside the objects' list, and
/// a default configuration (two templates).
fn web_view(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("id", None),
            ("wantsLayer", Some("YES")),
            ("fixedFrame", Some("YES")),
            ("translatesAutoresizingMaskIntoConstraints", Some("NO")),
        ],
    )?;
    only_children(n, &["rect", "autoresizingMask", "wkWebViewConfiguration"])?;
    let config = child(n, "wkWebViewConfiguration", "configuration").ok_or("a web view without a configuration")?;
    only_attributes(config, &[("key", None)])?;
    only_children(config, &["audiovisualMediaTypes", "wkPreferences"])?;
    let media = child(config, "audiovisualMediaTypes", "mediaTypesRequiringUserActionForPlayback").ok_or("a web view's configuration other than Xcode's templates'")?;
    only_attributes(media, &[("key", None), ("none", Some("YES"))])?;
    let preferences = child(config, "wkPreferences", "preferences").ok_or("a web view's configuration other than Xcode's templates'")?;
    only_attributes(preferences, &[("key", None)])?;
    only_children(preferences, &[])?;
    if n.attribute("wantsLayer").is_none() || media.attribute("none").is_none() {
        return fail("a web view other than Xcode's templates' is not supported by this ibtool");
    }
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), parent.to_string()));
    let mut values = view_values(nib, c, n, module, &name, parent, superview(n, parent))?;
    let f = frame(n, "frame")?;
    let size = nib.string(&format!("{{{}, {}}}", g(f.w), g(f.h)));
    let internal = |nib: &mut Nib, class: &'static str, hidden: bool, size: Value| {
        let mut v = vec![
            ("NSNextResponder", reference(name.clone())),
            ("NSNibTouchBar", Value::Nil),
            ("NSvFlags", Value::Int(if hidden { 0x8000_0000_u32 as i32 as i64 | 256 } else { 274 })),
            ("NSFrameSize", size),
            ("NSSuperview", reference(name.clone())),
            ("NSViewIsLayerTreeHost", Value::Bool(true)),
            ("NSViewWantsBestResolutionOpenGLSurface", Value::Bool(true)),
        ];
        if !hidden {
            v.push(("NSViewClipsToBoundsKeyName", Value::Bool(true)));
        }
        v.push(("IBNSSafeAreaLayoutGuide", Value::Nil));
        v.push(("IBNSLayoutMarginsGuide", Value::Nil));
        v.push(("IBNSClipsToBounds", Value::Int(0)));
        let _ = nib;
        Value::New(Object::new(class, v))
    };
    let flipped = internal(nib, "WKFlippedView", false, size);
    let zero = nib.string("{0, 0}");
    let first = internal(nib, "WKColorExtensionView", true, zero);
    let zero = nib.string("{0, 0}");
    let second = internal(nib, "WKColorExtensionView", true, zero);
    let at = values.iter().position(|v| v.0 == "NSFrame" || v.0 == "NSFrameSize").expect("its frame");
    values.insert(at, ("NSSubviews", nib.array("NSMutableArray", vec![flipped, first, second])));
    let types = [
        "Apple HTML pasteboard type",
        "Apple PDF pasteboard type",
        "Apple URL pasteboard type",
        "Apple Web Archive pasteboard type",
        "Apple WebKit dummy pasteboard type",
        "Apple files promise pasteboard type",
        "NSColor pasteboard type",
        "NSFilenamesPboardType",
        "NSStringPboardType",
        "NeXT RTFD pasteboard type",
        "NeXT Rich Text Format v1.0 pasteboard type",
        "NeXT TIFF v4.0 pasteboard type",
        "WebURLsWithTitlesPboardType",
        "com.apple.webarchive",
        "public.png",
        "public.url",
        "public.url-name",
    ];
    let types: Vec<Value> = types.iter().map(|t| nib.string(t)).collect();
    let at = values.iter().position(|v| v.0 == "NSFrame" || v.0 == "NSFrameSize").expect("its frame");
    values.insert(at, ("NSDragTypes", nib.array("NSMutableSet", types)));
    let preferences = Object::new(
        "WKPreferences",
        vec![
            ("minimumFontSize", Value::Double(0.0)),
            ("javaScriptCanOpenWindowsAutomatically", Value::Bool(true)),
            ("javaScriptEnabled", Value::Bool(true)),
            ("shouldPrintBackgrounds", Value::Bool(false)),
            ("tabFocusesLinks", Value::Bool(false)),
            ("textInteractionEnabled", Value::Bool(true)),
        ],
    );
    let empty = nib.string("");
    let configuration = Object::new(
        "WKWebViewConfiguration",
        vec![
            ("processPool", Value::New(Object::new("WKProcessPool", vec![]))),
            ("preferences", Value::New(preferences)),
            ("userContentController", Value::New(Object::new("WKUserContentController", vec![]))),
            ("websiteDataStore", Value::New(Object::new("WKWebsiteDataStore", vec![("isDefaultDataStore", Value::Bool(true))]))),
            ("suppressesIncrementalRendering", Value::Bool(false)),
            ("applicationNameForUserAgent", empty),
            ("allowsAirPlayForMediaPlayback", Value::Bool(true)),
            ("drawsBackground", Value::Bool(true)),
            ("userInterfaceDirectionPolicy", Value::Int(0)),
            ("scrollToTextFragmentIndicatorEnabled", Value::Bool(true)),
            ("scrollToTextFragmentMarkingEnabled", Value::Bool(true)),
            ("multiRepresentationHEICInsertionEnabled", Value::Bool(false)),
        ],
    );
    values.push(("configuration", Value::New(configuration)));
    values.push(("allowsBackForwardNavigationGestures", Value::Bool(false)));
    values.push(("customUserAgent", nib.string("")));
    values.push(("allowsLinkPreview", Value::Bool(true)));
    values.push(("allowsMagnification", Value::Bool(false)));
    values.push(("magnification", Value::Double(1.0)));
    Ok(Value::New(Object::new("WKWebView", values).named(name)))
}

/// A collection view item at a xib's top level: its class swapped in for
/// NSCollectionViewItem, then its presentation style 0 (a corpus xib).
fn collection_item(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(n, &[("id", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None)])?;
    only_children(n, &["connections"])?;
    if n.attribute("customClass").is_none() {
        return fail("a collection view item without a class is not supported by this ibtool");
    }
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), parent.to_string()));
    if children(n, "connections").next().is_some() {
        c.connection_sources.push(attr(n, "id")?.to_string());
    }
    let values = vec![
        ("NSClassName", nib.string(&runtime_class_name(n, module, "NSCollectionViewItem")?)),
        ("NSOriginalClassName", nib.string("NSCollectionViewItem")),
        ("showSeguePresentationStyle", Value::Int(0)),
    ];
    Ok(Value::New(Object::new("NSClassSwapper", values).named(name)))
}

/// A collection view with a flow layout, as a scroll view's document (about
/// ten probes): its core holding the layout and its selection, empty grid
/// sizes, its background colours and selection, and dragging masks. The
/// layout archives its spacings and item size only when not 10, 10 and 50
/// by 50.
fn collection_view(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(n, &[("id", None), ("ambiguous", Some("YES")), ("selectable", Some("YES")), ("allowsMultipleSelection", Some("YES"))])?;
    only_children(n, &["rect", "autoresizingMask", "collectionViewFlowLayout", "color"])?;
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), parent.to_string()));
    let layout = child(n, "collectionViewFlowLayout", "collectionViewLayout").ok_or("a collection view without a flow layout is not supported by this ibtool")?;
    only_attributes(layout, &[("key", None), ("id", None), ("minimumInteritemSpacing", None), ("minimumLineSpacing", None)])?;
    only_children(layout, &["size"])?;
    let layout_name = object_name(attr(layout, "id")?);
    c.parents.push((layout_name.clone(), name.clone()));
    let mut values = view_values(nib, c, n, module, &name, parent, Some(parent))?;
    let mut layout_values = Vec::new();
    for (key, archived) in [("minimumInteritemSpacing", "NSInteritemSpacing"), ("minimumLineSpacing", "NSLineSpacing")] {
        let v = number(layout.attribute(key).unwrap_or("10"))?;
        if v != 10.0 {
            // As floats (a probe).
            layout_values.push((archived, Value::Float(v as f32)));
        }
    }
    if let Some(size) = child(layout, "size", "itemSize") {
        only_attributes(size, &[("key", None), ("width", None), ("height", None)])?;
        let (w, h) = (number(attr(size, "width")?)?, number(attr(size, "height")?)?);
        if (w, h) != (50.0, 50.0) {
            let text = nib.string(&format!("{{{}, {}}}", g(w), g(h)));
            layout_values.push(("NSItemSize", Value::New(Object::new("NSValue", vec![("NS.special", Value::Int(2)), ("NS.sizeval", text)]))));
        }
    }
    let layout_value = Value::New(Object::new("NSCollectionViewFlowLayout", layout_values).named(layout_name));
    let selectable = n.attribute("selectable").is_some();
    let multiple = n.attribute("allowsMultipleSelection").is_some();
    let mut core = vec![("UICollectionLayout", layout_value)];
    if !selectable {
        core.push(("UIAllowsSelection", Value::Bool(false)));
    }
    if multiple {
        core.push(("UIAllowsMultipleSelection", Value::Bool(true)));
    }
    values.push(("NSCollectionViewCore", Value::New(Object::new("_NSCollectionViewCore", core))));
    let zero = nib.string("{0, 0}");
    values.push(("NSMinGridSize", zero));
    let zero = nib.string("{0, 0}");
    values.push(("NSMaxGridSize", zero));
    values.push(("NSMaxNumberOfGridRows", Value::Int(0)));
    values.push(("NSMaxNumberOfGridColumns", Value::Int(0)));
    let background = child(n, "color", "primaryBackgroundColor").ok_or("a collection view without a background colour is not supported by this ibtool")?;
    let background = color(nib, background)?;
    values.push(("NSBackgroundColors", nib.array("NSArray", vec![background])));
    if selectable {
        values.push(("NSSelectable", Value::Bool(true)));
    }
    if multiple {
        values.push(("NSAllowsMultipleSelection", Value::Bool(true)));
    }
    values.push(("NSDraggingSourceMaskForLocal", Value::Int(-1)));
    values.push(("NSDraggingSourceMaskForNonLocal", Value::Int(0)));
    Ok(Value::New(Object::new("NSCollectionView", values).named(name)))
}

/// A linear slider (about ten probes): vertical when its frame is taller
/// than wide, its own content hugging 250 and 750 (750 and 250 vertical),
/// its cell's state, continuity and enabling as a text field cell's, its
/// values, tick marks (below or right 0, above or left 1) and orientation.
/// A continuous one sends its action on dragging too (mask 70). Its height
/// is its own, so a constrained one is laid out by Xcode.
fn slider(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(
        n,
        &[
            ("id", None),
            ("horizontalHuggingPriority", None),
            ("verticalHuggingPriority", None),
            ("horizontalCompressionResistancePriority", None),
            ("verticalCompressionResistancePriority", None),
            ("translatesAutoresizingMaskIntoConstraints", Some("NO")),
            ("fixedFrame", Some("YES")),
        ],
    )?;
    only_children(n, &["rect", "autoresizingMask", "sliderCell", "constraints", "connections", "accessibility"])?;
    content_sized(nib, n)?;
    let id = attr(n, "id")?;
    let name = object_name(id);
    c.parents.push((name.clone(), parent.to_string()));
    if children(n, "connections").next().is_some() {
        c.connection_sources.push(id.to_string());
    }
    let cell = child(n, "sliderCell", "cell").ok_or("a slider without a cell")?;
    only_attributes(
        cell,
        &[
            ("key", None),
            ("id", None),
            ("continuous", Some("YES")),
            ("state", Some("on")),
            ("enabled", Some("NO")),
            ("alignment", Some("left")),
            ("controlSize", None),
            ("minValue", None),
            ("maxValue", None),
            ("doubleValue", None),
            ("tickMarkPosition", None),
            ("sliderType", Some("linear")),
            ("numberOfTickMarks", None),
            ("allowsTickMarkValuesOnly", Some("YES")),
        ],
    )?;
    only_children(cell, &[])?;
    let cell_name = object_name(attr(cell, "id")?);
    c.parents.push((cell_name.clone(), name.clone()));
    let f = frame(n, "frame")?;
    let vertical = f.h > f.w;
    let mut values = view_values(nib, c, n, module, &name, parent, superview(n, parent))?;
    // Its own hugging is left out (four probes).
    let own = if vertical { ("750", "250") } else { ("250", "750") };
    let hugging = (n.attribute("horizontalHuggingPriority").unwrap_or("250"), n.attribute("verticalHuggingPriority").unwrap_or("250"));
    let at = values.iter().position(|v| v.0 == "IBNSSafeAreaLayoutGuide").expect("the safe-area key");
    nib.remove(&mut values, "NSHuggingPriority");
    let at = at.min(values.iter().position(|v| v.0 == "IBNSSafeAreaLayoutGuide").expect("the safe-area key"));
    let at = if values.iter().any(|v| v.0 == "NSAntiCompressionPriority") { at - 1 } else { at };
    if hugging != own {
        values.insert(at, ("NSHuggingPriority", nib.string(&format!("{{{}, {}}}", hugging.0, hugging.1))));
    }
    let has = |a: &str| cell.attribute(a).is_some();
    let mut flags: i64 = 0;
    if has("state") {
        flags |= 0x8000_0000;
    }
    if has("continuous") {
        flags |= 0x4_0100;
    }
    if has("enabled") {
        flags |= 0x2000_0000;
    }
    let (control_size, size_bits) = match cell.attribute("controlSize") {
        None | Some("regular") => (0, 0),
        Some("small") => (1, 0x2_0000),
        Some("mini") => (2, 0x4_0000),
        Some(other) => return fail(format!("control size {other} is not supported by this ibtool")),
    };
    let min = number(cell.attribute("minValue").unwrap_or("0"))?;
    let max = number(cell.attribute("maxValue").ok_or("a slider without a maximum is not supported by this ibtool")?)?;
    let value = match cell.attribute("doubleValue") {
        Some(v) => number(v)?,
        None => min,
    };
    // Below or right 0, above or left 1, whatever its orientation (five
    // probes).
    let ticks = match cell.attribute("tickMarkPosition") {
        Some("below" | "right") => 0,
        Some("above" | "left") => 1,
        other => return fail(format!("a slider's tick mark position {} is not supported by this ibtool", other.unwrap_or("default"))),
    };
    let signed = |v: i64| v as u32 as i32 as i64;
    let cell_values = vec![
        ("NSCellFlags", Value::Int(signed(flags))),
        ("NSCellFlags2", Value::Int(size_bits)),
        ("NSControlView", reference(name.clone())),
        ("NSMaxValue", Value::Double(max)),
        ("NSMinValue", Value::Double(min)),
        ("NSValue", Value::Double(value)),
        ("NSAltIncValue", Value::Double(0.0)),
        (
            "NSNumberOfTickMarks",
            Value::Int(
                cell.attribute("numberOfTickMarks")
                    .map(|t| t.parse().map_err(|_| format!("not a count: {t}")))
                    .transpose()?
                    .unwrap_or(0),
            ),
        ),
        ("NSTickMarkPosition", Value::Int(ticks)),
        ("NSAllowsTickMarkValuesOnly", Value::Bool(has("allowsTickMarkValuesOnly"))),
        ("NSVertical", Value::Bool(vertical)),
    ];
    values.push(("NSEnabled", Value::Bool(true)));
    values.push(("NSCell", Value::New(Object::new("NSSliderCell", cell_values).named(cell_name))));
    values.push(("NSAllowsLogicalLayoutDirection", Value::Bool(nib.localizable)));
    values.push(("NSControlSize", Value::Int(control_size)));
    values.push(("NSControlContinuous", Value::Bool(has("continuous"))));
    values.push(("NSControlRefusesFirstResponder", Value::Bool(false)));
    values.push(("NSControlUsesSingleLineMode", Value::Bool(false)));
    values.push(("NSControlTextAlignment", Value::Int(0)));
    values.push(("NSControlLineBreakMode", Value::Int(0)));
    values.push(("NSControlWritingDirection", Value::Int(-1)));
    values.push(("NSControlSendActionMask", Value::Int(if has("continuous") { 70 } else { 4 })));
    Ok(Value::New(Object::new("NSSlider", values).named(name)))
}

/// A SceneKit view as Xcode's game templates make it: a view, not looping,
/// with the default rendering API, then SCNView's keys (two templates).
fn scene_view(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str) -> Result<Value, String> {
    only_attributes(n, &[("key", None), ("id", None), ("loops", Some("NO")), ("preferredRenderingAPI", Some("-1"))])?;
    only_children(n, &["rect", "autoresizingMask", "connections"])?;
    if n.attribute("loops").is_none() || n.attribute("preferredRenderingAPI").is_none() {
        return fail("a SceneKit view other than Xcode's templates' is not supported by this ibtool");
    }
    let name = object_name(attr(n, "id")?);
    c.parents.push((name.clone(), parent.to_string()));
    // Its outlet to its controller (Xcode's multi-platform template).
    if children(n, "connections").next().is_some() {
        c.connection_sources.push(attr(n, "id")?.to_string());
    }
    let mut values = view_values(nib, c, n, module, &name, parent, None)?;
    values.push(("loops", Value::Bool(false)));
    values.push(("playing", Value::Bool(false)));
    values.push(("jitteringEnabled", Value::Bool(false)));
    values.push(("allowsCameraControl", Value::Bool(false)));
    values.push(("ibWantsMultisampling", Value::Bool(true)));
    values.push(("ibPreferredRenderingAPI", Value::Int(-1)));
    values.push(("autoenablesDefaultLighting", Value::Bool(false)));
    values.push(("ibSceneName", Value::Nil));
    values.push(("backgroundColor", Value::Nil));
    Ok(Value::New(Object::new("SCNView", values).named(name)))
}

/// A container view's embed segue: its only connection.
fn embed_segue<'a, 'i>(n: Node<'a, 'i>) -> Result<Option<Node<'a, 'i>>, String> {
    let all: Vec<Node> = children(n, "connections").flat_map(|l| l.children().filter(|c| c.is_element())).collect();
    match all.as_slice() {
        [] => Ok(None),
        [s] if s.has_tag_name("segue") && s.attribute("kind") == Some("embed") => {
            only_attributes(*s, &[("destination", None), ("kind", None), ("identifier", None), ("id", None)])?;
            Ok(Some(*s))
        }
        _ => fail("a container view's connections other than one embed segue are not supported by this ibtool"),
    }
}

/// The embed segues of a view controller's view, in document order.
fn embeds<'a, 'i>(vc: Node<'a, 'i>) -> Vec<Node<'a, 'i>> {
    controller_view(vc)
        .into_iter()
        .flat_map(|v| v.descendants())
        .filter(|d| d.has_tag_name("segue") && d.attribute("kind") == Some("embed"))
        .collect()
}

/// Whether a view controller's view nib needs it as an upstream placeholder:
/// it has connections, or its view's objects connect to it.
fn uses_upstream(vc: Node) -> bool {
    let id = vc.attribute("id");
    children(vc, "connections").next().is_some()
        || controller_view(vc)
            .into_iter()
            .flat_map(|v| v.descendants())
            .any(|d| (d.has_tag_name("outlet") && d.attribute("destination") == id) || (d.has_tag_name("action") && d.attribute("target") == id))
}
