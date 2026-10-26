//! Table view controllers' tables and their prototype cells
//! (docs/clean-room/ibtool.md, "Table view controllers").

use super::*;

/// A table's header or footer view (`key`): a plain view, or for the
/// header a search bar.
fn header_or_footer<'a, 'i>(table: Node<'a, 'i>, key: &str) -> Option<Node<'a, 'i>> {
    table.children().find(|c| (c.has_tag_name("view") || c.has_tag_name("searchBar") || c.has_tag_name("pickerView")) && c.attribute("key") == Some(key))
}

/// A table's background: a plain table's white, as a system background
/// or any opaque grey white, is its own `tableBackgroundColor`.
fn table_background(nib: &mut Nib, n: Node, plain: bool) -> Result<Value, String> {
    // Without colorMatched, a grey white stays as it is (one corpus
    // storyboard).
    let white = n.attribute("systemColor") == Some("systemBackgroundColor") || (nib.color_matched && opaque_grey_white(n)?);
    if plain && white {
        return Ok(nib.system_color("tableBackgroundColor", &SystemColor::Gray([1.0, 1.0])));
    }
    // A grouped or inset grouped table's sRGB background of the grouped
    // background colour's own value, at 10 digits, is that system colour;
    // a plain table's, a near value and other colours are not (six probes,
    // one corpus storyboard).
    const GROUPED: [f64; 4] = [0.9490196078, 0.9490196078, 0.968627451, 1.0];
    if !plain && nib.color_matched && n.attribute("customColorSpace") == Some("sRGB") {
        let c = ["red", "green", "blue", "alpha"].map(|a| n.attribute(a).map(number));
        if let [Some(Ok(r)), Some(Ok(g)), Some(Ok(b)), Some(Ok(a))] = c {
            if [r, g, b, a].map(significant_10) == GROUPED {
                return Ok(nib.system_color("systemGroupedBackgroundColor", &SystemColor::Rgb(GROUPED)));
            }
        }
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
        ("userInteractionEnabled", Some("NO")),
        ("contentMode", Some("scaleToFill")), ("alwaysBounceVertical", None), ("showsHorizontalScrollIndicator", Some("NO")),
        ("showsVerticalScrollIndicator", Some("NO")),
        ("keyboardDismissMode", None), ("dataMode", None), ("style", None), ("separatorStyle", None),
        ("allowsSelection", Some("NO")), ("allowsMultipleSelection", Some("YES")), ("allowsSelectionDuringEditing", Some("YES")),
        ("rowHeight", None), ("estimatedRowHeight", None), ("sectionHeaderHeight", None), ("estimatedSectionHeaderHeight", None),
        ("sectionFooterHeight", None), ("estimatedSectionFooterHeight", None), ("restorationIdentifier", None), ("id", None),
        ("customClass", None), ("customModule", None), ("customModuleProvider", None),
    ])?;
    only_children(table, &["rect", "autoresizingMask", "color", "inset", "view", "searchBar", "pickerView", "prototypes", "sections", "connections", "constraints"])?;
    // A picker as a header or footer view too (a corpus storyboard).
    for v in children(table, "view").chain(children(table, "pickerView")) {
        if !matches!(v.attribute("key"), Some("tableHeaderView" | "tableFooterView")) {
            return fail("a <view> in a table other than its header or footer view is not supported by this ibtool");
        }
    }
    for v in children(table, "searchBar") {
        if v.attribute("key") != Some("tableHeaderView") {
            return fail("a <searchBar> in a table other than its header view is not supported by this ibtool");
        }
    }
    let keyed = |key: &str| children(table, "view").chain(children(table, "searchBar")).chain(children(table, "pickerView")).filter(|v| v.attribute("key") == Some(key)).count();
    if keyed("tableFooterView") > 1 {
        return fail("a table with two footer views is not supported by this ibtool");
    }
    if keyed("tableHeaderView") > 1 {
        return fail("a table with two header views is not supported by this ibtool");
    }
    for i in children(table, "inset") {
        only_attributes(i, &[("key", Some("separatorInset")), ("minX", None), ("minY", None), ("maxX", None), ("maxY", None)])?;
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
            let segue = cell_connections(c)?.0;
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
    layout: Vec<(&'static str, Value)>, refresh: Option<Value>,
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
    let header = header_or_footer(table, "tableHeaderView");
    let footer = header_or_footer(table, "tableFooterView");
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
    // These nibs don't list placeholder guides: a subview's own safe-area
    // guide or a scroll view's guides are refused here.
    if !scene.unused_guides.is_empty() {
        return fail("a safe-area or scroll view guide in a cell, header or table is not supported by this ibtool");
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
    // One probe.
    if table.attribute("userInteractionEnabled").is_some() {
        values.push(("UIUserInteractionDisabled", Value::Bool(true)));
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
    // A root table takes the document's tint.
    if table.attribute("key") == Some("view") {
        if let Some(tint) = document_tint(table)? {
            values.push(("UITintColor", color(nib, tint)?));
        }
    }
    values.extend(layout);
    restoration_identifier(nib, &mut values, table);
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
    if let Some(r) = refresh {
        values.push(("UIRefreshControl", r));
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
    // No separator style is the default (one probe).
    let separator = match table.attribute("separatorStyle").unwrap_or("default") {
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
        if let Some(v) = header_or_footer(table, key) {
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
            scene.control_outlets.push((attr(o, "property")?.to_string(), name.to_string(), "upstream".into()));
        }
    }
    let mut layout = Vec::new();
    if does_not_translate(scene, n) {
        layout.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
    }
    if let Some(own) = own_constraints(nib, scene, n, name)? {
        layout.push(("UIViewAutolayoutConstraints", own));
    }
    let values = table_values(nib, scene, name, n, f, &cells, 36, layout, None)?;
    swapped_table(nib, n, scene.module, values)
}

/// A table view, or its custom class swapped in for it after its values,
/// as for a view (two probes).
fn swapped_table(nib: &mut Nib, n: Node, module: Option<&str>, mut values: Vec<(&'static str, Value)>) -> Result<Object, String> {
    if n.attribute("customClass").is_none() {
        return Ok(Object::new("UITableView", values));
    }
    let class = runtime_class_name(n, module, "UITableView")?;
    values.push(("UIClassName", nib.string(&class)));
    values.push(("UIOriginalClassName", nib.string("UITableView")));
    Ok(Object::new("UIClassSwapper", values))
}

/// A table view controller's view nib: its table, the root view, holding
/// its prototype cells' nibs. Returns the nib's root, whether it uses the
/// controller's placeholder, the ids of the segues its cells trigger in
/// their placeholders' order, and for a static table the order the
/// controller's nib lists them in (as placeholder indices).
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub(super) fn table_view_nib(
    platform: Platform, color_matched: bool, localizable: bool, module: Option<&str>, vc_id: &str, table: Node, f: &Frame,
    vc_outlets: &[(String, String)], refresh: Option<Node>,
) -> Result<(Object, bool, Vec<String>, Option<Vec<usize>>), String> {
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
    // A refresh control is the table's, its action on the controller.
    let mut refresh_actions = Vec::new();
    if let Some(r) = refresh {
        only_attributes(r, &[
            ("key", Some("refreshControl")), ("opaque", Some("NO")), ("multipleTouchEnabled", Some("YES")), ("contentMode", Some("center")),
            ("enabled", Some("NO")), ("contentHorizontalAlignment", Some("center")), ("contentVerticalAlignment", Some("center")), ("id", None),
        ])?;
        only_children(r, &["rect", "autoresizingMask", "connections", "attributedString"])?;
        for m in children(r, "autoresizingMask") {
            if autoresizing_mask(m)? != 0 || m.has_children() {
                return fail("a refresh control with an autoresizing mask is not supported by this ibtool");
            }
        }
        for required in ["opaque", "multipleTouchEnabled", "contentMode", "enabled"] {
            if r.attribute(required).is_none() {
                return fail(format!("a refresh control without {required} is not supported by this ibtool"));
            }
        }
        if table.attribute("dataMode") != Some("prototypes")
            || header_or_footer(table, "tableHeaderView").is_some()
            || header_or_footer(table, "tableFooterView").is_some()
            || table.attribute("keyboardDismissMode").is_some()
        {
            return fail("a refresh control in a static table, or with a header or footer view or a keyboard dismiss mode, is not supported by this ibtool");
        }
        for c in children(r, "connections") {
            only_children(c, &["action"])?;
            for a in children(c, "action") {
                only_attributes(a, &[("selector", None), ("destination", None), ("eventType", Some("valueChanged")), ("id", None)])?;
                if attr(a, "destination")? != vc_id {
                    return fail("a refresh control's actions on anything but its controller are not supported by this ibtool");
                }
                refresh_actions.push(attr(a, "selector")?.to_string());
            }
        }
    }
    // A static cell's control's action is on the controller too.
    let static_actions = table.attribute("dataMode") == Some("static")
        && table.descendants().any(|d| d.has_tag_name("action") && d.ancestors().any(|a| a.has_tag_name("tableViewCell")));
    let upstream_used = !outlets.is_empty() || !refresh_actions.is_empty() || static_actions;

    let (mut cells, mut segues) = prototypes(table)?;
    // The embed segues of container views in the header view: their
    // placeholders follow the controller's, before the prototypes' or
    // static cells', wherever the header is in the document, and the
    // controller's nib lists them first (five probes). Those in a footer
    // aren't established.
    let embeds = |key: &str| -> Vec<String> {
        header_or_footer(table, key)
            .into_iter()
            .flat_map(|v| v.descendants().filter(|d| d.has_tag_name("containerView")))
            .flat_map(|c| c.descendants().filter(|d| d.has_tag_name("segue")).filter_map(|sg| sg.attribute("id")).map(str::to_string))
            .collect()
    };
    let header_segues = embeds("tableHeaderView");
    if !embeds("tableFooterView").is_empty() {
        return fail("container views in a table's footer are not supported by this ibtool");
    }
    if !header_segues.is_empty() {
        let shift = header_segues.len();
        for (_, _, placeholder) in &mut cells {
            if let Some(p) = placeholder {
                let i: usize = p.trim_start_matches("upstream segue ").parse().expect("named by prototypes");
                *p = format!("upstream segue {}", i + shift);
            }
        }
        segues.splice(0..0, header_segues.iter().cloned());
    }
    // A static table's cells' segues: their placeholders follow the
    // controller's, section by section, each section's cells in reverse;
    // the controller's nib lists them in document order.
    let mut listing = None;
    if table.attribute("dataMode") == Some("static") {
        let sections: Vec<Vec<Node>> = children(table, "sections")
            .flat_map(|ss| children(ss, "tableViewSection"))
            .map(|sec| children(sec, "cells").flat_map(|cs| children(cs, "tableViewCell")).collect())
            .collect();
        for cells in &sections {
            for cell in cells.iter().rev() {
                segues.extend(cell_connections(*cell)?.0.map(str::to_string));
            }
        }
        let mut order: Vec<usize> = (0..header_segues.len()).collect();
        for cell in sections.iter().flatten() {
            if let Some(id) = cell_connections(*cell)?.0 {
                order.push(segues.iter().position(|s| s == id).expect("collected above"));
            }
        }
        listing = Some(order);
    }
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
    let refresh_value = match refresh {
        Some(r) => {
            let mut values = Vec::new();
            // Without a frame, 1000x1000 at the origin, as a subview (two
            // corpus storyboards).
            let f = match child(r, "rect", "frame") {
                Some(_) => frame(r)?,
                None => Frame { x: 0.0, y: 0.0, w: 1000.0, h: 1000.0 },
            };
            geometry(&mut values, &f);
            values.extend([("UIDeepDrawRect", Value::Bool(true)), ("UIMultipleTouchEnabled", Value::Bool(true)), ("UIAutoresizeSubviews", Value::Bool(true))]);
            values.push(("UIContentMode", Value::Int(4)));
            values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
            nib.large_content(&mut values);
            values.push(("UIDisabled", Value::Bool(true)));
            values.push(("UIRefreshControlStyle", Value::Int(2)));
            if let Some(title) = child(r, "attributedString", "attributedTitle") {
                values.push(("UIAttributedTitle", refresh_title(&mut nib, title)?));
            }
            Some(Value::New(Object::new("UIRefreshControl", values).named("refresh control")))
        }
        None => None,
    };
    let values = table_values(&mut nib, &mut scene, &root_name, table, f, &cells, 0, Vec::new(), refresh_value)?;
    if scene.segues != header_segues {
        return fail("segues in a table's header or footer other than its containers' are not supported by this ibtool");
    }
    if !scene.actions.is_empty() || !scene.gesture_views.is_empty() {
        return fail("actions, gestures and segues in a table's header or footer are not supported by this ibtool");
    }
    // A header or footer view's outlets to the controller (a search bar's
    // or text field's delegate), before the table's own of one name, as
    // the document has them (a corpus storyboard).
    if scene.control_outlets.iter().any(|o| o.2 != "upstream") {
        return fail("a header or footer view's outlets to custom objects are not supported by this ibtool");
    }
    let header_outlets = std::mem::take(&mut scene.control_outlets);
    let at = vc_outlets.len();
    outlets.splice(at..at, header_outlets);
    if !scene.pairs.is_empty() || !scene.accessibility.is_empty() {
        return fail("runtime attributes, switches and accessibility settings in a table's header or footer are not supported by this ibtool");
    }
    top.push(Value::New(swapped_table(&mut nib, table, module, values)?.named(root_name.clone())));
    objects.push(reference(root_name.clone()));
    if refresh.is_some() {
        objects.push(reference("refresh control"));
    }
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
    let mut cell_actions = Vec::new();
    let mut cell_pairs = Vec::new();
    if static_table {
        let mut sections = Vec::new();
        let mut section_names = Vec::new();
        let mut parts = Vec::new();
        let mut segue_outlets = Vec::new();
        for ss in children(table, "sections") {
            only_children(ss, &["tableViewSection"])?;
            for sec in children(ss, "tableViewSection") {
                only_attributes(sec, &[("headerTitle", None), ("footerTitle", None), ("id", None)])?;
                only_children(sec, &["cells"])?;
                let sec_id = attr(sec, "id")?;
                let mut rows = Vec::new();
                // Its cells' outlets, which go among the outlets cell by cell
                // in reverse, as their segues' placeholders do: it decides
                // between outlets of the same name.
                let mut section_outlets = Vec::new();
                for cs in children(sec, "cells") {
                    only_children(cs, &["tableViewCell"])?;
                    for cell in children(cs, "tableViewCell") {
                        let (cell_value, cell_parts) = cell_object(&mut nib, module, cell, vc_id)?;
                        if !cell_parts.scene.accessibility.is_empty() {
                            return fail("accessibility settings on a static table's cells are not supported by this ibtool");
                        }
                        // Its segue's placeholder and its outlets into its
                        // views, among the outlets.
                        let (segue, cell_outlets) = cell_connections(cell)?;
                        if let Some(id) = segue {
                            let i = segues.iter().position(|s| s == id).expect("static segues are collected first");
                            segue_outlets.push((i, cell_parts.name.clone()));
                        }
                        let mut own = Vec::new();
                        for (label, destination) in cell_outlets {
                            let destination = view_name(destination);
                            if !cell_parts.scene.subviews.iter().any(|s| s.1 == destination) {
                                return fail("a cell's outlets to anything but its content's views are not supported by this ibtool");
                            }
                            own.push((label.to_string(), cell_parts.name.clone(), destination));
                        }
                        section_outlets.push(own);
                        let height = cell.attribute("rowHeight").map(number).transpose()?.unwrap_or(-1.0);
                        rows.push(Value::New(Object::new("UITableViewRow", vec![("UITableRowCell", cell_value), ("UITableRowHeight", Value::Float(height as f32))])));
                        parts.push(cell_parts);
                    }
                }
                outlets.extend(section_outlets.into_iter().rev().flatten());
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
        // Cells' segue outlets in their placeholders' order.
        segue_outlets.sort();
        outlets.extend(segue_outlets.into_iter().map(|(i, cell)| ("selectionSegueTemplate".to_string(), cell, format!("upstream segue {i}"))));
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
        // The content views' subviews are walked with them, cell by cell, as
        // they are listed (one corpus storyboard).
        let mut queue: std::collections::VecDeque<(usize, &CellParts, &String)> = std::collections::VecDeque::new();
        for p in &parts {
            queue.push_back((3, p, &p.content));
            queue.extend(p.scene.subviews.iter().filter(|sv| sv.0.as_ref() == Some(&p.content)).map(|sv| (3, p, &sv.1)));
        }
        while let Some((depth, p, v)) = queue.pop_front() {
            levels.extend(p.scene.own_constraints.iter().filter(|c| &c.0 == v).map(|c| (depth + 1, c.1.clone())));
            if *v == p.content {
                continue;
            }
            for (_, sv) in p.scene.subviews.iter().filter(|sv| sv.0.as_ref() == Some(v)) {
                levels.push((depth + 1, sv.clone()));
                queue.push_back((depth + 1, p, sv));
            }
        }
        connections.push(outlet(&mut nib, "staticDataSource", "owner", "static data source"));
        // Its cells' controls' actions on the controller and their switches'
        // frame pairs, cell by cell, are the table nib's (five probes).
        for (i, p) in parts.iter_mut().enumerate() {
            cell_actions.extend(std::mem::take(&mut p.scene.actions).into_iter().map(|a| (i, a)));
            cell_pairs.extend(std::mem::take(&mut p.scene.pairs).into_iter().map(|p| p.1));
        }
    }

    levels.sort_by_key(|l| l.0);
    objects.extend(levels.iter().map(|l| reference(l.1.clone())));
    for (_, destination) in vc_outlets {
        if *destination != root_name && !levels.iter().any(|l| l.1 == *destination) {
            return fail("a table view controller's outlets into anything but its table and its views are not supported by this ibtool");
        }
    }

    // The header's and footer's views' outlets among them, its container
    // views' from their placeholders.
    for (placeholder, view) in scene.containers {
        outlets.push(("containerView".to_string(), placeholder, view));
    }
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
    // The refresh control's value-changed actions, after the outlets.
    for selector in refresh_actions {
        let label = nib.string(&selector);
        connections.push(Value::New(Object::new("UIRuntimeEventConnection", vec![
            ("UILabel", label),
            ("UISource", reference("refresh control")),
            ("UIDestination", reference("upstream")),
            ("UIEventMask", Value::Int(4096)),
        ])));
    }
    // A static table's cells' actions after the outlets, by mask and then
    // selector, those sharing them in the cells' order (two probes); two in
    // one cell sharing them are refused.
    cell_actions.sort_by(|a, b| (a.1 .2, &a.1 .0).cmp(&(b.1 .2, &b.1 .0)));
    if cell_actions.windows(2).any(|w| (w[0].0, w[0].1 .2, &w[0].1 .0) == (w[1].0, w[1].1 .2, &w[1].1 .0)) {
        return fail("actions in a static table's cell sharing a selector are not supported by this ibtool");
    }
    for (_, (selector, source, mask, _)) in &cell_actions {
        let label = nib.string(selector);
        connections.push(Value::New(Object::new("UIRuntimeEventConnection", vec![
            ("UILabel", label),
            ("UISource", reference(source.clone())),
            ("UIDestination", reference("upstream")),
            ("UIEventMask", Value::Int(*mask)),
        ])));
    }
    Ok((root_object_with_pairs(&mut nib, top, objects, connections, cell_pairs), upstream_used, segues, listing))
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
/// Its controls' actions may target `target`: the cell itself in a
/// prototype's nib, the controller in a static table's.
fn cell_object<'a>(nib: &mut Nib, module: Option<&'a str>, cell: Node<'a, '_>, target: &'a str) -> Result<(Value, CellParts<'a>), String> {
    only_attributes(cell, &[
        ("clipsSubviews", Some("YES")), ("contentMode", Some("scaleToFill")), ("preservesSuperviewLayoutMargins", Some("YES")),
        ("selectionStyle", None), ("hidesAccessoryWhenEditing", Some("NO")), ("indentationLevel", None), ("indentationWidth", None),
        ("accessoryType", None), ("reuseIdentifier", None), ("rowHeight", None), ("customClass", None), ("customModule", None),
        ("customModuleProvider", None), ("userLabel", None), ("textLabel", None), ("detailTextLabel", None), ("style", None), ("restorationIdentifier", None),
        ("insetsLayoutMarginsFromSafeArea", Some("NO")), ("id", None),
    ])?;
    only_children(cell, &["rect", "autoresizingMask", "tableViewCellContentView", "color", "connections", "accessibility"])?;
    // A built-in style's labels, the content view's.
    let style = match cell.attribute("style") {
        None | Some("IBUITableViewCellStyleDefault") => 0,
        Some("IBUITableViewCellStyleValue1") => 1,
        Some("IBUITableViewCellStyleValue2") => 2,
        Some("IBUITableViewCellStyleSubtitle") => 3,
        Some(other) => return fail(format!("cell style {other} is not supported by this ibtool")),
    };
    let text_label = cell.attribute("textLabel").map(view_name);
    let detail_label = cell.attribute("detailTextLabel").map(view_name);
    if (style != 0) != detail_label.is_some() || (cell.attribute("style").is_some() && text_label.is_none()) {
        return fail("a cell style without its labels is not supported by this ibtool");
    }
    // Its mask, when it has one, after `UIAutoresizeSubviews` (three
    // probes, prototype and static cells).
    let mut mask = 0;
    for m in children(cell, "autoresizingMask") {
        mask = autoresizing_mask(m)?;
        if m.has_children() {
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
    only_children(content, &["rect", "autoresizingMask", "subviews", "constraints", "color", "edgeInsets"])?;
    if children(content, "color").any(|c| c.attribute("key") != Some("backgroundColor")) {
        return fail("a cell's content view colours other than its background are not supported by this ibtool");
    }
    for m in children(content, "autoresizingMask") {
        if autoresizing_mask(m)? != 0 || m.has_children() {
            return fail("a cell's content view with an autoresizing mask is not supported by this ibtool");
        }
    }
    let content_id = attr(content, "id")?;
    let content_name = view_name(content_id);

    // The content view and its subviews, compiled as a container's are.
    let mut scene = Scene::new(target, module, content, Some(content_name.clone()));
    let cf = frame(content)?;
    let mut cv = Vec::new();
    geometry(&mut cv, &cf);
    let mut subviews = Vec::new();
    for s in children(content, "subviews") {
        for c in s.children().filter(|c| c.is_element()) {
            let mut v = subview(nib, &mut scene, c)?;
            if let Value::New(o) = &mut v {
                if o.name.is_some() && (o.name == text_label || o.name == detail_label) {
                    if o.class != "UILabel" {
                        return fail("a cell's text label that isn't a label is not supported by this ibtool");
                    }
                    o.class = "UITableViewLabel";
                }
            }
            subviews.push(v);
        }
    }
    // These nibs don't list placeholder guides: a subview's own safe-area
    // guide or a scroll view's guides are refused here.
    if !scene.unused_guides.is_empty() {
        return fail("a safe-area or scroll view guide in a cell, header or table is not supported by this ibtool");
    }
    for l in text_label.iter().chain(&detail_label) {
        if !scene.subviews.iter().any(|s| s.0.as_ref() == Some(&content_name) && s.1 == *l) {
            return fail("a cell's text label outside its content view is not supported by this ibtool");
        }
    }
    // The objects list a style's detail label before its text label,
    // wherever they are in the document (three probes).
    if text_label.is_some() {
        if scene.subviews.len() != 1 + usize::from(detail_label.is_some()) {
            return fail("other views beside a cell style's labels are not supported by this ibtool");
        }
        scene.subviews.sort_by_key(|s| Some(&s.1) != detail_label.as_ref());
    }
    if scene.actions.iter().any(|a| a.3 != "upstream") || !scene.segues.is_empty() || !scene.control_outlets.is_empty() || !scene.view_outlets.is_empty() {
        return fail("actions other than on the cell, outlets and segues in a cell are not supported by this ibtool");
    }
    // A prototype's nib holds its switches' frames and its views' runtime
    // attributes as key-value pairs, as a controller's view does (five
    // probes, and see below); accessibility configurations are refused.
    if !scene.accessibility.is_empty() {
        return fail("accessibility settings in a cell are not supported by this ibtool");
    }
    // The cell's own accessibility configuration, as a view's, in its nib
    // (three probes); beside its views' it isn't established.
    accessibility(nib, &mut scene, cell, &cell_name)?;
    if !subviews.is_empty() {
        cv.push(("UISubviews", nib.array("NSMutableArray", subviews)));
    }
    // A background colour after its subviews, layout margins after its
    // constraints (two probes).
    if let Some(c) = child(content, "color", "backgroundColor") {
        cv.push(("UIBackgroundColor", color(nib, c)?));
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
    if let Some((m, directional)) = layout_margins(content)? {
        cv.push(("UIViewLayoutMargins", doubles(&m)));
        if directional {
            cv.push(("UIViewLayoutMarginsAreDirectional", Value::Bool(true)));
        }
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
    if mask != 0 {
        values.push(("UIAutoresizingMask", Value::Int(mask)));
    }
    if cell.attribute("clipsSubviews").is_some() {
        values.push(("UIClipsToBounds", Value::Bool(true)));
    }
    if cell.attribute("preservesSuperviewLayoutMargins").is_some() {
        values.push(("UIViewPreservesSuperviewMargins", Value::Bool(true)));
        values.push(("UIViewPreservesSpecificSuperviewMargins", Value::Int(15)));
    }
    // One probe.
    if cell.attribute("insetsLayoutMarginsFromSafeArea").is_some() {
        values.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0)));
    }
    restoration_identifier(nib, &mut values, cell);
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    // A static table's cell has no reuse identifier.
    if let Some(reuse) = cell.attribute("reuseIdentifier") {
        values.push(("UIReuseIdentifier", nib.string(reuse)));
    }
    if style != 0 {
        values.push(("UITableViewCellStyle", Value::Int(style)));
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
    values.push(("UITextLabel", text_label.map_or(Value::Nil, reference)));
    values.push(("UIDetailTextLabel", detail_label.map_or(Value::Nil, reference)));
    values.push(("UIImageView", Value::Nil));
    let object = if cell.attribute("customClass").is_some() {
        let class = runtime_class_name(cell, module, "UITableViewCell")?;
        values.push(("UIClassName", nib.string(&class)));
        values.push(("UIOriginalClassName", nib.string("UITableViewCell")));
        Object::new("UIClassSwapper", values)
    } else {
        Object::new("UITableViewCell", values)
    };
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

    let (cell_value, parts) = cell_object(&mut nib, module, cell, attr(cell, "id")?)?;
    let CellParts { name: cell_name, content: content_name, mut scene } = parts;
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
    for (label, destination) in cell_connections(cell)?.1 {
        let destination = view_name(destination);
        if !scene.subviews.iter().any(|s| s.1 == destination) {
            return fail("a cell's outlets to anything but its content's views are not supported by this ibtool");
        }
        outlets.push((label, destination));
    }
    outlets.sort_by(|a, b| a.0.cmp(b.0));
    let mut connections: Vec<Value> = outlets.iter().map(|(label, destination)| outlet(&mut nib, label, &cell_name, destination)).collect();
    // Its controls' actions on the cell after its outlets, by mask and then
    // selector (five probes, one corpus storyboard).
    let mut actions: Vec<_> = scene.actions.iter().collect();
    actions.sort_by(|a, b| (a.2, &a.0).cmp(&(b.2, &b.0)));
    if actions.windows(2).any(|w| (w[0].2, &w[0].0) == (w[1].2, &w[1].0)) {
        return fail("actions in a cell sharing a selector are not supported by this ibtool");
    }
    for (selector, source, mask, _) in actions {
        let label = nib.string(selector);
        connections.push(Value::New(Object::new("UIRuntimeEventConnection", vec![
            ("UILabel", label),
            ("UISource", reference(source.clone())),
            ("UIDestination", reference(cell_name.clone())),
            ("UIEventMask", Value::Int(*mask)),
        ])));
    }
    // Pairs of several views go by their views' places in the objects,
    // counted from 1 and compared as decimal strings, as a view nib's.
    let listed = |name: &String| objects.iter().position(|o| matches!(o, Value::Ref(r) if r == name)).map(|i| (i + 1).to_string());
    if scene.pairs.iter().any(|p| listed(&p.0).is_none()) {
        return fail("key-value pairs of a view a cell's objects don't list are not supported by this ibtool");
    }
    scene.pairs.sort_by_key(|p| listed(&p.0));
    let pairs = std::mem::take(&mut scene.pairs).into_iter().map(|p| p.1).collect();
    let accessibility = std::mem::take(&mut scene.accessibility).into_iter().map(|a| a.1).collect();
    nib::archive(root_object_full(&mut nib, top, objects, connections, accessibility, pairs))
}

/// A cell's own connections: its selection segue's id, if it has one, and
/// its outlets (label, destination id).
#[allow(clippy::type_complexity)]
fn cell_connections<'a>(cell: Node<'a, '_>) -> Result<(Option<&'a str>, Vec<(&'a str, &'a str)>), String> {
    let mut segue = None;
    let mut outlets = Vec::new();
    for c in children(cell, "connections") {
        only_children(c, &["outlet", "segue"])?;
        for sg in children(c, "segue") {
            if segue.is_some() {
                return fail("a cell with more than one segue is not supported by this ibtool");
            }
            if sg.attribute("trigger").is_some() {
                return fail("a cell's accessory segue is not supported by this ibtool");
            }
            segue = Some(attr(sg, "id")?);
        }
        for o in children(c, "outlet") {
            only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
            outlets.push((attr(o, "property")?, attr(o, "destination")?));
        }
    }
    Ok((segue, outlets))
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

/// A refresh control's attributed title: one fragment with a font and
/// perhaps a paragraph style, an `NSAttributedString` of the text and an
/// `NSFont` (the system font: 11 points for the small system font, 13 for
/// the message and system fonts unless sized) then an `NSParagraphStyle`
/// of its alignment (left left out, right 1, centre 2, natural 4) and no
/// tab stops (eight probes, two corpus storyboards).
fn refresh_title(nib: &mut Nib, title: Node) -> Result<Value, String> {
    only_attributes(title, &[("key", Some("attributedTitle"))])?;
    only_children(title, &["fragment"])?;
    let fragments: Vec<Node> = children(title, "fragment").collect();
    let [fragment] = fragments[..] else { return fail("a refresh control's title of other than one fragment is not supported by this ibtool") };
    only_attributes(fragment, &[("content", None)])?;
    only_children(fragment, &["attributes"])?;
    let attributes: Vec<Node> = children(fragment, "attributes").flat_map(|a| a.children().filter(|c| c.is_element())).collect();
    let mut font_size = None;
    let mut alignment = None;
    let mut paragraph = false;
    for a in attributes {
        match (a.tag_name().name(), a.attribute("key")) {
            ("font", Some("NSFont")) => {
                only_attributes(a, &[("key", None), ("metaFont", None), ("size", None)])?;
                let size = match (attr(a, "metaFont")?, a.attribute("size")) {
                    ("smallSystem", None) => 11.0,
                    ("message" | "system", None) => 13.0,
                    ("message" | "system", Some(size)) => number(size)?,
                    (meta, _) => return fail(format!("a refresh control's {meta} font is not supported by this ibtool")),
                };
                font_size = Some(size);
            }
            ("paragraphStyle", Some("NSParagraphStyle")) => {
                only_attributes(a, &[
                    ("key", None), ("alignment", None), ("lineBreakMode", Some("wordWrapping")), ("baseWritingDirection", Some("natural")),
                    ("tighteningFactorForTruncation", Some("0.0")),
                ])?;
                paragraph = true;
                alignment = match a.attribute("alignment") {
                    None | Some("left") => None,
                    Some("right") => Some(1),
                    Some("center") => Some(2),
                    Some("natural") => Some(4),
                    Some(other) => return fail(format!("a refresh control's title alignment {other} is not supported by this ibtool")),
                };
            }
            (tag, key) => return fail(format!("a refresh control's title attribute <{tag}> {} is not supported by this ibtool", key.unwrap_or("unkeyed"))),
        }
    }
    let size = font_size.ok_or("a refresh control's title without a font is not supported by this ibtool")?;
    let mut entries = vec![nib.string("NSFont"), font(nib, None, Some(size))?];
    if paragraph {
        let mut style = Vec::new();
        if let Some(a) = alignment {
            style.push(("NSAlignment", Value::Int(a)));
        }
        style.push(("NSTabStops", Value::Nil));
        entries.extend([nib.string("NSParagraphStyle"), Value::New(Object::new("NSParagraphStyle", style))]);
    }
    let text = nib.string(attr(fragment, "content")?);
    let dictionary = nib.array("NSDictionary", entries);
    Ok(Value::New(Object::new("NSAttributedString", vec![("NSString", text), ("NSAttributes", dictionary)])))
}
