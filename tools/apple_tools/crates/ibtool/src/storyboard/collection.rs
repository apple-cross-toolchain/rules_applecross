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
    only_children(cv, &["rect", "autoresizingMask", "color", "collectionViewFlowLayout", "collectionViewLayout", "cells", "collectionReusableView", "connections"])?;
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

    let (cells, segues) = prototypes(cv)?;

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

    let (values, estimated) = collection_values(&mut nib, module, cv, f, &cells, 0, Vec::new(), "layout")?;
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
    let pairs = if estimated { vec![estimated_item_size(&mut nib, "layout")] } else { Vec::new() };
    Ok((root_object_with_pairs(&mut nib, top, objects, connections, pairs), upstream_used, segues))
}

/// A collection view's prototype cells with their reuse identifiers, in
/// byte order of those, with the placeholder of each one's segue; and the
/// segues' ids. Each prototype is a nib of its own; a cell's segue is a
/// placeholder of the collection view's nib, after the controller's, in
/// document order.
#[allow(clippy::type_complexity)]
fn prototypes<'a, 'i>(cv: Node<'a, 'i>) -> Result<(Vec<(&'a str, Node<'a, 'i>, Option<String>)>, Vec<String>), String> {
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
    Ok((cells, segues))
}

/// The estimated item size an automatic one is set to as the nib loads:
/// the largest size, a key-value pair of the layout (`layout`).
fn estimated_item_size(nib: &mut Nib, layout: &str) -> Value {
    let size = nib.string("{1.7976931348623157e+308, 1.7976931348623157e+308}");
    let value = Value::New(Object::new("NSValue", vec![("NS.special", Value::Int(2)), ("NS.sizeval", size)]));
    let key_path = nib.string("estimatedItemSize");
    Value::New(Object::new("UINibKeyValuePair", vec![("UIObject", reference(layout)), ("UIValue", value), ("UIKeyPath", key_path)]))
}

/// A collection view's values, from its geometry to its prefetching:
/// `mask_default` when it gives no autoresizing mask, `layout_keys` (a
/// subview's constraint values) after its clipping, its flow layout named
/// `layout`; and whether that estimates its items' sizes automatically.
#[allow(clippy::too_many_arguments)]
fn collection_values(
    nib: &mut Nib, module: Option<&str>, cv: Node, f: &Frame, cells: &[(&str, Node, Option<String>)], mask_default: i64,
    layout_keys: Vec<(&'static str, Value)>, layout: &str,
) -> Result<(Vec<(&'static str, Value)>, bool), String> {
    let mut values = Vec::new();
    geometry(&mut values, f);
    // An opaque grey white is the system background colour, as a table's
    // is its own (with colorMatched).
    if let Some(c) = child(cv, "color", "backgroundColor") {
        let background = if nib.color_matched && opaque_grey_white(c)? {
            nib.system_color("systemBackgroundColor", &SystemColor::Gray([1.0, 1.0]))
        } else {
            color(nib, c)?
        };
        values.push(("UIBackgroundColor", background));
    }
    values.push(("UIDeepDrawRect", Value::Bool(true)));
    values.push(("UIOpaque", Value::Bool(true)));
    if cv.attribute("multipleTouchEnabled").is_some() {
        values.push(("UIMultipleTouchEnabled", Value::Bool(true)));
    }
    values.push(("UIAutoresizeSubviews", Value::Bool(true)));
    let mask = match children(cv, "autoresizingMask").next() {
        Some(m) => autoresizing_mask(m)?,
        None => mask_default,
    };
    values.push(("UIAutoresizingMask", Value::Int(mask)));
    if cv.attribute("clipsSubviews").is_some() {
        values.push(("UIClipsToBounds", Value::Bool(true)));
    }
    // A root collection view takes the document's tint.
    if cv.attribute("key") == Some("view") {
        if let Some(tint) = document_tint(cv)? {
            values.push(("UITintColor", color(nib, tint)?));
        }
    }
    values.extend(layout_keys);
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    values.push(("UIScrollViewIndicatorInsetAdjustmentBehavior", Value::Int(1)));
    // Hidden indicators, paging, then after `UIBouncesZoom` a vertical
    // bounce (four probes).
    for (attribute, key) in [("showsHorizontalScrollIndicator", "UIShowsHorizontalScrollIndicator"), ("showsVerticalScrollIndicator", "UIShowsVerticalScrollIndicator")] {
        if cv.attribute(attribute).is_some() {
            values.push((key, Value::Bool(false)));
        }
    }
    if cv.attribute("pagingEnabled").is_some() {
        values.push(("UIPagingEnabled", Value::Bool(true)));
    }
    values.push(("UIBouncesZoom", Value::Bool(true)));
    if cv.attribute("alwaysBounceVertical").is_some() {
        values.push(("UIAlwaysBounceVertical", Value::Bool(true)));
    }
    values.push(("UIDelegate", Value::Nil));
    values.push(("UIScrollViewContentInsetAdjustmentBehavior", Value::Int(0)));
    let (flow, estimated) = match child(cv, "collectionViewLayout", "collectionViewLayout") {
        Some(custom) => (custom_layout(nib, module, custom)?, false),
        None => flow_layout(nib, module, cv)?,
    };
    values.push(("UICollectionLayout", Value::New(flow.named(layout))));
    values.push(("UIAllowsUserInitiatedMultipleSelection", Value::Bool(true)));
    if !cells.is_empty() {
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
        values.push(("UICollectionViewCellPrototypeNibExternalObjects", nib.array("NSMutableDictionary", external)));
    }
    // Section headers and footers: by kind and reuse identifier, an empty
    // table of external objects after the cells' and a nib after theirs
    // (three probes).
    let mut supplementary = Vec::new();
    for v in children(cv, "collectionReusableView") {
        let kind = match v.attribute("key") {
            Some("sectionHeaderView") => "UICollectionElementKindSectionHeader",
            Some("sectionFooterView") => "UICollectionElementKindSectionFooter",
            _ => return fail("a collection reusable view other than a section header or footer is not supported by this ibtool"),
        };
        supplementary.push((format!("{kind}/{}", attr(v, "reuseIdentifier")?), v));
    }
    // In byte order of their keys (one probe of a header and a footer).
    supplementary.sort_by(|a, b| a.0.cmp(&b.0));
    if !supplementary.is_empty() {
        let mut external = Vec::new();
        for (key, _) in &supplementary {
            external.push(nib.string(key));
            external.push(nib.array("NSMutableDictionary", Vec::new()));
        }
        values.push(("UICollectionViewSupplementaryViewPrototypeNibExternalObjects", nib.array("NSMutableDictionary", external)));
    }
    if !cells.is_empty() {
        let mut nibs = Vec::new();
        for (reuse, c, placeholder) in cells {
            let bytes = cell_nib(nib.platform, nib.color_matched, nib.localizable, module, *c, placeholder.is_some())?;
            nibs.push(nib.string(reuse));
            let data = Value::New(Object::new("NSData", vec![("NS.bytes", Value::Data(bytes))]));
            nibs.push(Value::New(Object::new("UINib", vec![("captureEnclosingNIBBundleOnDecode", Value::Bool(true)), ("archiveData", data)])));
        }
        values.push(("UICollectionViewCellNibDict", nib.array("NSMutableDictionary", nibs)));
    }
    if !supplementary.is_empty() {
        let mut nibs = Vec::new();
        for (key, v) in &supplementary {
            let bytes = reusable_view_nib(nib.platform, nib.color_matched, nib.localizable, module, *v)?;
            nibs.push(nib.string(key));
            let data = Value::New(Object::new("NSData", vec![("NS.bytes", Value::Data(bytes))]));
            nibs.push(Value::New(Object::new("UINib", vec![("captureEnclosingNIBBundleOnDecode", Value::Bool(true)), ("archiveData", data)])));
        }
        values.push(("UICollectionViewSupplementaryViewNibDict", nib.array("NSMutableDictionary", nibs)));
    }
    // Prefetching, which tvOS leaves off (three probes and a corpus
    // storyboard).
    values.push(("UICollectionViewPrefetchingEnabled", Value::Bool(nib.platform != Platform::Tvos)));
    Ok((values, estimated))
}

/// A collection view as a subview of a controller's view: its cells' nibs
/// inside it as in a collection view controller's, its outlets to the
/// controller, its flow layout listed as its child. Its cells' segues are
/// not supported.
pub(super) fn subview_collection(nib: &mut Nib, scene: &mut Scene, n: Node, name: &str, f: &Frame) -> Result<Object, String> {
    only_attributes(n, &[
        ("clipsSubviews", Some("YES")), ("multipleTouchEnabled", Some("YES")), ("contentMode", Some("scaleToFill")), ("dataMode", Some("prototypes")),
        ("pagingEnabled", Some("YES")), ("alwaysBounceVertical", Some("YES")), ("showsHorizontalScrollIndicator", Some("NO")),
        ("showsVerticalScrollIndicator", Some("NO")), ("translatesAutoresizingMaskIntoConstraints", Some("NO")), ("id", None),
    ])?;
    only_children(n, &["rect", "autoresizingMask", "color", "collectionViewFlowLayout", "collectionViewLayout", "cells", "collectionReusableView", "connections", "constraints"])?;
    for c in children(n, "color") {
        if c.attribute("key") != Some("backgroundColor") {
            return fail(format!("a collection view's {} colour is not supported by this ibtool", c.attribute("key").unwrap_or("")));
        }
    }
    let (cells, segues) = prototypes(n)?;
    if !segues.is_empty() {
        return fail("segues from the cells of a collection view in a view are not supported by this ibtool");
    }
    for c in children(n, "connections") {
        only_children(c, &["outlet"])?;
        for o in children(c, "outlet") {
            only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
            if attr(o, "destination")? != scene.vc_id {
                return fail("a collection view's outlets to anything but its controller are not supported by this ibtool");
            }
            scene.control_outlets.push((attr(o, "property")?.to_string(), name.to_string()));
        }
    }
    let mut layout = Vec::new();
    if does_not_translate(scene, n) {
        layout.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
    }
    // Where the layout goes among constraints it holds isn't established.
    if children(n, "constraints").next().is_some() {
        return fail("constraints held by a collection view in a view are not supported by this ibtool");
    }
    let layout_name = format!("{name} layout");
    let (values, estimated) = collection_values(nib, scene.module, n, f, &cells, 36, layout, &layout_name)?;
    // The objects list its layout as its child.
    scene.subviews.push((Some(name.to_string()), layout_name.clone()));
    if estimated {
        let pair = estimated_item_size(nib, &layout_name);
        scene.pairs.push((layout_name, pair));
    }
    Ok(Object::new("UICollectionView", values))
}

/// The collection view's flow layout, and whether it estimates its items'
/// sizes automatically.
fn flow_layout(nib: &mut Nib, module: Option<&str>, cv: Node) -> Result<(Object, bool), String> {
    let layout = child(cv, "collectionViewFlowLayout", "collectionViewLayout").ok_or("a collection view without a flow layout is not supported by this ibtool")?;
    only_attributes(layout, &[
        ("key", Some("collectionViewLayout")), ("automaticEstimatedItemSize", Some("YES")), ("minimumLineSpacing", None),
        ("minimumInteritemSpacing", None), ("scrollDirection", Some("horizontal")), ("customClass", None), ("customModule", None),
        ("customModuleProvider", None), ("id", None),
    ])?;
    only_children(layout, &["size", "inset"])?;
    let mut item = None;
    let (mut header, mut footer) = (None, None);
    for s in children(layout, "size") {
        let (w, h) = (number(attr(s, "width")?)?, number(attr(s, "height")?)?);
        match s.attribute("key") {
            Some("itemSize") => item = Some((w, h)),
            Some("headerReferenceSize" | "footerReferenceSize") if (w, h) == (0.0, 0.0) => {}
            Some("headerReferenceSize") => header = Some((w, h)),
            Some("footerReferenceSize") => footer = Some((w, h)),
            _ => return fail("a flow layout's sizes other than its item size are not supported by this ibtool"),
        }
    }
    // Its section inset, top, left, bottom and right, unless zero (one
    // probe).
    let mut section_inset = None;
    for i in children(layout, "inset") {
        if i.attribute("key") != Some("sectionInset") {
            return fail("a flow layout's insets other than its section inset are not supported by this ibtool");
        }
        let edge = |k: &str| number(attr(i, k)?);
        let inset = [edge("minY")?, edge("minX")?, edge("maxY")?, edge("maxX")?];
        if inset != [0.0; 4] {
            section_inset = Some(inset);
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
    // Its header and footer reference sizes, unless zero, after its item
    // size (two probes); beside a section inset or a horizontal scroll
    // direction they aren't established.
    if (header.is_some() || footer.is_some()) && (section_inset.is_some() || layout.attribute("scrollDirection").is_some()) {
        return fail("a flow layout's header or footer size beside a section inset or horizontal scrolling is not supported by this ibtool");
    }
    for (size, key) in [(header, "UIHeaderReferenceSize"), (footer, "UIFooterReferenceSize")] {
        if let Some((w, h)) = size {
            values.push((key, doubles(&[w, h])));
        }
    }
    if let Some(inset) = section_inset {
        values.push(("UISectionInset", doubles(&inset)));
    }
    // A horizontal scroll direction after the item size and section inset
    // (three probes).
    if layout.attribute("scrollDirection").is_some() {
        values.push(("UIScrollDirection", Value::Int(1)));
    }
    let estimated = layout.attribute("automaticEstimatedItemSize").is_some();
    // A custom class is swapped in after its values (one probe).
    if layout.attribute("customClass").is_some() {
        let class = runtime_class_name(layout, module, "UICollectionViewFlowLayout")?;
        values.push(("UIClassName", nib.string(&class)));
        values.push(("UIOriginalClassName", nib.string("UICollectionViewFlowLayout")));
        return Ok((Object::new("UIClassSwapper", values), estimated));
    }
    Ok((Object::new("UICollectionViewFlowLayout", values), estimated))
}

/// A custom layout: its class swapped in for `UICollectionViewLayout`, with
/// nothing else (three probes).
fn custom_layout(nib: &mut Nib, module: Option<&str>, layout: Node) -> Result<Object, String> {
    only_attributes(layout, &[("key", Some("collectionViewLayout")), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("id", None)])?;
    only_children(layout, &[])?;
    let class = runtime_class_name(layout, module, "UICollectionViewLayout")?;
    let class_name = nib.string(&class);
    let original = nib.string("UICollectionViewLayout");
    Ok(Object::new("UIClassSwapper", vec![("UIClassName", class_name), ("UIOriginalClassName", original)]))
}

/// A prototype cell's nib: the cell, its content view and the content
/// view's subviews.
fn cell_nib(platform: Platform, color_matched: bool, localizable: bool, module: Option<&str>, cell: Node, has_segue: bool) -> Result<Vec<u8>, String> {
    only_attributes(cell, &[
        ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("multipleTouchEnabled", Some("YES")), ("contentMode", Some("center")),
        ("reuseIdentifier", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("id", None),
    ])?;
    only_children(cell, &["rect", "autoresizingMask", "collectionViewCellContentView", "view", "size", "color", "connections", "constraints"])?;
    if let Some(c) = children(cell, "color").find(|c| c.attribute("key") != Some("backgroundColor")) {
        return fail(format!("a collection view cell's {} colour is not supported by this ibtool", c.attribute("key").unwrap_or("unkeyed")));
    }
    // A custom size is Interface Builder's alone (one probe).
    for size in children(cell, "size") {
        if size.attribute("key") != Some("customSize") {
            return fail(format!("a collection view cell's {} size is not supported by this ibtool", size.attribute("key").unwrap_or("unkeyed")));
        }
    }
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
    // Its mask, when it has one, after `UIAutoresizeSubviews` (two probes).
    let mut mask = 0;
    for m in children(content, "autoresizingMask") {
        mask = autoresizing_mask(m)?;
        if m.has_children() || (mask != 0 && content.has_tag_name("view")) {
            return fail("a cell's content view with an autoresizing mask is not supported by this ibtool");
        }
    }
    // The old format's content view (a `<view>` without an id) is not the
    // document's: Xcode makes a plain one, which the objects leave out. The
    // constraints the cell holds are that view's, as is the cell's id in
    // them, and pin it to the cell (four probes).
    let old = content.has_tag_name("view");
    if old && content.attribute("id").is_some() {
        return fail("a collection view cell's old-format content view with an id is not supported by this ibtool");
    }
    let held = children(cell, "constraints").any(|c| c.has_children());
    if held && !old {
        return fail("constraints held by a collection view cell are not supported by this ibtool");
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
    // Its controls' actions may target the cell, as the scene's do its
    // controller.
    let cell_id = attr(cell, "id")?;
    let mut scene = Scene::new(cell_id, module, if old { cell } else { content }, Some(content_name.clone()));
    // A content view holding constraints is pinned to its cell's edges by
    // constraints Xcode makes, which the objects leave out.
    let pinned = held || children(content, "constraints").any(|c| c.has_children());
    let cf = frame(content)?;
    let mut cv = Vec::new();
    geometry(&mut cv, &cf);
    let mut subviews = Vec::new();
    for s in children(content, "subviews") {
        for c in s.children().filter(|c| c.is_element()) {
            subviews.push(subview(&mut nib, &mut scene, c)?);
        }
    }
    // These nibs don't list placeholder guides: a subview's own safe-area
    // guide or a scroll view's guides are refused here.
    if !scene.unused_guides.is_empty() {
        return fail("a safe-area or scroll view guide in a cell, header or table is not supported by this ibtool");
    }
    if scene.actions.iter().any(|a| a.3 != "upstream") || scene.actions.len() > 1 || !scene.segues.is_empty() || !scene.control_outlets.is_empty() || !scene.view_outlets.is_empty() {
        return fail("actions other than one on the cell, outlets and segues in a cell are not supported by this ibtool");
    }
    // A cell's nib has no key-value pairs or accessibility configurations
    // of its own here.
    if !scene.pairs.is_empty() || !scene.accessibility.is_empty() {
        return fail("runtime attributes, switches and accessibility settings in a cell are not supported by this ibtool");
    }
    if !subviews.is_empty() {
        cv.push(("UISubviews", nib.array("NSMutableArray", subviews)));
    }
    cv.push(("UIDeepDrawRect", Value::Bool(true)));
    if old {
        cv.push(("UIOpaque", Value::Bool(true)));
        cv.push(("UIAutoresizeSubviews", Value::Bool(true)));
        if pinned {
            cv.push(("UIViewDoesNotTranslateAutoresizingMaskIntoConstraints", Value::Bool(true)));
        }
        if let Some(own) = held_constraints(&mut nib, &mut scene, cell, content, &content_name)? {
            cv.push(("UIViewAutolayoutConstraints", own));
        }
    } else {
        cv.push(("UIMultipleTouchEnabled", Value::Bool(true)));
        cv.push(("UIAutoresizeSubviews", Value::Bool(true)));
        if mask != 0 {
            cv.push(("UIAutoresizingMask", Value::Int(mask)));
        }
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
    // Its background colour after its subviews (two probes).
    if let Some(c) = child(cell, "color", "backgroundColor") {
        values.push(("UIBackgroundColor", color(&mut nib, c)?));
    }
    values.push(("UIDeepDrawRect", Value::Bool(true)));
    values.push(("UIMultipleTouchEnabled", Value::Bool(true)));
    values.push(("UIAutoresizeSubviews", Value::Bool(true)));
    // Its mask, left out when empty (one corpus storyboard).
    let mask = match children(cell, "autoresizingMask").next() {
        Some(m) => autoresizing_mask(m)?,
        None => 0,
    };
    if mask != 0 {
        values.push(("UIAutoresizingMask", Value::Int(mask)));
    }
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
    // The old format's content view lists its subviews before its
    // constraints (two probes).
    let mut queue: std::collections::VecDeque<&String> = [&content_name].into();
    while let Some(v) = queue.pop_front() {
        let constraints = || scene.own_constraints.iter().filter(|c| &c.0 == v).map(|c| reference(c.1.clone()));
        let subviews_first = old && v == &content_name;
        if !subviews_first {
            objects.extend(constraints());
        }
        for (_, s) in scene.subviews.iter().filter(|s| s.0.as_ref() == Some(v)) {
            objects.push(reference(s.clone()));
            queue.push_back(s);
        }
        if subviews_first {
            objects.extend(constraints());
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
    let mut connections: Vec<Value> = outlets.iter().map(|(label, destination)| outlet(&mut nib, label, &cell_name, destination)).collect();
    // A control's action on the cell after its outlets (one probe).
    for (selector, source, mask, _) in &scene.actions {
        let label = nib.string(selector);
        connections.push(Value::New(Object::new("UIRuntimeEventConnection", vec![
            ("UILabel", label),
            ("UISource", reference(source.clone())),
            ("UIDestination", reference(cell_name.clone())),
            ("UIEventMask", Value::Int(*mask)),
        ])));
    }
    nib::archive(root_object(&mut nib, top, objects, connections))
}

/// A section header's or footer's nib: the owner, the first responder and
/// the view (`UICollectionReusableView`), its subviews compiled as a
/// container's, then the flags and its reuse identifier; the objects list a
/// safe-area guide after it (one probe). Its own constraints, outlets,
/// actions, colours and a custom class are refused.
fn reusable_view_nib(platform: Platform, color_matched: bool, localizable: bool, module: Option<&str>, v: Node) -> Result<Vec<u8>, String> {
    only_attributes(v, &[
        ("key", None), ("opaque", Some("NO")), ("clipsSubviews", Some("YES")), ("multipleTouchEnabled", Some("YES")), ("contentMode", Some("center")),
        ("reuseIdentifier", None), ("customClass", None), ("customModule", None), ("customModuleProvider", None), ("id", None),
    ])?;
    only_children(v, &["rect", "autoresizingMask", "subviews", "color", "constraints", "connections"])?;
    if let Some(c) = children(v, "color").find(|c| !matches!(c.attribute("key"), Some("backgroundColor" | "tintColor"))) {
        return fail(format!("a collection reusable view's {} colour is not supported by this ibtool", c.attribute("key").unwrap_or("unkeyed")));
    }
    for required in ["opaque", "multipleTouchEnabled", "contentMode"] {
        if v.attribute(required).is_none() {
            return fail(format!("a collection reusable view without {required} is not supported by this ibtool"));
        }
    }
    for m in children(v, "autoresizingMask") {
        if m.has_children() || autoresizing_mask(m)? != 0 {
            return fail("a collection reusable view with an autoresizing mask is not supported by this ibtool");
        }
    }
    let name = view_name(attr(v, "id")?);
    let mut nib = Nib::new(platform, color_matched, localizable);
    let owner = proxy(&mut nib, "IBFilesOwner", "owner");
    let responder = proxy(&mut nib, "IBFirstResponder", "responder");
    let mut scene = Scene::new(attr(v, "id")?, module, v, Some(name.clone()));
    let mut subviews = Vec::new();
    for s in children(v, "subviews") {
        for c in s.children().filter(|c| c.is_element()) {
            subviews.push(subview(&mut nib, &mut scene, c)?);
        }
    }
    // These nibs don't list placeholder guides: a subview's own safe-area
    // guide or a scroll view's guides are refused here.
    if !scene.unused_guides.is_empty() {
        return fail("a safe-area or scroll view guide in a cell, header or table is not supported by this ibtool");
    }
    if !scene.actions.is_empty() || !scene.segues.is_empty() || !scene.control_outlets.is_empty() || !scene.view_outlets.is_empty() || !scene.pairs.is_empty() || !scene.accessibility.is_empty() {
        return fail("actions, outlets, segues, runtime attributes and accessibility settings in a collection reusable view are not supported by this ibtool");
    }
    let f = frame(v)?;
    let mut values = Vec::new();
    geometry(&mut values, &f);
    if !subviews.is_empty() {
        values.push(("UISubviews", nib.array("NSMutableArray", subviews)));
    }
    // Its background colour after its subviews, its tint after clipping and
    // then its constraints; a custom class swaps it in after its reuse
    // identifier (one probe each, and one of all of them).
    if let Some(c) = child(v, "color", "backgroundColor") {
        values.push(("UIBackgroundColor", color(&mut nib, c)?));
    }
    values.push(("UIDeepDrawRect", Value::Bool(true)));
    values.push(("UIMultipleTouchEnabled", Value::Bool(true)));
    values.push(("UIAutoresizeSubviews", Value::Bool(true)));
    values.push(("UIContentMode", Value::Int(4)));
    if v.attribute("clipsSubviews").is_some() {
        values.push(("UIClipsToBounds", Value::Bool(true)));
    }
    if let Some(c) = child(v, "color", "tintColor") {
        values.push(("UITintColor", color(&mut nib, c)?));
    }
    if let Some(own) = own_constraints(&mut nib, &mut scene, v, &name)? {
        values.push(("UIViewAutolayoutConstraints", own));
    }
    values.push(("UIViewSemanticContentAttribute", Value::Int(0)));
    nib.large_content(&mut values);
    values.push(("UIReuseIdentifier", nib.string(attr(v, "reuseIdentifier")?)));
    let object = if v.attribute("customClass").is_some() {
        let class = runtime_class_name(v, module, "UICollectionReusableView")?;
        values.push(("UIClassName", nib.string(&class)));
        values.push(("UIOriginalClassName", nib.string("UICollectionReusableView")));
        Object::new("UIClassSwapper", values)
    } else {
        Object::new("UICollectionReusableView", values)
    };
    let top = vec![owner, responder, Value::New(object.named(name.clone()))];
    let identifier = nib.string("");
    let guide = Value::New(Object::new("UILayoutGuide", vec![
        ("UILayoutGuideOwningView", Value::Nil),
        ("UILayoutGuideIdentifier", identifier),
        ("UILayoutGuideOwningViewIsLocked", Value::Bool(false)),
        ("UILayoutGuideShouldBeArchived", Value::Bool(false)),
        ("UILayoutGuideAllowsNegativeDimensions", Value::Bool(false)),
    ]));
    let mut objects = vec![reference("owner"), reference("responder"), reference(name.clone()), guide];
    let mut queue: std::collections::VecDeque<String> = [name.clone()].into();
    while let Some(view) = queue.pop_front() {
        objects.extend(scene.own_constraints.iter().filter(|c| c.0 == view).map(|c| reference(c.1.clone())));
        for (_, s) in scene.subviews.iter().filter(|s| s.0.as_ref() == Some(&view)) {
            objects.push(reference(s.clone()));
            queue.push_back(s.clone());
        }
    }
    // Its outlets to its subviews, as a cell's, in byte order of their
    // names (one probe).
    let mut outlets = Vec::new();
    for c in children(v, "connections") {
        only_children(c, &["outlet"])?;
        for o in children(c, "outlet") {
            only_attributes(o, &[("property", None), ("destination", None), ("id", None)])?;
            let destination = view_name(attr(o, "destination")?);
            if !scene.subviews.iter().any(|s| s.1 == destination) {
                return fail("a collection reusable view's outlets to anything but its subviews are not supported by this ibtool");
            }
            outlets.push((attr(o, "property")?, destination));
        }
    }
    outlets.sort_by(|a, b| a.0.cmp(b.0));
    let connections = outlets.iter().map(|(label, destination)| outlet(&mut nib, label, &name, destination)).collect();
    nib::archive(root_object(&mut nib, top, objects, connections))
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
