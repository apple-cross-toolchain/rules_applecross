//! Table view controllers' tables and their prototype cells
//! (docs/clean-room/ibtool.md, "Table view controllers").

use super::*;

/// A table's background: a plain table's white, as a system background
/// or any opaque grey white, is its own `tableBackgroundColor`.
fn table_background(nib: &mut Nib, n: Node, plain: bool) -> Result<Value, String> {
    let white = n.attribute("systemColor") == Some("systemBackgroundColor") || opaque_grey_white(n)?;
    if plain && white {
        return Ok(nib.system_color("tableBackgroundColor", &SystemColor::Gray([1.0, 1.0])));
    }
    color(nib, n)
}

/// A height attribute, or its default when the document leaves it out.
fn height(n: Node, name: &str, default: Option<f32>) -> Result<f32, String> {
    match n.attribute(name) {
        Some(h) => Ok(number(h)? as f32),
        None => default.ok_or_else(|| format!("a table without {name} is not supported by this ibtool")),
    }
}

/// A table view controller's view nib: its table, the root view, holding
/// its prototype cells' nibs. Returns the nib's root, whether it uses the
/// controller's placeholder, and the ids of the segues its cells trigger.
pub(super) fn table_view_nib(
    platform: Platform, color_matched: bool, localizable: bool, module: Option<&str>, vc_id: &str, table: Node, f: &Frame,
    vc_outlets: &[(String, String)],
) -> Result<(Object, bool, Vec<String>), String> {
    only_attributes(table, &[
        ("key", Some("view")), ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("clearsContextBeforeDrawing", Some("NO")),
        ("contentMode", Some("scaleToFill")), ("alwaysBounceVertical", None), ("showsHorizontalScrollIndicator", Some("NO")),
        ("showsVerticalScrollIndicator", Some("NO")),
        ("keyboardDismissMode", None), ("dataMode", Some("prototypes")), ("style", None), ("separatorStyle", None),
        ("allowsSelection", Some("NO")), ("allowsMultipleSelection", Some("YES")), ("allowsSelectionDuringEditing", Some("YES")),
        ("rowHeight", None), ("estimatedRowHeight", None), ("sectionHeaderHeight", None), ("estimatedSectionHeaderHeight", None),
        ("sectionFooterHeight", None), ("estimatedSectionFooterHeight", None), ("id", None),
    ])?;
    only_children(table, &["rect", "autoresizingMask", "color", "prototypes", "connections"])?;
    let selection_keys = ["allowsSelection", "allowsMultipleSelection", "allowsSelectionDuringEditing"];
    if selection_keys.iter().filter(|k| table.attribute(**k).is_some()).count() > 1 {
        return fail("more than one of a table's selection settings is not supported by this ibtool");
    }
    for c in children(table, "color") {
        if !matches!(c.attribute("key"), Some("backgroundColor" | "sectionIndexBackgroundColor" | "separatorColor")) {
            return fail(format!("a table's {} colour is not supported by this ibtool", c.attribute("key").unwrap_or("")));
        }
    }
    if child(table, "color", "sectionIndexBackgroundColor").is_some() && child(table, "color", "separatorColor").is_some() {
        return fail("a table with both a section index background and a separator colour is not supported by this ibtool");
    }
    let table_id = attr(table, "id")?;
    let (style, ui_style) = match attr(table, "style")? {
        "plain" => (0, 0),
        "grouped" => (1, 1),
        "insetGrouped" => (2, 1),
        other => return fail(format!("table style {other} is not supported by this ibtool")),
    };
    let plain = style == 0;

    // The table's outlets, to its controller.
    let mut outlets: Vec<(String, &str, String)> = vc_outlets.iter().map(|(l, d)| (l.clone(), "upstream", d.clone())).collect();
    for c in children(table, "connections") {
        only_children(c, &["outlet"])?;
        for o in children(c, "outlet") {
            only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
            if attr(o, "destination")? != vc_id {
                return fail("a table's outlets to anything but its controller are not supported by this ibtool");
            }
            outlets.push((attr(o, "property")?.to_string(), "table", "upstream".to_string()));
        }
    }
    let root_name = view_name(table_id);
    for (_, destination) in vc_outlets {
        if *destination != root_name {
            return fail("a table view controller's outlets into anything but its table are not supported by this ibtool");
        }
    }
    let upstream_used = !outlets.is_empty();

    // Each prototype with a reuse identifier is a nib of its own, in byte
    // order of the identifiers; one without is left out. A cell's segue
    // is a placeholder of the table's nib, after the controller's, in
    // document order.
    let mut cells: Vec<(&str, Node, Option<String>)> = Vec::new();
    let mut segues = Vec::new();
    for p in children(table, "prototypes") {
        only_children(p, &["tableViewCell"])?;
        for c in children(p, "tableViewCell") {
            let segue = children(c, "connections").flat_map(|cs| children(cs, "segue")).map(|sg| attr(sg, "id")).next().transpose()?;
            if let Some(reuse) = c.attribute("reuseIdentifier") {
                if cells.iter().any(|(r, ..)| *r == reuse) {
                    return fail(format!("two prototypes reused as {reuse:?} are not supported by this ibtool"));
                }
                let placeholder = segue.map(|id| {
                    segues.push(id.to_string());
                    format!("upstream segue {}", segues.len() - 1)
                });
                cells.push((reuse, c, placeholder));
            } else if segue.is_some() {
                return fail("a segue from a prototype without a reuse identifier is not supported by this ibtool");
            }
        }
    }
    cells.sort_by(|a, b| a.0.cmp(b.0));

    let mut nib = Nib::new(platform, color_matched, localizable);
    let owner = proxy(&mut nib, "IBFilesOwner", "owner");
    let responder = proxy(&mut nib, "IBFirstResponder", "responder");
    let mut top = vec![owner, responder];
    let mut objects = vec![reference("owner"), reference("responder")];
    if upstream_used {
        top.push(proxy(&mut nib, "UpstreamPlaceholder-1", "upstream"));
        objects.push(reference("upstream"));
    }
    for i in 0..segues.len() {
        let name = format!("upstream segue {i}");
        top.push(proxy(&mut nib, &format!("UpstreamPlaceholder-{}", i + 1 + usize::from(upstream_used)), &name));
        objects.push(reference(name));
    }

    let mut values = Vec::new();
    geometry(&mut values, f);
    if let Some(c) = child(table, "color", "backgroundColor") {
        values.push(("UIBackgroundColor", table_background(&mut nib, c, plain)?));
    }
    values.push(("UIDeepDrawRect", Value::Bool(true)));
    if table.attribute("opaque").is_none() {
        values.push(("UIOpaque", Value::Bool(true)));
    }
    values.push(("UIAutoresizeSubviews", Value::Bool(true)));
    let mask = match children(table, "autoresizingMask").next() {
        Some(m) => autoresizing_mask(m)?,
        None => 0,
    };
    values.push(("UIAutoresizingMask", Value::Int(mask)));
    if table.attribute("clearsContextBeforeDrawing").is_some() {
        values.push(("UIClearsContextBeforeDrawing", Value::Bool(false)));
    }
    if table.attribute("clipsSubviews").is_some() {
        values.push(("UIClipsToBounds", Value::Bool(true)));
    }
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    values.push(("UIScrollViewIndicatorInsetAdjustmentBehavior", Value::Int(1)));
    if table.attribute("showsHorizontalScrollIndicator").is_some() {
        values.push(("UIShowsHorizontalScrollIndicator", Value::Bool(false)));
    }
    values.push(("UIBouncesZoom", Value::Bool(true)));
    match table.attribute("alwaysBounceVertical") {
        Some("YES") => values.push(("UIAlwaysBounceVertical", Value::Bool(true))),
        Some("NO") | None => {}
        Some(other) => return fail(format!("alwaysBounceVertical=\"{other}\" is not supported by this ibtool")),
    }
    match table.attribute("keyboardDismissMode") {
        None => {}
        Some("onDrag") => values.push(("UIKeyboardDismissMode", Value::Int(1))),
        Some("interactive") => values.push(("UIKeyboardDismissMode", Value::Int(2))),
        Some(other) => return fail(format!("keyboardDismissMode=\"{other}\" is not supported by this ibtool")),
    }
    values.push(("UIScrollViewContentInsetAdjustmentBehavior", Value::Int(0)));
    values.push(("UITableViewStyle", Value::Int(style)));
    values.push(("UIStyle", Value::Int(ui_style)));
    // The legacy keys are left out for none (0), the iOS 9 one for the
    // single line (1).
    let separator = match attr(table, "separatorStyle")? {
        "default" | "singleLine" => 1,
        "none" => 0,
        "singleLineEtched" => 2,
        other => return fail(format!("separator style {other} is not supported by this ibtool")),
    };
    if separator != 0 {
        values.push(("UISeparatorStyle", Value::Int(separator)));
        values.push(("UISeparatorStyleIOS5AndLater", Value::Int(separator)));
    }
    if separator != 1 {
        values.push(("UISeparatorStyleIOS9AndLater", Value::Int(separator)));
    }
    values.push(("UIPrefetchingEnabled", Value::Bool(true)));
    // Left out, a plain table's section heights are 22, the estimates 0.
    let section_default = if plain { Some(22.0) } else { None };
    values.push(("UIRowHeight", Value::Float(height(table, "rowHeight", Some(44.0))?)));
    values.push(("UISectionHeaderHeight", Value::Float(height(table, "sectionHeaderHeight", section_default)?)));
    values.push(("UISectionFooterHeight", Value::Float(height(table, "sectionFooterHeight", section_default)?)));
    values.push(("UIEstimatedRowHeight", Value::Float(height(table, "estimatedRowHeight", Some(0.0))?)));
    values.push(("UIEstimatedSectionHeaderHeight", Value::Float(height(table, "estimatedSectionHeaderHeight", Some(0.0))?)));
    values.push(("UIEstimatedSectionFooterHeight", Value::Float(height(table, "estimatedSectionFooterHeight", Some(0.0))?)));
    values.push(("UIFillerRowHeight", Value::Double(0.0)));
    values.push(("UISectionHeaderTopPadding", Value::Double(-1.0)));
    if table.attribute("allowsSelection").is_some() {
        values.push(("UIAllowsSelection", Value::Bool(false)));
    }
    if table.attribute("allowsMultipleSelection").is_some() {
        values.push(("UIAllowsMultipleSelection", Value::Bool(true)));
    }
    if table.attribute("allowsSelectionDuringEditing").is_some() {
        values.push(("UIAllowsSelectionDuringEditing", Value::Bool(true)));
    }
    values.push(("UIInsetsContentViewsToSafeArea", Value::Bool(true)));
    if let Some(c) = child(table, "color", "separatorColor") {
        values.push(("UISeparatorColor", color(&mut nib, c)?));
    }

    if !cells.is_empty() {
        let mut nibs = Vec::new();
        for (reuse, c, placeholder) in &cells {
            let bytes = cell_nib(platform, color_matched, localizable, module, *c, placeholder.is_some())?;
            nibs.push(nib.string(reuse));
            let data = Value::New(Object::new("NSData", vec![("NS.bytes", Value::Data(bytes))]));
            nibs.push(Value::New(Object::new("UINib", vec![("captureEnclosingNIBBundleOnDecode", Value::Bool(true)), ("archiveData", data)])));
        }
        values.push(("UITableViewCellPrototypeNibs", nib.array("NSMutableDictionary", nibs)));
        // A cell's external objects: its own nib's placeholder for its
        // segue's.
        let mut external = Vec::new();
        for (reuse, _, placeholder) in &cells {
            external.push(nib.string(reuse));
            let objects = match placeholder {
                Some(p) => vec![nib.string("UpstreamPlaceholder-1"), reference(p.clone())],
                None => Vec::new(),
            };
            external.push(nib.array("NSMutableDictionary", objects));
        }
        values.push(("UITableViewCellPrototypeNibExternalObjects", nib.array("NSMutableDictionary", external)));
    }
    if let Some(c) = child(table, "color", "sectionIndexBackgroundColor") {
        values.push(("UISectionIndexBackgroundColor", color(&mut nib, c)?));
    }
    values.push(("UISeparatorInsetReference", Value::Int(0)));
    top.push(Value::New(Object::new("UITableView", values).named(root_name.clone())));
    objects.push(reference(root_name.clone()));

    // Outlets, `view` among them, in byte order of their names.
    outlets.push(("view".to_string(), "owner", root_name.clone()));
    outlets.sort_by(|a, b| a.0.cmp(&b.0));
    let mut connections = Vec::new();
    for (label, source, destination) in &outlets {
        let source = if *source == "table" { root_name.as_str() } else { source };
        connections.push(outlet(&mut nib, label, source, destination));
    }
    Ok((root_object(&mut nib, top, objects, connections), upstream_used, segues))
}

/// A prototype cell's nib: the cell, its content view and the separator
/// Xcode lays out at its foot, and the content view's subviews.
fn cell_nib(platform: Platform, color_matched: bool, localizable: bool, module: Option<&str>, cell: Node, has_segue: bool) -> Result<Vec<u8>, String> {
    only_attributes(cell, &[
        ("clipsSubviews", Some("YES")), ("contentMode", Some("scaleToFill")), ("preservesSuperviewLayoutMargins", Some("YES")),
        ("selectionStyle", None), ("hidesAccessoryWhenEditing", Some("NO")), ("indentationLevel", None), ("indentationWidth", None),
        ("accessoryType", None), ("reuseIdentifier", None), ("rowHeight", None), ("customClass", None), ("customModule", None),
        ("customModuleProvider", None), ("userLabel", None), ("id", None),
    ])?;
    only_children(cell, &["rect", "autoresizingMask", "tableViewCellContentView", "color", "connections"])?;
    for m in children(cell, "autoresizingMask") {
        if autoresizing_mask(m)? != 0 || m.has_children() {
            return fail("a cell with an autoresizing mask is not supported by this ibtool");
        }
    }
    for c in children(cell, "color") {
        if c.attribute("key") != Some("backgroundColor") {
            return fail("a cell's colours other than its background are not supported by this ibtool");
        }
    }
    let cell_id = attr(cell, "id")?;
    let cell_name = view_name(cell_id);
    let content = child(cell, "tableViewCellContentView", "contentView").ok_or("a cell without a content view")?;
    only_attributes(content, &[
        ("key", Some("contentView")), ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("multipleTouchEnabled", Some("YES")),
        ("contentMode", Some("center")), ("preservesSuperviewLayoutMargins", Some("YES")), ("insetsLayoutMarginsFromSafeArea", Some("NO")),
        ("tableViewCell", None), ("id", None),
    ])?;
    for required in ["opaque", "multipleTouchEnabled", "contentMode"] {
        if content.attribute(required).is_none() {
            return fail(format!("a cell's content view without {required} is not supported by this ibtool"));
        }
    }
    only_children(content, &["rect", "autoresizingMask", "subviews", "constraints"])?;
    for m in children(content, "autoresizingMask") {
        if autoresizing_mask(m)? != 0 || m.has_children() {
            return fail("a cell's content view with an autoresizing mask is not supported by this ibtool");
        }
    }
    let content_id = attr(content, "id")?;
    let content_name = view_name(content_id);

    let mut nib = Nib::new(platform, color_matched, localizable);
    let owner = proxy(&mut nib, "IBFilesOwner", "owner");
    let responder = proxy(&mut nib, "IBFirstResponder", "responder");
    // A cell's segue is its placeholder's.
    let upstream = if has_segue { Some(proxy(&mut nib, "UpstreamPlaceholder-1", "upstream")) } else { None };

    // The content view and its subviews, compiled as a container's are.
    let mut scene = Scene {
        vc_id: "",
        module,
        own_constraints: Vec::new(),
        constrained: constrained_views(content),
        subviews: Vec::new(),
        parent: Some(content_name.clone()),
        actions: Vec::new(),
        segues: Vec::new(),
        containers: Vec::new(),
        control_outlets: Vec::new(),
        guides: Vec::new(),
    };
    let cf = frame(content)?;
    let mut cv = Vec::new();
    geometry(&mut cv, &cf);
    let mut subviews = Vec::new();
    for s in children(content, "subviews") {
        for c in s.children().filter(|c| c.is_element()) {
            subviews.push(subview(&mut nib, &mut scene, c)?);
        }
    }
    if !scene.actions.is_empty() || !scene.segues.is_empty() || !scene.control_outlets.is_empty() {
        return fail("actions, outlets and segues in a cell are not supported by this ibtool");
    }
    if !subviews.is_empty() {
        cv.push(("UISubviews", nib.array("NSMutableArray", subviews)));
    }
    cv.push(("UIDeepDrawRect", Value::Bool(true)));
    cv.push(("UIMultipleTouchEnabled", Value::Bool(true)));
    cv.push(("UIAutoresizeSubviews", Value::Bool(true)));
    cv.push(("UIContentMode", Value::Int(4)));
    if content.attribute("clipsSubviews").is_some() {
        cv.push(("UIClipsToBounds", Value::Bool(true)));
    }
    if let Some(own) = own_constraints(&mut nib, &mut scene, content, &content_name)? {
        cv.push(("UIViewAutolayoutConstraints", own));
    }
    if content.attribute("preservesSuperviewLayoutMargins").is_some() {
        cv.push(("UIViewPreservesSuperviewMargins", Value::Bool(true)));
        cv.push(("UIViewPreservesSpecificSuperviewMargins", Value::Int(15)));
    }
    if content.attribute("insetsLayoutMarginsFromSafeArea").is_some() {
        cv.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0)));
    }
    cv.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut cv);
    let content_view = Value::New(Object::new("UITableViewCellContentView", cv).with_fallback("UIView").named(content_name.clone()));

    // The separator: a point high at the cell's foot, from 8 points in.
    let f = frame(cell)?;
    let mut sv = Vec::new();
    geometry(&mut sv, &Frame { x: 8.0, y: f.h - 1.0, w: f.w - 8.0, h: 1.0 });
    sv.extend([("UIDeepDrawRect", Value::Bool(true)), ("UIOpaque", Value::Bool(true)), ("UIAutoresizeSubviews", Value::Bool(true))]);
    sv.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut sv);
    let separator = Value::New(Object::new("_UITableViewCellSeparatorView", sv));

    let selection = match cell.attribute("selectionStyle") {
        None | Some("default") => None,
        Some("none") => Some(0),
        Some("blue") => Some(1),
        Some("gray") => Some(2),
        Some(other) => return fail(format!("selection style {other} is not supported by this ibtool")),
    };
    let accessory = match cell.attribute("accessoryType") {
        None | Some("none") => None,
        Some("disclosureIndicator") => Some(1),
        Some("checkmark") => Some(3),
        Some("detailButton") => Some(4),
        Some(other) => return fail(format!("accessory type {other} is not supported by this ibtool")),
    };
    let background = child(cell, "color", "backgroundColor");

    let mut values = Vec::new();
    geometry(&mut values, &f);
    values.push(("UISubviews", nib.array("NSMutableArray", vec![content_view, separator])));
    // An explicit background, or for a cell that selects other than by
    // default, the plain cell background and a background view.
    let system_background = background.is_none() && selection.is_some();
    if let Some(c) = background {
        values.push(("UIBackgroundColor", color(&mut nib, c)?));
    } else if system_background {
        values.push(("UIBackgroundColor", nib.system_color("tableCellPlainBackgroundColor", &SystemColor::Gray([1.0, 1.0]))));
        values.push(("UIDidEncodeCompatibilityBackgroundColor", Value::Bool(true)));
    }
    values.extend([("UIDeepDrawRect", Value::Bool(true)), ("UIOpaque", Value::Bool(true)), ("UIAutoresizeSubviews", Value::Bool(true))]);
    if cell.attribute("clipsSubviews").is_some() {
        values.push(("UIClipsToBounds", Value::Bool(true)));
    }
    if cell.attribute("preservesSuperviewLayoutMargins").is_some() {
        values.push(("UIViewPreservesSuperviewMargins", Value::Bool(true)));
        values.push(("UIViewPreservesSpecificSuperviewMargins", Value::Int(15)));
    }
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    values.push(("UIReuseIdentifier", nib.string(attr(cell, "reuseIdentifier")?)));
    if let Some(level) = cell.attribute("indentationLevel") {
        let level: i64 = level.parse().map_err(|_| format!("not a number: {level}"))?;
        if level != 0 {
            values.push(("UIIndentationLevel", Value::Int(level)));
        }
    }
    if let Some(s) = selection {
        values.push(("UISelectionStyle", Value::Int(s)));
    }
    if let Some(a) = accessory {
        values.push(("UIAccessoryType", Value::Int(a)));
    }
    values.push(("UIIndentationWidth", Value::Float(number(attr(cell, "indentationWidth")?)? as f32)));
    if background.is_some() {
        values.push(("UITableCellBackgroundColorSet", Value::Bool(true)));
    }
    if system_background {
        values.push(("UISystemBackgroundView", system_background_view(&mut nib, &f)));
    } else {
        values.push(("UISystemBackgroundView", Value::Nil));
    }
    values.push(("UIContentConfigurationView", Value::Nil));
    values.push(("UIContentView", reference(content_name.clone())));
    values.push(("UITextLabel", Value::Nil));
    values.push(("UIDetailTextLabel", Value::Nil));
    values.push(("UIImageView", Value::Nil));
    let object = if cell.attribute("customClass").is_some() {
        let class = runtime_class_name(cell, module, "UITableViewCell")?;
        values.push(("UIClassName", nib.string(&class)));
        values.push(("UIOriginalClassName", nib.string("UITableViewCell")));
        Object::new("UIClassSwapper", values)
    } else {
        Object::new("UITableViewCell", values)
    };
    let mut top = vec![owner, responder];
    top.extend(upstream);
    top.push(Value::New(object.named(cell_name.clone())));

    // The objects: the cell, its content view and the content view's
    // subviews; then, breadth first, each view's constraints and its
    // subviews, the content view's already listed.
    let mut objects = vec![reference("owner"), reference("responder")];
    if has_segue {
        objects.push(reference("upstream"));
    }
    objects.extend([reference(cell_name.clone()), reference(content_name.clone())]);
    let content_parent = Some(content_name.clone());
    objects.extend(scene.subviews.iter().filter(|s| s.0 == content_parent).map(|s| reference(s.1.clone())));
    let mut queue: std::collections::VecDeque<&String> = [&content_name].into();
    while let Some(v) = queue.pop_front() {
        objects.extend(scene.own_constraints.iter().filter(|c| &c.0 == v).map(|c| reference(c.1.clone())));
        for (_, s) in scene.subviews.iter().filter(|s| s.0.as_ref() == Some(v)) {
            if *v != content_name {
                objects.push(reference(s.clone()));
            }
            queue.push_back(s);
        }
    }

    // The cell's outlets into its content and to its segue's placeholder,
    // in byte order of their names.
    let mut outlets = Vec::new();
    if has_segue {
        outlets.push(("selectionSegueTemplate", "upstream".to_string()));
    }
    for c in children(cell, "connections") {
        only_children(c, &["outlet", "segue"])?;
        if children(c, "segue").count() > 1 {
            return fail("a cell with more than one segue is not supported by this ibtool");
        }
        for sg in children(c, "segue") {
            if sg.attribute("trigger").is_some() {
                return fail("a cell's accessory segue is not supported by this ibtool");
            }
        }
        for o in children(c, "outlet") {
            only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
            let destination = view_name(attr(o, "destination")?);
            if !scene.subviews.iter().any(|s| s.1 == destination) {
                return fail("a cell's outlets to anything but its content's views are not supported by this ibtool");
            }
            outlets.push((attr(o, "property")?, destination));
        }
    }
    outlets.sort_by(|a, b| a.0.cmp(b.0));
    let connections = outlets.iter().map(|(label, destination)| outlet(&mut nib, label, &cell_name, destination)).collect();
    Ok(nib::archive(root_object(&mut nib, top, objects, connections)))
}

/// The background view of a cell that selects other than by default: its
/// size rounded up, holding a background configuration of the plain cell
/// background.
fn system_background_view(nib: &mut Nib, f: &Frame) -> Value {
    let mut values = Vec::new();
    geometry(&mut values, &Frame { x: 0.0, y: 0.0, w: f.w.ceil(), h: f.h.ceil() });
    values.extend([("UIDeepDrawRect", Value::Bool(true)), ("UIOpaque", Value::Bool(true)), ("UIAutoresizeSubviews", Value::Bool(true))]);
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    let background = nib.system_color("tableCellPlainBackgroundColor", &SystemColor::Gray([1.0, 1.0]));
    let insets = nib.string("{0, 0, 0, 0}");
    let clear = nib.system_color("clearColor", &SystemColor::Gray([0.0, 0.0]));
    let configuration = Object::new("UIBackgroundConfiguration", vec![
        ("customView", Value::Nil),
        ("hasCustomized-customView", Value::Bool(false)),
        ("cornerRadius", Value::Double(0.0)),
        ("hasCustomized-cornerRadius", Value::Bool(false)),
        ("maskedCorners", Value::Int(15)),
        ("hasCustomized-maskedCorners", Value::Bool(false)),
        ("backgroundColor", background),
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
        ("strokeColor", clear),
        ("hasCustomized-strokeColor", Value::Bool(false)),
        ("strokeColorTransformerIdentifier", Value::Int(0)),
        ("hasCustomized-strokeColorTransformerIdentifier", Value::Bool(false)),
        ("strokeWidth", Value::Double(0.0)),
        ("hasCustomized-strokeWidth", Value::Bool(false)),
        ("strokeOutset", Value::Double(0.0)),
        ("hasCustomized-strokeOutset", Value::Bool(false)),
        ("strokeLocation", Value::Int(0)),
        ("defaultStyle", Value::Int(1)),
        ("defaultStyle_v2", Value::Int(11)),
    ]);
    values.push(("configurationAPI", Value::New(configuration)));
    Value::New(Object::new("_UISystemBackgroundView", values))
}
