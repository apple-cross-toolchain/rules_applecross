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

    fn string(&mut self, s: &str) -> Value {
        self.shared(format!("s:{s}"), || Object::new("NSString", vec![("NS.bytes", Value::Data(s.as_bytes().to_vec()))]))
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

struct Frame {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn frame(n: Node, key: &str) -> Result<Frame, String> {
    let r = child(n, "rect", key).ok_or_else(|| format!("<{}> without a {key} rect", n.tag_name().name()))?;
    only_attributes(r, &[("key", None), ("x", None), ("y", None), ("width", None), ("height", None)])?;
    Ok(Frame { x: number(attr(r, "x")?)?, y: number(attr(r, "y")?)?, w: number(attr(r, "width")?)?, h: number(attr(r, "height")?)? })
}

/// What compiling the document collects beside the objects themselves.
struct Collected {
    /// (object, parent) in the order the nib lists them.
    parents: Vec<(String, String)>,
    /// The windows visible at launch.
    visible: Vec<String>,
}

/// An autoresizing mask's bits, as NSView's: flexible left margin 1, width
/// 2, right margin 4, bottom margin 8, height 16, top margin 32.
fn autoresizing_mask(n: Node) -> Result<i64, String> {
    let Some(m) = child(n, "autoresizingMask", "autoresizingMask") else {
        return fail(format!("<{}> without an autoresizing mask is not supported by this ibtool", n.tag_name().name()));
    };
    only_attributes(m, &[("key", None), ("flexibleMinX", Some("YES")), ("widthSizable", Some("YES")), ("flexibleMaxX", Some("YES")), ("flexibleMinY", Some("YES")), ("heightSizable", Some("YES")), ("flexibleMaxY", Some("YES"))])?;
    let mut mask = 0;
    for (name, bit) in [("flexibleMinX", 1), ("widthSizable", 2), ("flexibleMaxX", 4), ("flexibleMinY", 8), ("heightSizable", 16), ("flexibleMaxY", 32)] {
        if m.attribute(name).is_some() {
            mask |= bit;
        }
    }
    Ok(mask)
}

/// A view and its subviews: an `NSView`, or for a top-level custom view an
/// `NSClassSwapper` for its class.
fn view(nib: &mut Nib, c: &mut Collected, n: Node, module: Option<&str>, parent: &str, next_responder: Option<&str>, swapped: bool) -> Result<Value, String> {
    only_attributes(n, &[("key", None), ("id", None), ("wantsLayer", Some("YES")), ("customClass", None), ("customModule", None), ("customModuleProvider", None)])?;
    only_children(n, &["rect", "autoresizingMask", "subviews"])?;
    let id = attr(n, "id")?;
    let name = object_name(id);
    c.parents.push((name.clone(), parent.to_string()));
    let mut values = Vec::new();
    let class = runtime_class_name(n, module, "NSView")?;
    if swapped {
        values.push(("NSClassName", nib.string(&class)));
        values.push(("NSOriginalClassName", nib.string("NSView")));
    } else if class != "NSView" {
        return fail("a custom class on a view other than a top-level one is not supported by this ibtool");
    }
    values.push(("NSNextResponder", next_responder.map_or(Value::Nil, reference)));
    values.push(("NSNibTouchBar", Value::Nil));
    values.push(("NSvFlags", Value::Int(256 | autoresizing_mask(n)?)));
    // Its subviews, then its frame (a template's text field; plain views
    // alone so far).
    let subviews: Vec<Node> = children(n, "subviews").flat_map(|s| s.children().filter(|c| c.is_element())).collect();
    if !subviews.is_empty() {
        let mut items = Vec::new();
        for s in subviews {
            if !matches!(s.tag_name().name(), "view" | "customView") {
                return fail(format!("a subview <{}> is not supported by this ibtool", s.tag_name().name()));
            }
            items.push(view(nib, c, s, module, &name, Some(&name), false)?);
        }
        values.push(("NSSubviews", nib.array("NSMutableArray", items)));
    }
    let f = frame(n, "frame")?;
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
    values.push(("IBNSSafeAreaLayoutGuide", Value::Nil));
    values.push(("IBNSLayoutMarginsGuide", Value::Nil));
    values.push(("IBNSClipsToBounds", Value::Int(0)));
    Ok(Value::New(Object::new(if swapped { "NSClassSwapper" } else { "NSView" }, values).named(name)))
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
    only_attributes(m, &[("key", None), ("leftStrut", Some("YES")), ("rightStrut", Some("YES")), ("topStrut", Some("YES")), ("bottomStrut", Some("YES")), ("horizontalStrut", Some("YES")), ("verticalStrut", Some("YES"))])?;
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
    only_attributes(n, &[
        ("title", None), ("allowsToolTipsWhenApplicationIsInactive", Some("NO")), ("autorecalculatesKeyViewLoop", Some("NO")), ("releasedWhenClosed", Some("NO")),
        ("deferred", Some("NO")), ("frameAutosaveName", None), ("animationBehavior", Some("default")), ("visibleAtLaunch", Some("NO")), ("id", None), ("customClass", None),
        ("customModule", None), ("customModuleProvider", None),
    ])?;
    only_children(n, &["windowStyleMask", "windowPositionMask", "rect", "value", "view", "point", "connections"])?;
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
    if n.attribute("visibleAtLaunch").is_none() {
        c.visible.push(name.clone());
    }
    let style = child(n, "windowStyleMask", "styleMask").ok_or("a window without a style mask")?;
    only_attributes(style, &[("key", None), ("titled", Some("YES")), ("closable", Some("YES")), ("miniaturizable", Some("YES")), ("resizable", Some("YES"))])?;
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
    let x = if has("leftStrut") || has("rightStrut") { content.x } else { screen.x + (screen.w - content.w) / 2.0 };
    let y = if has("topStrut") || has("bottomStrut") { content.y } else { screen.y + (screen.h - content.h) / 2.0 };
    let contents = children(n, "view").collect::<Vec<_>>();
    let [content_view] = contents.as_slice() else {
        return fail("a window without one content view is not supported by this ibtool");
    };
    if content_view.attribute("key") != Some("contentView") {
        return fail("a window's view other than its content view is not supported by this ibtool");
    }
    let titlebar = if mask & 1 != 0 { 32.0 } else { 0.0 };
    let size = |nib: &mut Nib, (w, h): (f64, f64), extra: f64| nib.string(&format!("{{{}, {}}}", g(w), g(h + extra)));
    let mut values = vec![
        ("NSWindowStyleMask", Value::Int(mask)),
        ("NSWindowBacking", Value::Int(2)),
        ("NSWindowRect", nib.string(&format!("{{{{{}, {}}}, {{{}, {}}}}}", g(x), g(y), g(content.w), g(content.h)))),
        ("NSWTFlags", Value::Int(flags)),
        ("NSWindowTitle", nib.string(n.attribute("title").unwrap_or(""))),
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
    values.push(("NSWindowView", view(nib, c, *content_view, module, &name, None, false)?));
    values.push(("NSScreenRect", nib.string(&format!("{{{{{}, {}}}, {{{}, {}}}}}", g(screen.x), g(screen.y), g(screen.w), g(screen.h)))));
    if let Some(min) = sizes[0] {
        values.push(("NSMinSize", size(nib, min, titlebar)));
    }
    values.push(("NSMaxSize", match sizes[1] {
        Some(max) => size(nib, max, titlebar),
        None => nib.string("{10000000000000, 10000000000000}"),
    }));
    if let Some(name) = n.attribute("frameAutosaveName") {
        values.push(("NSFrameAutosaveName", nib.string(name)));
    }
    values.push(("NSWindowIsRestorable", Value::Bool(true)));
    values.push(("NSMinFullScreenContentSize", nib.string("{0, 0}")));
    values.push(("NSMaxFullScreenContentSize", nib.string("{0, 0}")));
    Ok(Value::New(Object::new("NSWindowTemplate", values).named(name)))
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
    let reference = Object::new("IBClassReference", vec![
        ("IBClassName", nib.string(class)),
        ("IBModuleName", module_name.map_or(Value::Nil, |m| nib.string(&m))),
        ("IBModuleProvider", provider.map_or(Value::Nil, |p| nib.string(p))),
    ]);
    let runtime = match runtime {
        Some(r) => r.to_string(),
        None => runtime_class_name(n, module, "NSObject")?,
    };
    Ok(vec![("IBClassReference", Value::New(reference)), ("NSClassName", nib.string(&runtime))])
}

/// Compiles an AppKit xib to its nib.
pub fn compile_xib(xml: &str, module: Option<&str>) -> Result<Vec<u8>, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    if root.attribute("targetRuntime") != Some("MacOSX.Cocoa") {
        return fail("an AppKit xib for another runtime than MacOSX.Cocoa is not supported by this ibtool");
    }
    only_children(root, &["dependencies", "objects", "resources"])?;
    let objects_node = children(root, "objects").next().ok_or("a xib without objects")?;
    let mut nib = Nib { shared: HashMap::new(), next: 0 };
    let mut c = Collected { parents: Vec::new(), visible: Vec::new() };
    let mut owner = None;
    let mut application = None;
    let mut tops = Vec::new();
    let mut connections: Vec<(String, Value)> = Vec::new();
    for o in objects_node.children().filter(|c| c.is_element()) {
        match (o.tag_name().name(), o.attribute("id")) {
            ("customObject", Some("-2")) => owner = Some(o),
            ("customObject", Some("-1")) => {
                only_attributes(o, &[("id", None), ("userLabel", None), ("customClass", Some("FirstResponder"))])?;
                only_children(o, &[])?;
            }
            ("customObject", Some("-3")) => application = Some(o),
            ("customView" | "window" | "customObject", _) => tops.push(o),
            (tag, _) => return fail(format!("a xib's top-level <{tag}> is not supported by this ibtool")),
        }
    }
    let owner = owner.ok_or("a xib without a File's Owner")?;
    only_attributes(owner, &[("id", None), ("userLabel", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None)])?;
    only_children(owner, &["connections"])?;
    let owner_name = object_name("-2");
    let root_object = Object::new("NSCustomObject", class_values(&mut nib, owner, module, None)?).named(owner_name.clone());

    // The application placeholder: its class, NSApplication at run time.
    let application = application.ok_or("a xib without an Application placeholder")?;
    only_attributes(application, &[("id", None), ("userLabel", None), ("customClass", Some("NSObject"))])?;
    only_children(application, &[])?;
    let application_name = object_name("-3");
    let application_object = Object::new("NSCustomObject", class_values(&mut nib, application, module, Some("NSApplication"))?).named(application_name.clone());
    c.parents.push((application_name.clone(), owner_name.clone()));

    let mut definitions = Vec::new();
    for t in &tops {
        let value = match t.tag_name().name() {
            "window" => window(&mut nib, &mut c, *t, module, &owner_name)?,
            // A custom object: its class swapped in for NSObject, made with
            // init as the document says (an installer pane template).
            "customObject" => {
                only_attributes(*t, &[("id", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("userLabel", None)])?;
                only_children(*t, &["connections"])?;
                if t.attribute("customClass").is_none() || root.attribute("customObjectInstantitationMethod") != Some("direct") {
                    return fail("a custom object without a class, or made other than directly, is not supported by this ibtool");
                }
                let name = object_name(attr(*t, "id")?);
                c.parents.push((name.clone(), owner_name.clone()));
                let values = vec![
                    ("NSClassName", nib.string(&runtime_class_name(*t, module, "NSObject")?)),
                    ("NSOriginalClassName", nib.string("NSObject")),
                    ("NSInitializeWithInit", Value::Bool(true)),
                ];
                Value::New(Object::new("NSClassSwapper", values).named(name))
            }
            _ => view(&mut nib, &mut c, *t, module, &owner_name, None, true)?,
        };
        definitions.push(value);
    }

    // Outlets, in document order.
    for source in std::iter::once(owner).chain(tops.iter().copied()) {
        for list in children(source, "connections") {
            only_children(list, &["outlet"])?;
            for o in children(list, "outlet") {
                only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
                let connector = Object::new("NSNibOutletConnector", vec![
                    ("NSSource", reference(object_name(attr(source, "id")?))),
                    ("NSDestination", reference(object_name(attr(o, "destination")?))),
                    ("NSLabel", nib.string(attr(o, "property")?)),
                    ("NSChildControllerCreationSelectorName", Value::Nil),
                ]);
                let name = object_name(attr(o, "id")?);
                connections.push((name.clone(), Value::New(connector.named(name))));
            }
        }
    }

    // Every object is defined where the encoder first meets it (all are
    // shared): the definitions stand in the objects' list.
    let visible: Vec<Value> = c.visible.iter().map(|v| reference(v.clone())).collect();
    let visible = nib.array("NSMutableSet", visible);
    let mut oids = vec![reference(owner_name.clone())];
    let mut keys = Vec::new();
    let mut parents = Vec::new();
    let mut definitions = definitions.into_iter();
    let mut application_object = Some(application_object);
    for (object, parent) in &c.parents {
        oids.push(reference(object.clone()));
        parents.push(reference(parent.clone()));
        let definition = if *object == application_name {
            application_object.take().map(Value::New)
        } else if tops.iter().any(|t| t.attribute("id").map(object_name).as_ref() == Some(object)) {
            definitions.next()
        } else {
            None
        };
        keys.push(definition.unwrap_or_else(|| reference(object.clone())));
    }
    let mut connection_values = Vec::new();
    for (name, value) in connections {
        oids.push(reference(name));
        connection_values.push(value);
    }
    let numbers: Vec<Value> = (1..=oids.len() as i64).map(|i| nib.int_number(i)).collect();
    let data = Object::new("NSIBObjectData", vec![
        ("NSRoot", Value::New(root_object)),
        ("NSVisibleWindows", visible),
        ("NSConnections", nib.array("NSMutableArray", connection_values)),
        ("NSObjectsKeys", nib.array("NSArray", keys)),
        ("NSObjectsValues", nib.array("NSArray", parents)),
        ("NSOidsKeys", nib.array("NSArray", oids)),
        ("NSOidsValues", nib.array("NSArray", numbers)),
        ("NSAccessibilityConnectors", nib.array("NSMutableArray", Vec::new())),
        ("NSAccessibilityOidsKeys", nib.empty_array()),
        ("NSAccessibilityOidsValues", nib.empty_array()),
    ]);
    nib::archive_appkit(Object::new("NSObject", vec![("IB.objectdata", Value::New(data)), ("IB.systemFontUpdateVersion", Value::Int(1))]))
}
