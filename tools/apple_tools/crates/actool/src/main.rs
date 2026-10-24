//! Clean-room `actool` for Linux; see docs/clean-room/actool.md. Strict by
//! default: its output is Xcode's byte for byte, or the build fails.
//! Relaxed (RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed,
//! `--config=relaxed`), it writes a near match or hands what it doesn't
//! support to xcbuild's actool.

mod bom;
mod car;
mod directions;
mod dmp2;
mod icon;
mod iconset;
mod image;
mod imageset;
mod qsort;
mod stack;

use cf::Value;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Default)]
struct Options {
    compile: Option<PathBuf>,
    output_format: String,
    platform: String,
    minimum_deployment_target: String,
    app_icon: Option<String>,
    target_devices: Vec<String>,
    partial_info_plist: Option<PathBuf>,
    inputs: Vec<PathBuf>,
    /// `--compress-pngs`: a device build's bundle icons as CgBI PNGs.
    compress_pngs: bool,
    /// Whether output must be Xcode's byte for byte; see [`strict`].
    strict: bool,
}

/// Strict output, the default: byte-identical to Xcode's, or the build fails.
/// RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed (`--config=relaxed`)
/// accepts a near match.
fn strict() -> bool {
    std::env::var("RULES_APPLECROSS_RESOURCE_OUTPUT_MODE").map_or(true, |v| v != "relaxed")
}

fn parse(args: Vec<String>) -> Result<Options, String> {
    let mut o = Options { output_format: "xml1".into(), strict: strict(), ..Default::default() };
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "--compile" => o.compile = Some(PathBuf::from(value(&a)?)),
            "--output-format" => o.output_format = value(&a)?,
            "--platform" => o.platform = value(&a)?,
            "--minimum-deployment-target" => o.minimum_deployment_target = value(&a)?,
            "--app-icon" => o.app_icon = Some(value(&a)?),
            "--target-device" => o.target_devices.push(value(&a)?),
            "--output-partial-info-plist" => o.partial_info_plist = Some(PathBuf::from(value(&a)?)),
            "--compress-pngs" => o.compress_pngs = true,
            "--errors" | "--warnings" | "--notices" => {}
            "--alternate-app-icon" | "--launch-image" | "--accent-color" | "--widget-background-color" | "--sticker-pack-identifier-prefix"
            | "--stickers-icon-role" | "--filter-for-device-model" | "--filter-for-device-os-version" | "--product-type" | "--development-region" => {
                let v = value(&a)?;
                return Err(format!("{a} {v} is not supported by this actool"));
            }
            s if s.starts_with("--") => return Err(format!("{s} is not supported by this actool")),
            _ => o.inputs.push(PathBuf::from(a)),
        }
    }
    if o.compile.is_none() {
        return Err("--compile is required".into());
    }
    Ok(o)
}

/// CoreUI's 16-bit name identifier for an asset name (docs/clean-room/actool.md):
/// a 64-bit djb2 with seed 100823, folded by evaluating its four 16-bit
/// limbs, low first, as a base-33 polynomial.
fn name_identifier(name: &str) -> u16 {
    let mut h: u64 = 100_823;
    for &c in name.as_bytes() {
        h = h.wrapping_mul(33).wrapping_add(c as u64);
    }
    let limb = |k: u32| (h >> (16 * k)) & 0xffff;
    (limb(0).wrapping_mul(35937).wrapping_add(limb(1) * 1089).wrapping_add(limb(2) * 33).wrapping_add(limb(3)) & 0xffff) as u16
}

struct IconImage {
    file: PathBuf,
    filename: String,
    idiom: String,
    platform: Option<String>,
    size: String,
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// What a catalog holds, as far as this actool compiles it.
#[derive(Default)]
struct Scan {
    /// App icon sets by name.
    icon_sets: Vec<(String, PathBuf)>,
    /// tvOS brand assets and visionOS solid image stacks, by name.
    brand_assets: Vec<(String, PathBuf)>,
    solid_stacks: Vec<(String, PathBuf)>,
    /// Image sets by name.
    image_sets: Vec<(String, PathBuf)>,
}

fn has_entries(contents: &serde_json::Value, list: &str, field: &str) -> bool {
    contents[list].as_array().is_some_and(|items| items.iter().any(|i| !i[field].is_null()))
}

/// Lists a catalog's assets, refusing kinds this actool doesn't compile. A
/// color set without colour components compiles to nothing, as in Xcode.
fn scan(catalogs: &[PathBuf]) -> Result<Scan, String> {
    let mut found = Scan::default();
    for catalog in catalogs {
        scan_dir(catalog, "", true, &mut found)?;
    }
    Ok(found)
}

/// A catalog's directory, or a folder in it: image sets in folders are named
/// with the folder's name and a slash when it provides a namespace.
fn scan_dir(dir: &Path, prefix: &str, top: bool, found: &mut Scan) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .map(|e| e.map(|e| e.path()).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    entries.sort();
    for p in entries {
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        if name == "Contents.json" || name.starts_with('.') {
            continue;
        }
        match name.rsplit_once('.') {
            Some((stem, "imageset")) => found.image_sets.push((format!("{prefix}{stem}"), p)),
            Some((_, "colorset")) if !has_entries(&read_json(&p.join("Contents.json"))?, "colors", "color") => {}
            Some((stem, "appiconset")) if top => found.icon_sets.push((stem.to_string(), p)),
            Some((stem, "brandassets")) if top => found.brand_assets.push((stem.to_string(), p)),
            Some((stem, "solidimagestack")) if top => found.solid_stacks.push((stem.to_string(), p)),
            None if p.is_dir() => {
                // A folder: its Contents.json may say it provides a namespace.
                let contents = p.join("Contents.json");
                let mut namespace = false;
                if contents.is_file() {
                    let j = read_json(&contents)?;
                    for (k, v) in j.as_object().into_iter().flatten() {
                        match (k.as_str(), v) {
                            ("info", _) => {}
                            ("properties", serde_json::Value::Object(props)) => {
                                for (pk, pv) in props {
                                    match (pk.as_str(), pv.as_bool()) {
                                        ("provides-namespace", Some(b)) => namespace = b,
                                        _ => return Err(format!("{}: the folder property {pk} is not supported by this actool", p.display())),
                                    }
                                }
                            }
                            _ => return Err(format!("{}: the {k} key of a folder is not supported by this actool", p.display())),
                        }
                    }
                }
                let inner = if namespace { format!("{prefix}{name}/") } else { prefix.to_string() };
                scan_dir(&p, &inner, false, found)?;
            }
            _ => return Err(format!("{}: this kind of asset is not supported by this actool yet", p.display())),
        }
    }
    Ok(())
}

fn xcode_identity() -> String {
    // "Xcode 27.0 (27A266a)", from the Xcode whose SDK the build targets.
    let developer = std::env::var("DEVELOPER_DIR").unwrap_or_else(|_| "/Applications/Xcode.app/Contents/Developer".into());
    let plist = Path::new(&developer).join("../version.plist");
    let v = std::fs::read(&plist).ok().and_then(|b| Value::parse(&b).ok());
    let short = v.as_ref().and_then(|v| v.get("CFBundleShortVersionString")).and_then(|x| x.as_str()).unwrap_or("27.0").to_string();
    let build = v.as_ref().and_then(|v| v.get("ProductBuildVersion")).and_then(|x| x.as_str()).unwrap_or("27A266a").to_string();
    format!("Xcode {short} ({build})")
}

fn platform_name(platform: &str) -> Result<&'static str, String> {
    match platform {
        "iphoneos" | "iphonesimulator" => Ok("ios"),
        "watchsimulator" => Ok("watch"),
        "macosx" => Ok("macosx"),
        "appletvos" | "appletvsimulator" => Ok("atv"),
        "xros" | "xrsimulator" => Ok("vision"),
        other => Err(format!("--platform {other} is not supported by this actool")),
    }
}

fn icon_images(icon_dir: &Path) -> Result<Vec<IconImage>, String> {
    let contents = read_json(&icon_dir.join("Contents.json"))?;
    Ok(contents["images"]
        .as_array()
        .ok_or_else(|| format!("{}: app icon set has no images list", icon_dir.display()))?
        .iter()
        .filter_map(|i| {
            Some(IconImage {
                filename: i["filename"].as_str()?.to_string(),
                file: icon_dir.join(i["filename"].as_str()?),
                idiom: i["idiom"].as_str().unwrap_or("universal").to_string(),
                platform: i["platform"].as_str().map(String::from),
                size: i["size"].as_str().unwrap_or("").to_string(),
            })
        })
        .collect())
}

enum IconSource {
    /// A single universal 1024 px image (its file name), scaled for each size.
    Single(String, image::Rgba),
    /// An image per size.
    PerSize(Vec<iconset::Slot>),
}

/// An app icon set's images; None for a set with no images, which compiles
/// to nothing.
fn app_icon_set(icon_dir: &Path) -> Result<Option<IconSource>, String> {
    let images = icon_images(icon_dir)?;
    if images.is_empty() {
        return Ok(None);
    }
    let universal: Vec<&IconImage> = images.iter().filter(|i| i.idiom == "universal" && i.platform.as_deref() == Some("ios") && i.size == "1024x1024").collect();
    if universal.len() != 1 || images.len() != 1 {
        let contents = read_json(&icon_dir.join("Contents.json"))?;
        return Ok(Some(IconSource::PerSize(iconset::slots(icon_dir, &contents["images"])?)));
    }
    let icon = universal[0];
    let src = image::decode_png(&std::fs::read(&icon.file).map_err(|e| format!("{}: {e}", icon.file.display()))?)?;
    if src.width != 1024 || src.height != 1024 {
        return Err(format!("{}: is {}x{} but should be 1024x1024", icon.file.display(), src.width, src.height));
    }
    Ok(Some(IconSource::Single(icon.filename.clone(), src)))
}

fn write_partial_info_plist(o: &Options, entries: Vec<(&str, Value)>, outputs: &mut Vec<PathBuf>) -> Result<(), String> {
    if let Some(p) = &o.partial_info_plist {
        std::fs::write(p, cf::xml::to_bytes(&Value::dict(entries))).map_err(|e| e.to_string())?;
        outputs.push(p.clone());
    }
    Ok(())
}

fn run(o: &Options) -> Result<Vec<PathBuf>, String> {
    let out_dir = o.compile.clone().unwrap();
    if !out_dir.is_dir() {
        return Err(format!("The output directory \"{}\" does not exist.", out_dir.display()));
    }
    let platform = platform_name(&o.platform)?;
    let devices: Vec<&str> = o.target_devices.iter().map(String::as_str).collect();
    let phone = devices.contains(&"iphone");
    let pad = devices.contains(&"ipad");
    let mut outputs = Vec::new();

    // An Icon Composer document is an input of its own; catalogs are the rest.
    let is_document = |p: &PathBuf| p.extension().is_some_and(|e| e == "icon");
    let catalogs: Vec<PathBuf> = o.inputs.iter().filter(|p| !is_document(p)).cloned().collect();
    let scan = scan(&catalogs)?;
    let document = o.inputs.iter().find(|p| is_document(p) && o.app_icon.as_deref().is_some_and(|n| p.file_stem().is_some_and(|s| s.to_string_lossy() == n)));
    if let Some(p) = o.inputs.iter().find(|p| is_document(p) && Some(*p) != document) {
        return Err(format!("{}: only the app icon's own Icon Composer document is supported by this actool", p.display()));
    }
    let images = imageset::compile(&scan.image_sets, name_identifier)?;
    if !images.is_empty() && platform != "ios" {
        return Err("image sets are supported by this actool for iOS only".into());
    }
    if !images.is_empty() && (!scan.brand_assets.is_empty() || !scan.solid_stacks.is_empty()) {
        return Err("image sets alongside image stacks are not supported by this actool".into());
    }
    if let Some(outputs) = stack_icon(o, &out_dir, platform, &scan, &mut outputs)? {
        return Ok(outputs);
    }
    for (name, dir) in &scan.icon_sets {
        if o.app_icon.as_deref() != Some(name.as_str()) && !icon_images(dir)?.is_empty() {
            return Err(format!("{}: app icon sets other than --app-icon's are not supported by this actool yet", dir.display()));
        }
    }
    let icon = match (&o.app_icon, document) {
        (Some(name), Some(doc)) => {
            if scan.icon_sets.iter().any(|(n, _)| n == name) {
                return Err(format!("{name}: both an Icon Composer document and an app icon set; this actool compiles one"));
            }
            if o.strict {
                // Xcode's own output names renditions with a fresh UUID each
                // run, and this one is a flat rendering: never Xcode's.
                return Err(format!("{}: Icon Composer documents are rendered flat by this actool, not as Xcode compiles them (docs/clean-room/actool.md)", doc.display()));
            }
            Some((name.clone(), IconSource::Single(format!("{name}1024x1024.png"), icon::render(doc)?)))
        }
        (Some(name), None) => {
            let (_, dir) = scan.icon_sets.iter().find(|(n, _)| n == name).ok_or_else(|| format!("none of the input catalogs contains an app icon set named \"{name}\""))?;
            app_icon_set(dir)?.map(|source| (name.clone(), source))
        }
        (None, _) => None,
    };
    let Some((icon_name, source)) = icon else {
        if !images.is_empty() {
            return finish(o, &out_dir, platform, None, false, Vec::new(), images, Kind::Ios { phone_files: &[], pad_files: &[] }, outputs);
        }
        // Nothing to compile: no Assets.car, an empty partial Info.plist.
        write_partial_info_plist(o, Vec::new(), &mut outputs)?;
        return Ok(outputs);
    };
    let (filename, src) = match source {
        IconSource::Single(filename, src) => (filename, src),
        IconSource::PerSize(slots) => return per_size_icon(o, &out_dir, platform, &icon_name, &slots, phone, pad, outputs, images),
    };

    let bgra = src.to_bgra();
    // CgBI for a device build with --compress-pngs (as rules_apple passes);
    // otherwise the PNG a simulator build gets.
    let cgbi = !o.platform.ends_with("simulator") && o.compress_pngs;
    let bundle_png = |size: usize| -> Result<Vec<u8>, String> {
        let resized = image::resize(&bgra, size, size);
        if cgbi {
            return Ok(image::cgbi_png(&resized));
        }
        let rgba = resized.opaque_rgba().ok_or("a translucent app icon is not supported by this actool for simulators")?;
        Ok(image::plain_png(&rgba, true))
    };

    // The bundle's flat icons: 60pt@2x for iPhone, 76pt@2x for iPad.
    let mut phone_files = Vec::new();
    let mut pad_files = Vec::new();
    if phone {
        let p = out_dir.join(format!("{icon_name}60x60@2x.png"));
        std::fs::write(&p, bundle_png(120)?).map_err(|e| e.to_string())?;
        outputs.push(p);
        phone_files.push(format!("{icon_name}60x60"));
    }
    if pad {
        let p = out_dir.join(format!("{icon_name}76x76@2x~ipad.png"));
        std::fs::write(&p, bundle_png(152)?).map_err(|e| e.to_string())?;
        outputs.push(p);
        pad_files.push(format!("{icon_name}60x60"));
        pad_files.push(format!("{icon_name}76x76"));
    }

    // Assets.car: per idiom, the 1024 px image and the index of its sizes.
    let id = name_identifier(&icon_name);
    let image_csi = car::icon_image_csi(&filename, 1024, 1024, 1, &bgra.data, car::chunked_version(platform));
    let multi_csi = car::multisized_csi(&icon_name, &[(1024, 1024, 1)]);
    let rendition = |idiom: u16, part: u16, dim2: u16, csi: &Vec<u8>| car::Rendition {
        attrs: vec![(car::ATTR_SCALE, 1), (car::ATTR_IDIOM, idiom), (car::ATTR_DIMENSION2, dim2), (car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, part)],
        csi: csi.clone(),
    };
    let multi = |idiom| rendition(idiom, car::PART_MULTISIZED_IMAGE, 0, &multi_csi);
    let img = |idiom| rendition(idiom, car::PART_ICON_IMAGE, 1, &image_csi);
    let mut renditions = Vec::new();
    for (wanted, idiom) in [(phone, car::IDIOM_PHONE), (pad, car::IDIOM_PAD)] {
        if wanted {
            renditions.push(img(idiom));
            renditions.push(multi(idiom));
        }
    }
    if renditions.is_empty() {
        return Err("--target-device iphone and/or ipad is required".into());
    }
    finish(o, &out_dir, platform, Some(&icon_name), false, renditions, images, Kind::Ios { phone_files: &phone_files, pad_files: &pad_files }, outputs)
}

/// A tvOS or visionOS app icon from an image stack, when --app-icon names
/// one; None otherwise. Other stacks are refused.
fn stack_icon(o: &Options, out_dir: &Path, platform: &str, scan: &Scan, outputs: &mut Vec<PathBuf>) -> Result<Option<Vec<PathBuf>>, String> {
    let wanted = |list: &[(String, PathBuf)]| list.iter().find(|(n, _)| o.app_icon.as_deref() == Some(n.as_str())).map(|(_, d)| d.clone());
    for (_, dir) in scan.brand_assets.iter().chain(&scan.solid_stacks) {
        if o.app_icon.as_deref() != dir.file_stem().and_then(|s| s.to_str()) {
            return Err(format!("{}: image stacks other than --app-icon's are not supported by this actool", dir.display()));
        }
    }
    let devices = |d: &str| o.target_devices.len() == 1 && o.target_devices[0] == d;
    let (stack, renditions, tv) = if let Some(dir) = wanted(&scan.brand_assets) {
        if platform != "atv" || !devices("tv") {
            return Err(format!("{}: brand assets are supported by this actool for tvOS, --target-device tv", dir.display()));
        }
        let contents = read_json(&dir.join("Contents.json"))?;
        let assets = contents["assets"].as_array().cloned().unwrap_or_default();
        let [asset] = &assets[..] else {
            return Err(format!("{}: brand assets other than one primary app icon are not supported by this actool", dir.display()));
        };
        let file = asset["filename"].as_str().unwrap_or("");
        if asset["role"].as_str() != Some("primary-app-icon") || asset["idiom"].as_str() != Some("tv") || !file.ends_with(".imagestack") {
            return Err(format!("{}: brand assets other than one primary app icon are not supported by this actool", dir.display()));
        }
        let size = asset["size"].as_str().unwrap_or("");
        let (w, h) = size.split_once('x').and_then(|(w, h)| Some((w.parse::<usize>().ok()?, h.parse::<usize>().ok()?))).ok_or_else(|| format!("{}: size {size:?}", dir.display()))?;
        let stack = stack::read(&dir.join(file), "imagestacklayer", "tv", "1x")?;
        let renditions = stack::tv(&stack, w, h, o.strict)?;
        (stack, renditions, true)
    } else if let Some(dir) = wanted(&scan.solid_stacks) {
        if platform != "vision" || !devices("vision") {
            return Err(format!("{}: solid image stacks are supported by this actool for visionOS, --target-device vision", dir.display()));
        }
        let stack = stack::read(&dir, "solidimagestacklayer", "reality", "2x")?;
        let renditions = stack::vision(&stack)?;
        (stack, renditions, false)
    } else {
        return Ok(None);
    };
    if !scan.icon_sets.is_empty() {
        return Err("app icon sets alongside an image stack app icon are not supported by this actool".into());
    }
    let id = name_identifier(&stack.name);
    let mut facets = vec![car::Facet { name: stack.name.clone(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_STACK), (car::ATTR_IDENTIFIER, id)] }];
    for l in &stack.layers {
        let lid = name_identifier(&l.facet);
        let attrs = if tv {
            vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_STACK), (car::ATTR_IDENTIFIER, lid)]
        } else {
            vec![(car::ATTR_ELEMENT, car::ELEMENT_TEXTURE), (car::ATTR_PART, car::PART_STACK), (car::ATTR_IDENTIFIER, lid), (car::ATTR_IDIOM, car::IDIOM_VISION)]
        };
        facets.push(car::Facet { name: l.facet.clone(), attrs });
    }
    let mut key_format = vec![car::ATTR_APPEARANCE, car::ATTR_LOCALIZATION, car::ATTR_SCALE, car::ATTR_IDIOM, car::ATTR_SUBTYPE];
    if !tv {
        key_format.push(car::ATTR_DIMENSION1);
    }
    key_format.extend([car::ATTR_IDENTIFIER, car::ATTR_ELEMENT, car::ATTR_PART]);
    let catalog = car::Catalog {
        key_format,
        facets,
        appearances: vec![("UIAppearanceAny".into(), 0)],
        schema_version: if tv { 2 } else { 1 },
        key_semantics: if tv { 2 } else { 4 },
        renditions,
        xcode: xcode_identity(),
        deployment_target: o.minimum_deployment_target.clone(),
        platform: platform.into(),
    };
    let car_path = out_dir.join("Assets.car");
    std::fs::write(&car_path, catalog.to_bytes()).map_err(|e| e.to_string())?;
    let mut outputs = std::mem::take(outputs);
    outputs.push(car_path);
    let entries = vec![("CFBundleIcons", Value::dict([("CFBundlePrimaryIcon", Value::string(stack.name.clone()))]))];
    write_partial_info_plist(o, entries, &mut outputs)?;
    Ok(Some(outputs))
}

/// What an app icon's catalog and partial Info.plist are for.
enum Kind<'a> {
    Ios { phone_files: &'a [String], pad_files: &'a [String] },
    Watch,
    Mac,
}

/// Writes Assets.car and the partial Info.plist of an app icon.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
fn finish(o: &Options, out_dir: &Path, platform: &str, icon_name: Option<&str>, has_dimension1: bool, mut renditions: Vec<car::Rendition>, images: imageset::Compiled, kind: Kind, mut outputs: Vec<PathBuf>) -> Result<Vec<PathBuf>, String> {
    renditions.extend(images.renditions);
    let has_dimension2 = renditions.iter().any(|r| r.attrs.iter().any(|a| a.0 == car::ATTR_DIMENSION2));
    let has_dimension1 = has_dimension1 || renditions.iter().any(|r| r.attrs.iter().any(|a| a.0 == car::ATTR_DIMENSION1 && a.1 != 0));
    let key_format = if matches!(kind, Kind::Mac) {
        vec![car::ATTR_APPEARANCE, car::ATTR_LOCALIZATION, car::ATTR_ELEMENT, car::ATTR_PART, car::ATTR_SIZE, car::ATTR_IDENTIFIER, car::ATTR_DIMENSION2, car::ATTR_LAYER, car::ATTR_SCALE]
    } else {
        let mut k = vec![car::ATTR_APPEARANCE, car::ATTR_LOCALIZATION, car::ATTR_SCALE, car::ATTR_IDIOM, car::ATTR_SUBTYPE];
        if has_dimension2 {
            k.push(car::ATTR_DIMENSION2);
        }
        if has_dimension1 {
            k.push(car::ATTR_DIMENSION1);
        }
        k.extend([car::ATTR_IDENTIFIER, car::ATTR_ELEMENT, car::ATTR_PART]);
        k
    };
    let mac = matches!(kind, Kind::Mac);
    let catalog = car::Catalog {
        key_format,
        facets: icon_name
            .map(|n| car::Facet { name: n.to_string(), attrs: vec![(car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_ICON_IMAGE), (car::ATTR_IDENTIFIER, name_identifier(n))] })
            .into_iter()
            .chain(images.facets)
            .collect(),
        appearances: if mac { Vec::new() } else { vec![("UIAppearanceAny".into(), 0)] },
        schema_version: 2,
        key_semantics: if mac { 1 } else { 2 },
        renditions,
        xcode: xcode_identity(),
        deployment_target: o.minimum_deployment_target.clone(),
        platform: platform.into(),
    };
    let car_path = out_dir.join("Assets.car");
    std::fs::write(&car_path, catalog.to_bytes()).map_err(|e| e.to_string())?;
    outputs.push(car_path);
    outputs.sort();

    let Some(icon_name) = icon_name else {
        write_partial_info_plist(o, Vec::new(), &mut outputs)?;
        return Ok(outputs);
    };
    let primary = |files: &[String]| {
        Value::dict([("CFBundlePrimaryIcon", Value::dict([
            ("CFBundleIconFiles", Value::Array(files.iter().map(|f| Value::string(f.clone())).collect())),
            ("CFBundleIconName", Value::string(icon_name)),
        ]))])
    };
    let entries = match kind {
        Kind::Ios { phone_files, pad_files } => {
            let mut e = Vec::new();
            if !phone_files.is_empty() {
                e.push(("CFBundleIcons", primary(phone_files)));
            }
            if !pad_files.is_empty() {
                e.push(("CFBundleIcons~ipad", primary(pad_files)));
            }
            e
        }
        // A watch app's icons are all in the catalog.
        Kind::Watch => vec![("CFBundleIcons", Value::dict([("CFBundlePrimaryIcon", Value::dict([("CFBundleIconName", Value::string(icon_name))]))]))],
        Kind::Mac => vec![("CFBundleIconFile", Value::string(icon_name)), ("CFBundleIconName", Value::string(icon_name))],
    };
    write_partial_info_plist(o, entries, &mut outputs)?;
    Ok(outputs)
}

/// An app icon set with an image per size: packed into atlases, the bundle's
/// icons copied as given (simulators and macOS only: devices' CgBI
/// conversion of them is not established).
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
fn per_size_icon(o: &Options, out_dir: &Path, platform: &str, icon_name: &str, slots: &[iconset::Slot], phone: bool, pad: bool, mut outputs: Vec<PathBuf>, images: imageset::Compiled) -> Result<Vec<PathBuf>, String> {
    let watch = o.target_devices.iter().any(|d| d == "watch");
    let mac = o.target_devices.iter().any(|d| d == "mac");
    match (o.platform.as_str(), phone || pad, watch, mac) {
        ("iphonesimulator", true, false, false) | ("watchsimulator", false, true, false) | ("macosx", false, false, true) => {}
        _ => return Err(format!("this actool compiles icon sets with an image per size for the iOS and watchOS simulators and macOS, with their own target devices, not --platform {}", o.platform)),
    }
    let compiled = iconset::compile(icon_name, name_identifier(icon_name), slots, phone, pad, watch, mac)?;
    for (name, source) in &compiled.bundle_files {
        let p = out_dir.join(name);
        std::fs::copy(source, &p).map_err(|e| format!("{}: {e}", source.display()))?;
        outputs.push(p);
    }
    if let Some(icns) = &compiled.icns {
        let p = out_dir.join(format!("{icon_name}.icns"));
        std::fs::write(&p, icns).map_err(|e| e.to_string())?;
        outputs.push(p);
    }
    let kind = if mac {
        Kind::Mac
    } else if watch {
        Kind::Watch
    } else {
        Kind::Ios { phone_files: &compiled.phone_files, pad_files: &compiled.pad_files }
    };
    finish(o, out_dir, platform, Some(icon_name), compiled.has_dimension1, compiled.renditions, images, kind, outputs)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version") {
        print!("{}", cf::xml::to_string(&Value::dict([("com.apple.actool.version", Value::dict([
            ("bundle-version", Value::string("24303")), ("short-bundle-version", Value::string("27.0"))]))])));
        return ExitCode::SUCCESS;
    }
    let o = match parse(args.clone()) {
        Ok(o) => o,
        Err(e) => return refuse(&args, &e),
    };
    match run(&o) {
        Ok(outputs) => {
            if o.output_format == "human-readable-text" {
                println!("/* com.apple.actool.compilation-results */");
                for p in outputs {
                    println!("{}", p.display());
                }
                println!();
            }
            ExitCode::SUCCESS
        }
        Err(e) => refuse(&args, &e),
    }
}

/// An input this actool can't compile as Xcode would: an error when strict,
/// xcbuild's actool when relaxed.
fn refuse(args: &[String], reason: &str) -> ExitCode {
    if strict() {
        eprintln!("error: {reason}");
        eprintln!("note: strict output must be byte-identical to Xcode's; build with --config=relaxed (RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed) to accept a near match");
        return ExitCode::FAILURE;
    }
    eprintln!("note: {reason}; compiling with xcbuild's actool instead");
    ExitCode::from(xcbuild_acdriver::run(args).clamp(0, 255) as u8)
}
