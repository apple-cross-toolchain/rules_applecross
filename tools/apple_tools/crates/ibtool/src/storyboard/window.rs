//! A main window xib: its window, its custom objects (the app delegate)
//! and one container controller beside them, all top-level objects of its
//! one nib (docs/clean-room/ibtool.md, "Main window xibs"). The container's
//! children load their views from their own nibs (`nibName`), so the nib
//! holds no views but the window and the bars.

use super::*;

/// What Interface Builder alone keeps on a top-level object: where it sits
/// on the canvas, the bars it is shown with, its notes.
fn editor_only(c: Node) -> bool {
    match (c.tag_name().name(), c.attribute("key")) {
        ("point", Some("canvasLocation")) => true,
        ("metadata", None) => !c.has_children(),
        (_, Some(key)) => key.starts_with("simulated") && key.ends_with("Metrics"),
        _ => false,
    }
}

/// The nib's name for the document object `id`.
fn object_name(id: &str) -> String {
    format!("window xib {id}")
}

pub(super) fn compile(xml: &str, target: Option<u32>, module: Option<&str>, localizable: bool) -> Result<Vec<u8>, String> {
    let doc = Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    let objects_node = children(root, "objects").next().ok_or("a xib without objects")?;
    let mut nib = Nib::new(Settings { platform: Platform::Ios, color_matched: root.attribute("colorMatched") == Some("YES"), localizable, target });
    nib.xib = true;
    let mut top = vec![proxy(&mut nib, "IBFilesOwner", "owner"), proxy(&mut nib, "IBFirstResponder", "responder")];
    let mut objects = vec![reference("owner"), reference("responder")];
    // Outlets: (label, source, destination id), the top-level objects'
    // and the controllers'.
    let mut outlets: Vec<(String, String, String)> = Vec::new();
    let mut controller_outlets: Vec<(String, String, String)> = Vec::new();
    let mut parts: Vec<Value> = Vec::new();
    let mut seen = (false, false, false);
    for o in objects_node.children().filter(|c| c.is_element()) {
        match (o.tag_name().name(), o.attribute("placeholderIdentifier")) {
            ("placeholder", Some("IBFilesOwner")) if !seen.0 => {
                seen.0 = true;
                only_attributes(o, &[("placeholderIdentifier", None), ("id", None), ("userLabel", None), ("customClass", None)])?;
                connections_of(o, "owner", &mut outlets)?;
            }
            ("placeholder", Some("IBFirstResponder")) if !seen.1 => {
                seen.1 = true;
                if o.has_children() {
                    return fail("a first responder with connections is not supported by this ibtool");
                }
            }
            ("customObject", _) => {
                only_attributes(o, &[("id", None), ("userLabel", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None)])?;
                let name = object_name(attr(o, "id")?);
                let class = runtime_class_name(o, module, "UICustomObject")?;
                if o.attribute("customClass").is_none() {
                    return fail("a custom object without a class is not supported by this ibtool");
                }
                let values = vec![("UIClassName", nib.string(&class)), ("UIOriginalClassName", nib.string("UICustomObject"))];
                connections_of(o, &name, &mut outlets)?;
                top.push(Value::New(Object::new("UIClassSwapper", values).named(name.clone())));
                objects.push(reference(name));
            }
            ("window", _) => {
                let name = object_name(attr(o, "id")?);
                top.push(window(&mut nib, o, &name)?);
                objects.push(reference(name));
            }
            ("navigationController" | "tabBarController" | "splitViewController", _) if !seen.2 => {
                seen.2 = true;
                let name = object_name(attr(o, "id")?);
                let (value, mut own_parts) = container(&mut nib, module, o, &name, None, &mut controller_outlets)?;
                top.push(value);
                objects.push(reference(name));
                parts.append(&mut own_parts);
            }
            (tag, _) => return fail(format!("a main window xib's top-level {tag} is not supported by this ibtool")),
        }
    }
    if !seen.0 {
        return fail("a xib without a File's Owner is not supported by this ibtool");
    }
    objects.extend(parts);
    // The controllers' outlets first, in document order, then the
    // top-level objects', in byte order of their labels, whatever the
    // labels (four corpus xibs, three probes); two of one label aren't
    // established.
    outlets.sort_by(|a, b| a.0.cmp(&b.0));
    if outlets.windows(2).any(|w| w[0].0 == w[1].0) || controller_outlets.len() > 1 {
        return fail("two outlets of one name, or several of controllers, in a main window xib are not supported by this ibtool");
    }
    outlets.splice(0..0, controller_outlets);
    let named: Vec<String> = objects.iter().filter_map(|o| if let Value::Ref(r) = o { Some(r.clone()) } else { None }).collect();
    let mut connections = Vec::new();
    for (label, source, destination) in &outlets {
        let destination = object_name(destination);
        if !named.contains(&destination) {
            return fail("an outlet to anything but the nib's own objects is not supported by this ibtool");
        }
        connections.push(outlet(&mut nib, label, source, &destination));
    }
    nib::archive(root_object_full(&mut nib, top, objects, connections, Vec::new(), Vec::new()))
}

/// An object's outlets, from `source`.
fn connections_of(o: Node, source: &str, outlets: &mut Vec<(String, String, String)>) -> Result<(), String> {
    for c in children(o, "connections") {
        only_children(c, &["outlet"])?;
        for out in children(c, "outlet") {
            only_attributes(out, &[("property", None), ("destination", None), ("id", None)])?;
            outlets.push((attr(out, "property")?.to_string(), source.to_string(), attr(out, "destination")?.to_string()));
        }
    }
    Ok(())
}

/// The window: a view of no subviews, resizing to fill the screen.
fn window(nib: &mut Nib, w: Node, name: &str) -> Result<Value, String> {
    only_attributes(w, &[("opaque", Some("NO")), ("clearsContextBeforeDrawing", Some("NO")), ("contentMode", Some("scaleToFill")), ("id", None)])?;
    for c in w.children().filter(|c| c.is_element() && !editor_only(*c)) {
        match (c.tag_name().name(), c.attribute("key")) {
            ("rect", Some("frame")) | ("autoresizingMask", Some("autoresizingMask")) | ("color", Some("backgroundColor")) => {}
            (tag, _) => return fail(format!("<{tag}> inside a main window xib's window is not supported by this ibtool")),
        }
    }
    let mut values = Vec::new();
    geometry(&mut values, &frame(w)?);
    if let Some(c) = child(w, "color", "backgroundColor") {
        values.push(("UIBackgroundColor", color(nib, c)?));
    }
    values.push(("UIDeepDrawRect", Value::Bool(true)));
    if w.attribute("opaque").is_none() {
        values.push(("UIOpaque", Value::Bool(true)));
    }
    values.push(("UIAutoresizeSubviews", Value::Bool(true)));
    let mask = match children(w, "autoresizingMask").next() {
        Some(m) => autoresizing_mask(m)?,
        None => 0,
    };
    if mask != 0 {
        values.push(("UIAutoresizingMask", Value::Int(mask)));
    }
    values.push(("UIShouldNotAdaptToMaterials", Value::Bool(true)));
    if w.attribute("clearsContextBeforeDrawing").is_some() {
        values.push(("UIClearsContextBeforeDrawing", Value::Bool(false)));
    }
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    values.push(("UIResizesToFullScreen", Value::Bool(true)));
    Ok(Value::New(Object::new("UIWindow", values).named(name.to_string())))
}

/// `UIKeyEdgesForExtendedLayout` when the document gives it.
fn edges(values: &mut Vec<(&'static str, Value)>, c: Node) -> Result<(), String> {
    if let Some(e) = extended_edges(c)? {
        values.push(("UIKeyEdgesForExtendedLayout", Value::Int(e)));
    }
    Ok(())
}

/// A container controller (navigation, tab bar or split view) named
/// `name`, held by `parent` if any: its object, and what the objects list
/// after it, breadth first as a storyboard's container nib lists them.
fn container(nib: &mut Nib, module: Option<&str>, c: Node, name: &str, parent: Option<&str>, outlets: &mut Vec<(String, String, String)>) -> Result<(Value, Vec<Value>), String> {
    only_attributes(c, &[("id", None), ("key", None)])?;
    if c.attribute("key").is_some() != parent.is_some() {
        return fail("a keyed controller outside a split view controller is not supported by this ibtool");
    }
    let mut values = Vec::new();
    let mut listed = Vec::new();
    match c.tag_name().name() {
        "navigationController" => {
            for k in c.children().filter(|k| k.is_element() && !editor_only(*k)) {
                if !matches!(k.tag_name().name(), "extendedEdge" | "navigationBar" | "viewControllers") {
                    return fail(format!("<{}> inside a main window xib's navigation controller is not supported by this ibtool", k.tag_name().name()));
                }
            }
            let kids: Vec<Node> = children(c, "viewControllers").flat_map(|v| v.children().filter(|k| k.is_element())).collect();
            let [root] = kids[..] else { return fail("a main window xib's navigation controller without one root is not supported by this ibtool") };
            let root_name = object_name(attr(root, "id")?);
            let bar_name = format!("{name} bar");
            let (root_value, root_parts, item) = child_controller(nib, module, root, &root_name, name, Some(&bar_name), outlets)?;
            if let Some(p) = parent {
                values.push(("UIParentViewController", reference(p)));
            }
            values.push(("UIChildViewControllers", nib.array("NSMutableArray", vec![root_value])));
            edges(&mut values, c)?;
            values.push(("UIViewControllers", nib.array("NSMutableArray", vec![reference(&root_name)])));
            let implied = format!("{root_name} implied navigation item");
            values.push(("UINavigationBar", navigation_bar(nib, c, &bar_name, Some(item.as_deref().unwrap_or(&implied)), name)?));
            if item.is_none() {
                return fail("a main window xib's navigation root without a navigation item is not supported by this ibtool");
            }
            listed.push(reference(&bar_name));
            listed.push(reference(&root_name));
            listed.extend(root_parts);
            Ok((Value::New(Object::new("UINavigationController", values).named(name.to_string())), listed))
        }
        "tabBarController" => {
            if parent.is_some() {
                return fail("a tab bar controller in a split view controller is not supported by this ibtool");
            }
            for k in c.children().filter(|k| k.is_element() && !editor_only(*k)) {
                if !matches!(k.tag_name().name(), "extendedEdge" | "tabBar" | "viewControllers") {
                    return fail(format!("<{}> inside a main window xib's tab bar controller is not supported by this ibtool", k.tag_name().name()));
                }
            }
            let kids: Vec<Node> = children(c, "viewControllers").flat_map(|v| v.children().filter(|k| k.is_element())).collect();
            let mut child_values = Vec::new();
            let mut child_names = Vec::new();
            let mut items = Vec::new();
            let mut child_parts = Vec::new();
            for k in &kids {
                let child_name = object_name(attr(*k, "id")?);
                let (value, own_parts, _) = child_controller(nib, module, *k, &child_name, name, None, outlets)?;
                if child(*k, "tabBarItem", "tabBarItem").is_none() {
                    return fail("a main window xib's tab bar controller's child without a tab bar item is not supported by this ibtool");
                }
                items.push(reference(format!("{child_name} tab bar item")));
                child_values.push(value);
                child_names.push(child_name);
                child_parts.extend(own_parts);
            }
            values.push(("UIChildViewControllers", nib.array("NSMutableArray", child_values)));
            edges(&mut values, c)?;
            values.push(("UITabBar", tab::tab_bar(nib, c, items, name)?));
            let refs = || child_names.iter().map(reference).collect::<Vec<Value>>();
            values.push(("UICustomizableViewControllers", nib.array("NSMutableArray", refs())));
            values.push(("UIViewControllers", nib.array("NSMutableArray", refs())));
            listed.push(reference("tab bar"));
            listed.extend(child_names.iter().map(reference));
            listed.extend(child_parts);
            Ok((Value::New(Object::new("UITabBarController", values).named(name.to_string())), listed))
        }
        "splitViewController" => {
            if parent.is_some() {
                return fail("a split view controller in another is not supported by this ibtool");
            }
            let mut kids = Vec::new();
            for k in c.children().filter(|k| k.is_element() && !editor_only(*k)) {
                match (k.tag_name().name(), k.attribute("key")) {
                    ("extendedEdge", _) => {}
                    ("toolbarItems", _) if !k.has_children() => {}
                    (_, Some(key @ ("masterViewController" | "detailViewController"))) => kids.push((key, k)),
                    (tag, _) => return fail(format!("<{tag}> inside a main window xib's split view controller is not supported by this ibtool")),
                }
            }
            // Archived master first; listed by relationship, detail first,
            // each child's parts after both, as a storyboard's split view
            // controller's (a corpus xib).
            kids.sort_by_key(|k| k.0 != "masterViewController");
            let mut child_values = Vec::new();
            let mut listing: Vec<(&str, String, Vec<Value>)> = Vec::new();
            for (key, k) in &kids {
                let child_name = object_name(attr(*k, "id")?);
                let (value, own_parts) = if k.has_tag_name("navigationController") {
                    let (value, own) = container(nib, module, *k, &child_name, Some(name), outlets)?;
                    (value, own)
                } else {
                    let (value, own, _) = child_controller(nib, module, *k, &child_name, name, None, outlets)?;
                    (value, own)
                };
                child_values.push(value);
                listing.push((key, child_name, own_parts));
            }
            values.push(("UIChildViewControllers", nib.array("NSMutableArray", child_values)));
            edges(&mut values, c)?;
            values.push(("UISplitViewControllerPrimaryEdge", Value::Int(0)));
            listing.sort_by_key(|l| l.0);
            listed.extend(listing.iter().map(|l| reference(&l.1)));
            for (_, _, own) in listing {
                listed.extend(own);
            }
            Ok((Value::New(Object::new("UISplitViewController", values).named(name.to_string())), listed))
        }
        other => fail(format!("a main window xib's {other} is not supported by this ibtool")),
    }
}

/// A container's child, a view or table view controller loading its view
/// from its own nib: its object, what the objects list for it (its tab bar
/// item, its navigation item and that item's bar button items), and its
/// navigation item's name. `bar` is the navigation bar its item refers to.
#[allow(clippy::type_complexity)]
fn child_controller(
    nib: &mut Nib, module: Option<&str>, k: Node, name: &str, parent: &str, bar: Option<&str>, outlets: &mut Vec<(String, String, String)>,
) -> Result<(Value, Vec<Value>, Option<String>), String> {
    let table = k.has_tag_name("tableViewController");
    if !table && !k.has_tag_name("viewController") {
        return fail(format!("a main window xib's {} child is not supported by this ibtool", k.tag_name().name()));
    }
    only_attributes(
        k,
        &[
            ("id", None), ("key", None), ("nibName", None), ("title", None), ("userLabel", None), ("customClass", None), ("customModule", None),
            ("customModuleProvider", None), ("clearsSelectionOnViewWillAppear", Some("NO")),
        ],
    )?;
    let nib_name = attr(k, "nibName").map_err(|_| "a main window xib's child controller without a nib name is not supported by this ibtool".to_string())?;
    for c in k.children().filter(|c| c.is_element() && !editor_only(*c)) {
        match c.tag_name().name() {
            "extendedEdge" | "navigationItem" | "tabBarItem" | "connections" => {}
            "toolbarItems" if !c.has_children() => {}
            other => return fail(format!("<{other}> inside a main window xib's child controller is not supported by this ibtool")),
        }
    }
    connections_of(k, name, outlets)?;
    let id = attr(k, "id")?;
    let mut values = Vec::new();
    let mut listed = Vec::new();
    if let Some(title) = k.attribute("title") {
        values.push(("UITitle", nib.text(id, "title", title)));
    }
    if child(k, "tabBarItem", "tabBarItem").is_some() {
        let item_name = format!("{name} tab bar item");
        values.push(("UITabBarItem", tab::tab_bar_item(nib, k, &item_name)?.0));
        listed.push(reference(&item_name));
    }
    let mut item_name = None;
    if let Some(n) = child(k, "navigationItem", "navigationItem") {
        let Some(bar) = bar else { return fail("a navigation item outside a navigation controller is not supported by this ibtool") };
        let own = format!("{name} navigation item");
        let parts = navigation_item(nib, n, &own, id)?;
        if !parts.actions.is_empty() || !parts.segues.is_empty() || !parts.responder_actions.is_empty() || !parts.pairs.is_empty() {
            return fail("a main window xib's navigation item's actions, segues or runtime attributes are not supported by this ibtool");
        }
        let mut value = parts.value;
        if let Value::New(o) = &mut value {
            o.values.push(("UINavigationBar", reference(bar)));
        }
        values.push(("UINavigationItem", value));
        listed.push(reference(&own));
        listed.extend(parts.bar_items.iter().map(reference));
        item_name = Some(own);
    }
    values.push(("UINibName", nib.string(nib_name)));
    values.push(("UIParentViewController", reference(parent)));
    edges(&mut values, k)?;
    if k.attribute("clearsSelectionOnViewWillAppear").is_some() {
        values.push(("UIClearsSelectionOnViewWillAppear", Value::Bool(false)));
    }
    let class = if table { "UITableViewController" } else { "UIViewController" };
    let object = if k.attribute("customClass").is_some() {
        let runtime = runtime_class_name(k, module, class)?;
        values.push(("UIClassName", nib.string(&runtime)));
        values.push(("UIOriginalClassName", nib.string(class)));
        Object::new("UIClassSwapper", values)
    } else {
        Object::new(class, values)
    };
    Ok((Value::New(object.named(name.to_string())), listed, item_name))
}
