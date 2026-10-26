//! Table view controllers' tables and their prototype cells
//! (docs/clean-room/ibtool.md, "Table view controllers").

use super::*;

/// A table's background: a plain table's white, as a system background
/// or any opaque grey white, is its own `tableBackgroundColor`.
fn table_background(nib: &mut Nib, n: Node, plain: bool) -> Result<Value, String> {
    // Without colorMatched, a grey white stays as it is (one corpus
    // storyboard).
    let white = n.attribute("systemColor") == Some("systemBackgroundColor") || (nib.color_matched && opaque_grey_white(n)?);
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

/// Checks a table's attributes and children, as the root view (`key`
/// view) or a subview.
fn check_table(table: Node) -> Result<(), String> {
    only_attributes(table, &[
        ("key", Some("view")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("clearsContextBeforeDrawing", Some("NO")),
        ("contentMode", Some("scaleToFill")), ("alwaysBounceVertical", None), ("showsHorizontalScrollIndicator", Some("NO")),
        ("showsVerticalScrollIndicator", Some("NO")),
        ("keyboardDismissMode", None), ("dataMode", None), ("style", None), ("separatorStyle", None),
        ("allowsSelection", Some("NO")), ("allowsMultipleSelection", Some("YES")), ("allowsSelectionDuringEditing", Some("YES")),
        ("rowHeight", None), ("estimatedRowHeight", None), ("sectionHeaderHeight", None), ("estimatedSectionHeaderHeight", None),
        ("sectionFooterHeight", None), ("estimatedSectionFooterHeight", None), ("id", None),
    ])?;
    only_children(table, &["rect", "autoresizingMask", "color", "inset", "view", "prototypes", "sections", "connections", "constraints"])?;
    for v in children(table, "view") {
        if !matches!(v.attribute("key"), Some("tableHeaderView" | "tableFooterView")) {
            return fail("a <view> in a table other than its header or footer view is not supported by this ibtool");
        }
    }
    for i in children(table, "inset") {
        only_attributes(i, &[("key", Some("separatorInset")), ("minX", None), ("minY", None), ("maxX", None), ("maxY", None)])?;
    }
    if children(table, "inset").next().is_some() && child(table, "color", "sectionIndexBackgroundColor").is_some() {
        return fail("a table with both a separator inset and a section index background is not supported by this ibtool");
    }
    if children(table, "view").next().is_some() && child(table, "color", "separatorColor").is_some() {
        return fail("a table with both a header or footer view and a separator colour is not supported by this ibtool");
    }
    if !matches!(table.attribute("dataMode"), Some("prototypes" | "static")) {
        return fail("a table's dataMode other than prototypes or static is not supported by this ibtool");
    }
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
    if table.attribute("key").is_some() && table.attribute("translatesAutoresizingMaskIntoConstraints").is_some() {
        return fail("a root table view that doesn't translate its mask is not supported by this ibtool");
    }
    Ok(())
}

/// A table's prototype cells with reuse identifiers, in byte order of
/// those, with the placeholder of each one's segue; and the segues' ids.
#[allow(clippy::type_complexity)]
fn prototypes<'a, 'i>(table: Node<'a, 'i>) -> Result<(Vec<(&'a str, Node<'a, 'i>, Option<String>)>, Vec<String>), String> {
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
    Ok((cells, segues))
}

/// A table's values, from its geometry to its separator inset reference:
/// `mask_default` when it gives no autoresizing mask, `layout` (a subview's
/// constraint values) after its clipping. Its header and footer views,
/// named `name`'s subviews, join `scene`.
#[allow(clippy::too_many_arguments)]
fn table_values(
    nib: &mut Nib, scene: &mut Scene, name: &str, table: Node, f: &Frame, cells: &[(&str, Node, Option<String>)], mask_default: i64,
    layout: Vec<(&'static str, Value)>,
) -> Result<Vec<(&'static str, Value)>, String> {
    let (style, ui_style) = match attr(table, "style")? {
        "plain" => (0, 0),
        "grouped" => (1, 1),
        "insetGrouped" => (2, 1),
        other => return fail(format!("table style {other} is not supported by this ibtool")),
    };
    let plain = style == 0;

    let mut values = Vec::new();
    geometry(&mut values, f);
    // The footer view before the header view, wherever they are in the
    // document. The header view is at the top, as wide as the table is laid
    // out. The footer view is right under it, or at the top; in a grouped
    // table, under the header view or 35 points, and a section footer's
    // height (17 when automatic).
    let header = child(table, "view", "tableHeaderView");
    let footer = child(table, "view", "tableFooterView");
    let header_height = header.map(frame).transpose()?.map_or(0.0, |h| h.h);
    let footer_y = if plain {
        header_height
    } else {
        let section_footer = match height(table, "sectionFooterHeight", None)? {
            -1.0 => 17.0,
            h if h >= 0.0 => h as f64,
            h => return fail(format!("a section footer height of {h} is not supported by this ibtool")),
        };
        (if header.is_some() { header_height } else { 35.0 }) + section_footer
    };
    let mut inner = Vec::new();
    for (v, placed) in [
        (footer, footer.map(frame).transpose()?.map(|ff| Frame { y: footer_y, ..ff })),
        (header, Some(Frame { x: 0.0, y: 0.0, w: f.w, h: header_height })),
    ] {
        if let (Some(v), Some(placed)) = (v, placed) {
            let outer = scene.parent.replace(name.to_string());
            let compiled = placed_subview(nib, scene, v, placed);
            scene.parent = outer;
            inner.push(compiled?);
        }
    }
    if !inner.is_empty() {
        values.push(("UISubviews", nib.array("NSMutableArray", inner)));
    }
    if let Some(c) = child(table, "color", "backgroundColor") {
        values.push(("UIBackgroundColor", table_background(nib, c, plain)?));
    }
    values.push(("UIDeepDrawRect", Value::Bool(true)));
    if table.attribute("opaque").is_none() {
        values.push(("UIOpaque", Value::Bool(true)));
    }
    values.push(("UIAutoresizeSubviews", Value::Bool(true)));
    let mask = match children(table, "autoresizingMask").next() {
        Some(m) => autoresizing_mask(m)?,
        None => mask_default,
    };
    values.push(("UIAutoresizingMask", Value::Int(mask)));
    if table.attribute("clearsContextBeforeDrawing").is_some() {
        values.push(("UIClearsContextBeforeDrawing", Value::Bool(false)));
    }
    if table.attribute("clipsSubviews").is_some() {
        values.push(("UIClipsToBounds", Value::Bool(true)));
    }
    values.extend(layout);
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
    for (key, value) in [("tableHeaderView", "UITableHeaderView"), ("tableFooterView", "UITableFooterView")] {
        if let Some(v) = child(table, "view", key) {
            values.push((value, reference(view_name(attr(v, "id")?))));
        }
    }
    if let Some(c) = child(table, "color", "separatorColor") {
        values.push(("UISeparatorColor", color(nib, c)?));
    }

    if !cells.is_empty() {
        let mut nibs = Vec::new();
        for (reuse, c, placeholder) in cells {
            let bytes = cell_nib(nib.platform, nib.color_matched, nib.localizable, scene.module, *c, placeholder.is_some())?;
            nibs.push(nib.string(reuse));
            let data = Value::New(Object::new("NSData", vec![("NS.bytes", Value::Data(bytes))]));
            nibs.push(Value::New(Object::new("UINib", vec![("captureEnclosingNIBBundleOnDecode", Value::Bool(true)), ("archiveData", data)])));
        }
        values.push(("UITableViewCellPrototypeNibs", nib.array("NSMutableDictionary", nibs)));
        // A cell's external objects: its own nib's placeholder for its
        // segue's.
        let mut external = Vec::new();
        for (reuse, _, placeholder) in cells {
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
        values.push(("UISectionIndexBackgroundColor", color(nib, c)?));
    }
    // Top, left, bottom, right; written even when it is the default.
    if let Some(i) = child(table, "inset", "separatorInset") {
        let edges = ["minY", "minX", "maxY", "maxX"].map(|e| attr(i, e).and_then(number));
        let [top, left, bottom, right] = edges;
        values.push(("UISeparatorInset", doubles(&[top?, left?, bottom?, right?])));
    }
    values.push(("UISeparatorInsetReference", Value::Int(0)));
    Ok(values)
}

/// A table view as a subview of a controller's view: its cells' nibs
/// inside it as in a table view controller's, its outlets to the
/// controller. Its cells' segues are not supported.
pub(super) fn subview_table(nib: &mut Nib, scene: &mut Scene, n: Node, name: &str, f: &Frame) -> Result<Object, String> {
    check_table(n)?;
    let (cells, segues) = prototypes(n)?;
    if !segues.is_empty() {
        return fail("segues from the cells of a table view in a view are not supported by this ibtool");
    }
    for c in children(n, "connections") {
        only_children(c, &["outlet"])?;
        for o in children(c, "outlet") {
            only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
            if attr(o, "destination")? != scene.vc_id {
                return fail("a table's outlets to anything but its controller are not supported by this ibtool");
            }
            scene.control_outlets.push((attr(o, "property")?.to_string(), name.to_string()));
        }
    }
    let mut layout = Vec::new();
    if does_not_translate(scene, n) {
        layout.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
    }
    if let Some(own) = own_constraints(nib, scene, n, name)? {
        layout.push(("UIViewAutolayoutConstraints", own));
    }
    let values = table_values(nib, scene, name, n, f, &cells, 36, layout)?;
    Ok(Object::new("UITableView", values))
}

/// A table view controller's view nib: its table, the root view, holding
/// its prototype cells' nibs. Returns the nib's root, whether it uses the
/// controller's placeholder, and the ids of the segues its cells trigger.
pub(super) fn table_view_nib(
    platform: Platform, color_matched: bool, localizable: bool, module: Option<&str>, vc_id: &str, table: Node, f: &Frame,
    vc_outlets: &[(String, String)],
) -> Result<(Object, bool, Vec<String>), String> {
    check_table(table)?;
    if table.attribute("key").is_none() {
        return fail("a table view controller's table view needs key=\"view\"");
    }
    let table_id = attr(table, "id")?;
    // The table's outlets, to its controller.
    let root_name = view_name(table_id);
    let mut outlets: Vec<(String, String, String)> = vc_outlets.iter().map(|(l, d)| (l.clone(), "upstream".to_string(), d.clone())).collect();
    for c in children(table, "connections") {
        only_children(c, &["outlet"])?;
        for o in children(c, "outlet") {
            only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
            if attr(o, "destination")? != vc_id {
                return fail("a table's outlets to anything but its controller are not supported by this ibtool");
            }
            outlets.push((attr(o, "property")?.to_string(), root_name.clone(), "upstream".to_string()));
        }
    }
    let static_table = table.attribute("dataMode") == Some("static");
    let upstream_used = !outlets.is_empty();

    let (cells, segues) = prototypes(table)?;
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

    // The header and footer views, and the views in them.
    let mut scene = Scene::new(vc_id, module, table, Some(root_name.clone()));
    let values = table_values(&mut nib, &mut scene, &root_name, table, f, &cells, 0, Vec::new())?;
    if !scene.actions.is_empty() || !scene.segues.is_empty() || !scene.control_outlets.is_empty() || !scene.gesture_views.is_empty() {
        return fail("actions, outlets to the controller, gestures and segues in a table's header or footer are not supported by this ibtool");
    }
    top.push(Value::New(Object::new("UITableView", values).named(root_name.clone())));
    objects.push(reference(root_name.clone()));
    // After the table (and a static table's data source), breadth first
    // through the table's views and the data source's sections and cells,
    // level by level, the table's first in each: (level, object).
    let mut levels: Vec<(usize, String)> = Vec::new();
    let mut queue: std::collections::VecDeque<(usize, &String)> = [(0, &root_name)].into();
    while let Some((depth, v)) = queue.pop_front() {
        levels.extend(scene.own_constraints.iter().filter(|c| &c.0 == v).map(|c| (depth + 1, c.1.clone())));
        for (_, s) in scene.subviews.iter().filter(|s| s.0.as_ref() == Some(v)) {
            levels.push((depth + 1, s.clone()));
            queue.push_back((depth + 1, s));
        }
    }

    // A static table's sections and their cells, as its data source holds
    // them; its cells are in the table's nib.
    let mut connections = Vec::new();
    if static_table {
        let mut sections = Vec::new();
        let mut section_names = Vec::new();
        let mut parts = Vec::new();
        for ss in children(table, "sections") {
            only_children(ss, &["tableViewSection"])?;
            for sec in children(ss, "tableViewSection") {
                only_attributes(sec, &[("headerTitle", None), ("footerTitle", None), ("id", None)])?;
                only_children(sec, &["cells"])?;
                let sec_id = attr(sec, "id")?;
                let mut rows = Vec::new();
                for cs in children(sec, "cells") {
                    only_children(cs, &["tableViewCell"])?;
                    for cell in children(cs, "tableViewCell") {
                        if children(cell, "connections").next().is_some() {
                            return fail("a static cell's own connections are not supported by this ibtool");
                        }
                        let (cell_value, cell_parts) = cell_object(&mut nib, module, cell, false)?;
                        let height = cell.attribute("rowHeight").map(number).transpose()?.unwrap_or(-1.0);
                        rows.push(Value::New(Object::new("UITableViewRow", vec![("UITableRowCell", cell_value), ("UITableRowHeight", Value::Float(height as f32))])));
                        parts.push(cell_parts);
                    }
                }
                let mut values = Vec::new();
                if let Some(t) = sec.attribute("headerTitle") {
                    values.push(("UITableSectionHeaderTitle", nib.text(sec_id, "headerTitle", t)));
                }
                if let Some(t) = sec.attribute("footerTitle") {
                    values.push(("UITableSectionFooterTitle", nib.text(sec_id, "footerTitle", t)));
                }
                values.push(("UITableSectionRows", nib.array("NSArray", rows)));
                let name = format!("section {sec_id}");
                sections.push(Value::New(Object::new("UITableViewSection", values).named(name.clone())));
                section_names.push(name);
            }
        }
        let sections = nib.array("NSArray", sections);
        top.push(Value::New(Object::new("UITableViewDataSource", vec![("UITableSections", sections)]).named("static data source")));
        // The data source; the sections (level 1); the cells (2); each cell's
        // content view and its views (3), as a prototype's nib lists them.
        objects.push(reference("static data source"));
        levels.extend(section_names.into_iter().map(|n| (1, n)));
        levels.extend(parts.iter().map(|p| (2, p.name.clone())));
        for p in &parts {
            levels.push((3, p.content.clone()));
            let content_parent = Some(p.content.clone());
            levels.extend(p.scene.subviews.iter().filter(|s| s.0 == content_parent).map(|s| (3, s.1.clone())));
        }
        // Then breadth first across the cells: each view's constraints and
        // its subviews, the content views' already listed.
        let mut queue: std::collections::VecDeque<(usize, &CellParts, &String)> = parts.iter().map(|p| (3, p, &p.content)).collect();
        while let Some((depth, p, v)) = queue.pop_front() {
            levels.extend(p.scene.own_constraints.iter().filter(|c| &c.0 == v).map(|c| (depth + 1, c.1.clone())));
            for (_, sv) in p.scene.subviews.iter().filter(|sv| sv.0.as_ref() == Some(v)) {
                if *v == p.content {
                    queue.push_back((depth, p, sv));
                } else {
                    levels.push((depth + 1, sv.clone()));
                    queue.push_back((depth + 1, p, sv));
                }
            }
        }
        connections.push(outlet(&mut nib, "staticDataSource", "owner", "static data source"));
    }

    levels.sort_by_key(|l| l.0);
    objects.extend(levels.iter().map(|l| reference(l.1.clone())));
    for (_, destination) in vc_outlets {
        if *destination != root_name && !levels.iter().any(|l| l.1 == *destination) {
            return fail("a table view controller's outlets into anything but its table and its views are not supported by this ibtool");
        }
    }

    // The header's and footer's views' outlets among them.
    for (label, source, destination) in scene.view_outlets {
        if !levels.iter().any(|l| l.1 == destination) {
            return fail("a header or footer view's outlets to anything but its table's views are not supported by this ibtool");
        }
        outlets.push((label, source, destination));
    }

    // Outlets, `view` among them, in byte order of their names.
    outlets.push(("view".to_string(), "owner".to_string(), root_name.clone()));
    outlets.sort_by(|a, b| a.0.cmp(&b.0));
    for (label, source, destination) in &outlets {
        connections.push(outlet(&mut nib, label, source, destination));
    }
    Ok((root_object(&mut nib, top, objects, connections), upstream_used, segues))
}

/// A table cell as a prototype's nib or a static table's row holds it,
/// and what was compiled for it.
struct CellParts<'a> {
    name: String,
    content: String,
    scene: Scene<'a>,
}

/// A cell: its object (named after its id), with its content view and the
/// separator Xcode lays out at its foot.
fn cell_object<'a>(nib: &mut Nib, module: Option<&'a str>, cell: Node<'a, '_>, has_segue: bool) -> Result<(Value, CellParts<'a>), String> {
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

    // The content view and its subviews, compiled as a container's are.
    let mut scene = Scene::new("", module, content, Some(content_name.clone()));
    let cf = frame(content)?;
    let mut cv = Vec::new();
    geometry(&mut cv, &cf);
    let mut subviews = Vec::new();
    for s in children(content, "subviews") {
        for c in s.children().filter(|c| c.is_element()) {
            subviews.push(subview(nib, &mut scene, c)?);
        }
    }
    if !scene.actions.is_empty() || !scene.segues.is_empty() || !scene.control_outlets.is_empty() || !scene.view_outlets.is_empty() {
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
    if let Some(own) = own_constraints(nib, &mut scene, content, &content_name)? {
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
        values.push(("UIBackgroundColor", color(nib, c)?));
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
    // A static table's cell has no reuse identifier.
    if let Some(reuse) = cell.attribute("reuseIdentifier") {
        values.push(("UIReuseIdentifier", nib.string(reuse)));
    }
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
        values.push(("UISystemBackgroundView", system_background_view(nib, &f)));
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
    let _ = has_segue;
    Ok((Value::New(object.named(cell_name.clone())), CellParts { name: cell_name, content: content_name, scene }))
}

/// A prototype cell's nib: the cell, its content view and the separator
/// Xcode lays out at its foot, and the content view's subviews.
fn cell_nib(platform: Platform, color_matched: bool, localizable: bool, module: Option<&str>, cell: Node, has_segue: bool) -> Result<Vec<u8>, String> {
    let mut nib = Nib::new(platform, color_matched, localizable);
    let owner = proxy(&mut nib, "IBFilesOwner", "owner");
    let responder = proxy(&mut nib, "IBFirstResponder", "responder");
    // A cell's segue is its placeholder's.
    let upstream = if has_segue { Some(proxy(&mut nib, "UpstreamPlaceholder-1", "upstream")) } else { None };

    let (cell_value, parts) = cell_object(&mut nib, module, cell, has_segue)?;
    let CellParts { name: cell_name, content: content_name, scene } = parts;
    let mut top = vec![owner, responder];
    top.extend(upstream);
    top.push(cell_value);

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
