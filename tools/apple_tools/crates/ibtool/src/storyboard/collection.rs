//! Collection view controllers' collection views and their prototype cells
//! (docs/clean-room/ibtool.md, "Collection view controllers").

use super::*;

/// A collection view controller's view nib: its collection view, the root
/// view, with its flow layout and its prototype cells' nibs. Returns the
/// nib's root, whether it uses the controller's placeholder, and the ids of
/// the segues its cells trigger.
#[allow(clippy::too_many_arguments)]
pub(super) fn collection_view_nib(
    platform: Platform, color_matched: bool, localizable: bool, module: Option<&str>, vc_id: &str, cv: Node, f: &Frame,
    vc_outlets: &[(String, String)],
) -> Result<(Object, bool, Vec<String>), String> {
    only_attributes(cv, &[
        ("key", Some("view")), ("clipsSubviews", Some("YES")), ("multipleTouchEnabled", Some("YES")), ("contentMode", Some("scaleToFill")),
        ("dataMode", Some("prototypes")), ("id", None),
    ])?;
    only_children(cv, &["rect", "autoresizingMask", "color", "collectionViewFlowLayout", "cells", "connections"])?;
    for c in children(cv, "color") {
        if c.attribute("key") != Some("backgroundColor") {
            return fail(format!("a collection view's {} colour is not supported by this ibtool", c.attribute("key").unwrap_or("")));
        }
    }
    let cv_id = attr(cv, "id")?;
    let root_name = view_name(cv_id);

    // The collection view's outlets, to its controller.
    let mut outlets: Vec<(String, &str, String)> = vc_outlets.iter().map(|(l, d)| (l.clone(), "upstream", d.clone())).collect();
    for c in children(cv, "connections") {
        only_children(c, &["outlet"])?;
        for o in children(c, "outlet") {
            only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
            if attr(o, "destination")? != vc_id {
                return fail("a collection view's outlets to anything but its controller are not supported by this ibtool");
            }
            outlets.push((attr(o, "property")?.to_string(), "collection", "upstream".to_string()));
        }
    }
    for (_, destination) in vc_outlets {
        if *destination != root_name {
            return fail("a collection view controller's outlets into anything but its collection view are not supported by this ibtool");
        }
    }
    let upstream_used = !outlets.is_empty();

    // Each prototype is a nib of its own, in byte order of the reuse
    // identifiers. A cell's segue is a placeholder of the collection
    // view's nib, after the controller's, in document order.
    let mut cells: Vec<(&str, Node, Option<String>)> = Vec::new();
    let mut segues = Vec::new();
    for c in children(cv, "cells") {
        only_children(c, &["collectionViewCell"])?;
        for cell in children(c, "collectionViewCell") {
            let reuse = cell.attribute("reuseIdentifier").ok_or("a collection view cell without a reuse identifier is not supported by this ibtool")?;
            if cells.iter().any(|(r, ..)| *r == reuse) {
                return fail(format!("two prototypes reused as {reuse:?} are not supported by this ibtool"));
            }
            let segue = children(cell, "connections").flat_map(|cs| children(cs, "segue")).map(|sg| attr(sg, "id")).next().transpose()?;
            let placeholder = segue.map(|id| {
                segues.push(id.to_string());
                format!("upstream segue {}", segues.len() - 1)
            });
            cells.push((reuse, cell, placeholder));
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
    if let Some(c) = child(cv, "color", "backgroundColor") {
        values.push(("UIBackgroundColor", color(&mut nib, c)?));
    }
    values.push(("UIDeepDrawRect", Value::Bool(true)));
    values.push(("UIOpaque", Value::Bool(true)));
    if cv.attribute("multipleTouchEnabled").is_some() {
        values.push(("UIMultipleTouchEnabled", Value::Bool(true)));
    }
    values.push(("UIAutoresizeSubviews", Value::Bool(true)));
    let mask = match children(cv, "autoresizingMask").next() {
        Some(m) => autoresizing_mask(m)?,
        None => 0,
    };
    values.push(("UIAutoresizingMask", Value::Int(mask)));
    if cv.attribute("clipsSubviews").is_some() {
        values.push(("UIClipsToBounds", Value::Bool(true)));
    }
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    values.push(("UIScrollViewIndicatorInsetAdjustmentBehavior", Value::Int(1)));
    values.push(("UIBouncesZoom", Value::Bool(true)));
    values.push(("UIDelegate", Value::Nil));
    values.push(("UIScrollViewContentInsetAdjustmentBehavior", Value::Int(0)));
    let (layout, estimated) = flow_layout(cv)?;
    values.push(("UICollectionLayout", Value::New(layout.named("layout"))));
    values.push(("UIAllowsUserInitiatedMultipleSelection", Value::Bool(true)));
    if !cells.is_empty() {
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
        values.push(("UICollectionViewCellPrototypeNibExternalObjects", nib.array("NSMutableDictionary", external)));
        let mut nibs = Vec::new();
        for (reuse, c, placeholder) in &cells {
            let bytes = cell_nib(platform, color_matched, localizable, module, *c, placeholder.is_some())?;
            nibs.push(nib.string(reuse));
            let data = Value::New(Object::new("NSData", vec![("NS.bytes", Value::Data(bytes))]));
            nibs.push(Value::New(Object::new("UINib", vec![("captureEnclosingNIBBundleOnDecode", Value::Bool(true)), ("archiveData", data)])));
        }
        values.push(("UICollectionViewCellNibDict", nib.array("NSMutableDictionary", nibs)));
    }
    values.push(("UICollectionViewPrefetchingEnabled", Value::Bool(true)));
    top.push(Value::New(Object::new("UICollectionView", values).named(root_name.clone())));
    objects.push(reference(root_name.clone()));
    objects.push(reference("layout"));

    // Outlets, `view` among them, in byte order of their names.
    outlets.push(("view".to_string(), "owner", root_name.clone()));
    outlets.sort_by(|a, b| a.0.cmp(&b.0));
    let mut connections = Vec::new();
    for (label, source, destination) in &outlets {
        let source = if *source == "collection" { root_name.as_str() } else { source };
        connections.push(outlet(&mut nib, label, source, destination));
    }
    // An automatic estimated item size is set as the largest size when
    // the nib loads.
    let mut pairs = Vec::new();
    if estimated {
        let size = nib.string("{1.7976931348623157e+308, 1.7976931348623157e+308}");
        let value = Value::New(Object::new("NSValue", vec![("NS.special", Value::Int(2)), ("NS.sizeval", size)]));
        let key_path = nib.string("estimatedItemSize");
        pairs.push(Value::New(Object::new("UINibKeyValuePair", vec![("UIObject", reference("layout")), ("UIValue", value), ("UIKeyPath", key_path)])));
    }
    Ok((root_object_with_pairs(&mut nib, top, objects, connections, pairs), upstream_used, segues))
}

/// The collection view's flow layout, and whether it estimates its items'
/// sizes automatically.
fn flow_layout(cv: Node) -> Result<(Object, bool), String> {
    let layout = child(cv, "collectionViewFlowLayout", "collectionViewLayout").ok_or("a collection view without a flow layout is not supported by this ibtool")?;
    only_attributes(layout, &[
        ("key", Some("collectionViewLayout")), ("automaticEstimatedItemSize", Some("YES")), ("minimumLineSpacing", None),
        ("minimumInteritemSpacing", None), ("id", None),
    ])?;
    only_children(layout, &["size", "inset"])?;
    let mut item = None;
    for s in children(layout, "size") {
        let (w, h) = (number(attr(s, "width")?)?, number(attr(s, "height")?)?);
        match s.attribute("key") {
            Some("itemSize") => item = Some((w, h)),
            Some("headerReferenceSize" | "footerReferenceSize") if (w, h) == (0.0, 0.0) => {}
            _ => return fail("a flow layout's sizes other than its item size are not supported by this ibtool"),
        }
    }
    for i in children(layout, "inset") {
        let zero = ["minX", "minY", "maxX", "maxY"].iter().all(|k| i.attribute(*k).map(number) == Some(Ok(0.0)));
        if i.attribute("key") != Some("sectionInset") || !zero {
            return fail("a flow layout's section insets are not supported by this ibtool");
        }
    }
    let mut values = Vec::new();
    // Spacings of 10 are the default, left out; others are floats.
    let spacing = |key: &str| -> Result<f64, String> { layout.attribute(key).map_or(Ok(10.0), number) };
    let interitem = spacing("minimumInteritemSpacing")?;
    if interitem != 10.0 {
        values.push(("UIInteritemSpacing", Value::Float(interitem as f32)));
    }
    let line = spacing("minimumLineSpacing")?;
    if line != 10.0 {
        values.push(("UILineSpacing", Value::Float(line as f32)));
    }
    // An item size of 50x50 is the default, left out.
    let (w, h) = item.ok_or("a flow layout without an item size is not supported by this ibtool")?;
    if (w, h) != (50.0, 50.0) {
        values.push(("UIItemSize", doubles(&[w, h])));
    }
    Ok((Object::new("UICollectionViewFlowLayout", values), layout.attribute("automaticEstimatedItemSize").is_some()))
}

/// A prototype cell's nib: the cell, its content view and the content
/// view's subviews.
fn cell_nib(platform: Platform, color_matched: bool, localizable: bool, module: Option<&str>, cell: Node, has_segue: bool) -> Result<Vec<u8>, String> {
    only_attributes(cell, &[
        ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("multipleTouchEnabled", Some("YES")), ("contentMode", Some("center")),
        ("reuseIdentifier", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("id", None),
    ])?;
    only_children(cell, &["rect", "autoresizingMask", "collectionViewCellContentView", "view", "connections"])?;
    for required in ["opaque", "multipleTouchEnabled", "contentMode"] {
        if cell.attribute(required).is_none() {
            return fail(format!("a collection view cell without {required} is not supported by this ibtool"));
        }
    }
    let cell_name = view_name(attr(cell, "id")?);
    let content = cell
        .children()
        .find(|c| matches!(c.tag_name().name(), "collectionViewCellContentView" | "view") && c.attribute("key") == Some("contentView"))
        .ok_or("a collection view cell without a content view")?;
    only_attributes(content, &[
        ("key", Some("contentView")), ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("multipleTouchEnabled", Some("YES")),
        ("contentMode", Some("center")), ("insetsLayoutMarginsFromSafeArea", Some("NO")), ("id", None),
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
    // The old format's content view (a `<view>` without an id) is not the
    // document's: Xcode makes a plain one, which the objects leave out.
    let old = content.has_tag_name("view");
    if old && (content.attribute("id").is_some() || children(content, "subviews").next().is_some()) {
        return fail("a collection view cell's old-format content view with subviews is not supported by this ibtool");
    }
    let content_name = match content.attribute("id") {
        Some(id) => view_name(id),
        None => format!("{cell_name} content"),
    };

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
    // A content view holding constraints is pinned to its cell's edges by
    // constraints Xcode makes, which the objects leave out.
    let pinned = children(content, "constraints").any(|c| c.has_children());
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
    if old {
        cv.push(("UIOpaque", Value::Bool(true)));
        cv.push(("UIAutoresizeSubviews", Value::Bool(true)));
    } else {
        cv.push(("UIMultipleTouchEnabled", Value::Bool(true)));
        cv.push(("UIAutoresizeSubviews", Value::Bool(true)));
        cv.push(("UIContentMode", Value::Int(4)));
        if content.attribute("clipsSubviews").is_some() {
            cv.push(("UIClipsToBounds", Value::Bool(true)));
        }
        if pinned {
            cv.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
        }
        if let Some(own) = own_constraints(&mut nib, &mut scene, content, &content_name)? {
            cv.push(("UIViewAutolayoutConstraints", own));
        }
        if content.attribute("insetsLayoutMarginsFromSafeArea").is_some() {
            cv.push(("UIViewInsetsLayoutMarginsFromSafeArea", Value::Int(0)));
        }
    }
    cv.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut cv);
    let content_view = Value::New(Object::new("UIView", cv).named(content_name.clone()));

    let f = frame(cell)?;
    let mut values = Vec::new();
    geometry(&mut values, &f);
    values.push(("UISubviews", nib.array("NSMutableArray", vec![content_view])));
    values.push(("UIDeepDrawRect", Value::Bool(true)));
    values.push(("UIMultipleTouchEnabled", Value::Bool(true)));
    values.push(("UIAutoresizeSubviews", Value::Bool(true)));
    let mask = match children(cell, "autoresizingMask").next() {
        Some(m) => autoresizing_mask(m)?,
        None => 0,
    };
    values.push(("UIAutoresizingMask", Value::Int(mask)));
    values.push(("UIContentMode", Value::Int(4)));
    if cell.attribute("clipsSubviews").is_some() {
        values.push(("UIClipsToBounds", Value::Bool(true)));
    }
    if pinned {
        // Leading, trailing, top and bottom.
        let identifier = nib.string("UIIBSystemGenerated");
        let mut pins = vec![Value::New(edge(&cell_name, &content_name, 5, identifier))];
        for attribute in [6, 3, 4] {
            let identifier = nib.string("UIIBSystemGenerated");
            pins.push(Value::New(edge(&cell_name, &content_name, attribute, identifier)));
        }
        values.push(("UIViewAutolayoutConstraints", nib.array("NSMutableArray", pins)));
    }
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    values.push(("UIReuseIdentifier", nib.string(attr(cell, "reuseIdentifier")?)));
    values.push(("UISystemBackgroundView", Value::Nil));
    values.push(("UIContentConfigurationView", Value::Nil));
    values.push(("UIContentView", reference(content_name.clone())));
    let object = if cell.attribute("customClass").is_some() {
        let class = runtime_class_name(cell, module, "UICollectionViewCell")?;
        values.push(("UIClassName", nib.string(&class)));
        values.push(("UIOriginalClassName", nib.string("UICollectionViewCell")));
        Object::new("UIClassSwapper", values)
    } else {
        Object::new("UICollectionViewCell", values)
    };
    let mut top = vec![owner, responder];
    top.extend(upstream);
    top.push(Value::New(object.named(cell_name.clone())));

    // The objects: the cell and its content view; then, breadth first, each
    // view's constraints and its subviews.
    let mut objects = vec![reference("owner"), reference("responder")];
    if has_segue {
        objects.push(reference("upstream"));
    }
    objects.push(reference(cell_name.clone()));
    if !old {
        objects.push(reference(content_name.clone()));
    }
    let mut queue: std::collections::VecDeque<&String> = [&content_name].into();
    while let Some(v) = queue.pop_front() {
        objects.extend(scene.own_constraints.iter().filter(|c| &c.0 == v).map(|c| reference(c.1.clone())));
        for (_, s) in scene.subviews.iter().filter(|s| s.0.as_ref() == Some(v)) {
            objects.push(reference(s.clone()));
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

/// A constraint Xcode makes between the cell's and its content view's
/// edges.
fn edge(cell: &str, content: &str, attribute: i64, identifier: Value) -> Object {
    Object::new("NSLayoutConstraint", vec![
        ("NSFirstItem", reference(cell)),
        ("NSFirstAttributeV2", Value::Int(attribute)),
        ("NSFirstAttribute", Value::Int(attribute)),
        ("NSSecondItem", reference(content)),
        ("NSSecondAttributeV2", Value::Int(attribute)),
        ("NSSecondAttribute", Value::Int(attribute)),
        ("NSShouldBeArchived", Value::Bool(true)),
        ("NSLayoutIdentifier", identifier),
    ])
}
