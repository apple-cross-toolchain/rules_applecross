//! Clean-room `actool` for Linux; see docs/clean-room/actool.md. Strict by
//! default: its output is Xcode's byte for byte, or the build fails.
//! Relaxed (RULES_APPLECROSS_RESOURCE_OUTPUT_MODE=relaxed,
//! `--config=relaxed`), it writes a near match or hands what it doesn't
//! support to xcbuild's actool.

mod bom;
mod car;
mod colorset;
mod colour;
mod dataset;
mod directions;
mod dmp2;
mod icon;
mod iconset;
mod image;
mod imageset;
mod launchimage;
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
    /// `--launch-image`: the launch image set the bundle gets.
    launch_image: Option<String>,
    /// That set's `UILaunchImages`, for the partial Info.plist.
    launch_entries: std::cell::RefCell<Vec<Value>>,
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
            "--launch-image" => o.launch_image = Some(value(&a)?),
            "--alternate-app-icon" | "--accent-color" | "--widget-background-color" | "--sticker-pack-identifier-prefix"
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
    /// Its appearance: none, or `luminosity` dark or tinted, or something
    /// else (`None`).
    appearance: Option<u16>,
}

pub(crate) fn read_json(path: &Path) -> Result<serde_json::Value, String> {
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
    /// Colour sets with colours, by name.
    color_sets: Vec<(String, PathBuf)>,
    /// Data sets by name.
    data_sets: Vec<(String, PathBuf)>,
    /// Launch image sets by name.
    launch_images: Vec<(String, PathBuf)>,
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
        // Xcode ignores files that aren't assets, such as a README beside
        // them (three probes).
        if name == "Contents.json" || name.starts_with('.') || p.is_file() {
            continue;
        }
        // It ignores image, colour and data sets without a Contents.json too
        // (three probes); an app icon set without one is an empty set.
        if [".imageset", ".colorset", ".dataset"].iter().any(|e| name.ends_with(e)) && !p.join("Contents.json").is_file() {
            continue;
        }
        match name.rsplit_once('.') {
            Some((stem, "imageset")) => found.image_sets.push((format!("{prefix}{stem}"), p)),
            Some((_, "colorset")) if !has_entries(&read_json(&p.join("Contents.json"))?, "colors", "color") => {}
            Some((stem, "colorset")) => found.color_sets.push((format!("{prefix}{stem}"), p)),
            Some((stem, "dataset")) => found.data_sets.push((format!("{prefix}{stem}"), p)),
            Some((stem, "appiconset")) if top => found.icon_sets.push((stem.to_string(), p)),
            Some((stem, "brandassets")) if top => found.brand_assets.push((stem.to_string(), p)),
            Some((stem, "solidimagestack")) if top => found.solid_stacks.push((stem.to_string(), p)),
            Some((stem, "launchimage")) if top => found.launch_images.push((stem.to_string(), p)),
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
    // A set without a Contents.json has no images (one probe).
    if !icon_dir.join("Contents.json").is_file() {
        return Ok(Vec::new());
    }
    let contents = read_json(&icon_dir.join("Contents.json"))?;
    // No images list is as an empty one (one probe).
    Ok(contents["images"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        // An image whose file is missing is left out (two probes).
        .filter(|i| i["filename"].as_str().is_some_and(|f| icon_dir.join(f).is_file()))
        .filter_map(|i| {
            Some(IconImage {
                filename: i["filename"].as_str()?.to_string(),
                file: icon_dir.join(i["filename"].as_str()?),
                idiom: i["idiom"].as_str().unwrap_or("universal").to_string(),
                platform: i["platform"].as_str().map(String::from),
                size: i["size"].as_str().unwrap_or("").to_string(),
                appearance: match i["appearances"].as_array().map(Vec::as_slice) {
                    None | Some([]) => Some(0),
                    Some([a]) => match (a["appearance"].as_str(), a["value"].as_str()) {
                        (Some("luminosity"), Some("dark")) => Some(1),
                        (Some("luminosity"), Some("tinted")) => Some(TINTED),
                        _ => None,
                    },
                    Some(_) => None,
                },
            })
        })
        .collect())
}

/// The appearance value of an app icon's tinted image.
const TINTED: u16 = 10;

enum IconSource {
    /// A single universal 1024 px image (its file name), scaled for each
    /// size, and its dark and tinted images (appearance, file name, image).
    Single(String, image::Rgba, Vec<(u16, String, image::Rgba)>),
    /// An image per size.
    PerSize(Vec<iconset::Slot>),
}

/// An app icon set's images; None for a set with no images, which compiles
/// to nothing. Building for iOS, Xcode ignores a set's macOS images (and
/// compiles a set of only those to nothing).
fn app_icon_set(icon_dir: &Path, ios: bool) -> Result<Option<IconSource>, String> {
    let images = icon_images(icon_dir)?;
    if images.is_empty() || (ios && images.iter().all(|i| i.idiom == "mac")) {
        return Ok(None);
    }
    let universal: Vec<&IconImage> = images.iter().filter(|i| i.idiom == "universal" && i.platform.as_deref() == Some("ios") && i.size == "1024x1024").collect();
    let others = images.iter().filter(|i| !(ios && i.idiom == "mac")).count();
    let any: Vec<&&IconImage> = universal.iter().filter(|i| i.appearance == Some(0)).collect();
    if any.len() != 1 || others != universal.len() || universal.iter().any(|i| i.appearance.is_none()) {
        let contents = read_json(&icon_dir.join("Contents.json"))?;
        return Ok(Some(IconSource::PerSize(iconset::slots(icon_dir, &contents["images"])?)));
    }
    let read = |icon: &IconImage| -> Result<image::Rgba, String> {
        let bytes = std::fs::read(&icon.file).map_err(|e| format!("{}: {e}", icon.file.display()))?;
        let src = image::decode_png_srgb(&bytes).map_err(|e| format!("{}: {e}", icon.file.display()))?;
        if src.width != 1024 || src.height != 1024 {
            return Err(format!("{}: is {}x{} but should be 1024x1024", icon.file.display(), src.width, src.height));
        }
        Ok(src)
    };
    let icon = any[0];
    let src = read(icon)?;
    let mut variants = Vec::new();
    for i in universal.iter().filter(|i| i.appearance != Some(0)) {
        let appearance = i.appearance.expect("checked above");
        if variants.iter().any(|v: &(u16, String, image::Rgba)| v.0 == appearance) {
            return Err(format!("{}: two app icon images for one appearance", icon_dir.display()));
        }
        variants.push((appearance, i.filename.clone(), read(i)?));
    }
    Ok(Some(IconSource::Single(icon.filename.clone(), src, variants)))
}

fn write_partial_info_plist(o: &Options, mut entries: Vec<(&str, Value)>, outputs: &mut Vec<PathBuf>) -> Result<(), String> {
    let launch = o.launch_entries.borrow();
    if !launch.is_empty() {
        entries.push(("UILaunchImages", Value::Array(launch.clone())));
    }
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
    let mut scan = scan(&catalogs)?;
    // An iOS build leaves out tvOS brand assets and visionOS solid image
    // stacks that --app-icon doesn't name (two probes).
    if platform == "ios" {
        let named = |list: &mut Vec<(String, PathBuf)>| list.retain(|(n, _)| o.app_icon.as_deref() == Some(n.as_str()));
        named(&mut scan.brand_assets);
        named(&mut scan.solid_stacks);
    }
    let document = o.inputs.iter().find(|p| is_document(p) && o.app_icon.as_deref().is_some_and(|n| p.file_stem().is_some_and(|s| s.to_string_lossy() == n)));
    if let Some(p) = o.inputs.iter().find(|p| is_document(p) && Some(*p) != document) {
        return Err(format!("{}: only the app icon's own Icon Composer document is supported by this actool", p.display()));
    }
    // The launch image set --launch-image names is copied into the bundle;
    // the others compile to nothing.
    if let Some(name) = &o.launch_image {
        let (_, dir) = scan.launch_images.iter().find(|(n, _)| n == name).ok_or_else(|| format!("none of the input catalogs contains a launch image set named \"{name}\""))?;
        if platform != "ios" {
            return Err("launch images are supported by this actool for iOS only".into());
        }
        let compiled = launchimage::compile(name, dir, phone, pad)?;
        for (file, source) in &compiled.files {
            let p = out_dir.join(file);
            std::fs::copy(source, &p).map_err(|e| format!("{}: {e}", source.display()))?;
            outputs.push(p);
        }
        if let Some((file, bytes)) = &compiled.made {
            let p = out_dir.join(file);
            std::fs::write(&p, bytes).map_err(|e| format!("{}: {e}", p.display()))?;
            outputs.push(p);
        }
        *o.launch_entries.borrow_mut() = compiled.entries;
    }
    let mut images = imageset::compile(&scan.image_sets, phone && pad, name_identifier)?;
    let (color_facets, colors) = colorset::compile(&scan.color_sets, name_identifier)?;
    images.facets.extend(color_facets);
    images.renditions.extend(colors);
    let (data_facets, data) = dataset::compile(&scan.data_sets, name_identifier)?;
    images.facets.extend(data_facets);
    images.renditions.extend(data);
    if !images.is_empty() && platform != "ios" {
        return Err("image, colour and data sets are supported by this actool for iOS only".into());
    }
    if !images.is_empty() && (!scan.brand_assets.is_empty() || !scan.solid_stacks.is_empty()) {
        return Err("image sets alongside image stacks are not supported by this actool".into());
    }
    if let Some(outputs) = stack_icon(o, &out_dir, platform, &scan, &mut outputs)? {
        return Ok(outputs);
    }
    // Building for iOS without --include-all-app-icons (refused), Xcode
    // ignores the icon sets --app-icon doesn't name, as if they weren't
    // there, and all of them without it (three probes).
    for (name, dir) in &scan.icon_sets {
        if platform != "ios" && o.app_icon.as_deref() != Some(name.as_str()) && !icon_images(dir)?.is_empty() {
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
            if o.target_devices.iter().any(|d| d == "mac") {
                return document_mac_icon(o, &out_dir, platform, name, &icon::render(doc)?, outputs, images);
            }
            Some((name.clone(), IconSource::Single(format!("{name}1024x1024.png"), icon::render(doc)?, Vec::new())))
        }
        (Some(name), None) => {
            let (_, dir) = scan.icon_sets.iter().find(|(n, _)| n == name).ok_or_else(|| format!("none of the input catalogs contains an app icon set named \"{name}\""))?;
            app_icon_set(dir, platform == "ios")?.map(|source| (name.clone(), source))
        }
        (None, _) => None,
    };
    let Some((icon_name, source)) = icon else {
        if !images.is_empty() {
            return finish(o, &out_dir, platform, None, false, Vec::new(), images, Kind::Ios { phone: None, pad: None }, outputs);
        }
        // Nothing to compile: no Assets.car, an empty partial Info.plist.
        write_partial_info_plist(o, Vec::new(), &mut outputs)?;
        return Ok(outputs);
    };
    let (filename, src, variants) = match source {
        IconSource::Single(filename, src, variants) => (filename, src, variants),
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

    // The bundle's flat icons, 60pt@2x for iPhone and 76pt@2x for iPad,
    // both whatever the target devices; each device's entry lists them as
    // for both and names the icon only for a target.
    for (file, size) in [(format!("{icon_name}60x60@2x.png"), 120), (format!("{icon_name}76x76@2x~ipad.png"), 152)] {
        let p = out_dir.join(file);
        std::fs::write(&p, bundle_png(size)?).map_err(|e| e.to_string())?;
        outputs.push(p);
    }
    let phone_icons = iconset::DeviceIcons { files: vec![format!("{icon_name}60x60")], named: phone };
    let pad_icons = iconset::DeviceIcons { files: vec![format!("{icon_name}60x60"), format!("{icon_name}76x76")], named: pad };

    // Assets.car: per idiom, the 1024 px image and the index of its sizes.
    let id = name_identifier(&icon_name);
    // A grey image is stored as GA8, as its dark and tinted ones are (two
    // probes); its bundle PNGs are RGBA as any other's.
    let image_csi = if src.is_gray() {
        let ga: Vec<u8> = src.premultiplied_bgra().chunks_exact(4).flat_map(|p| [p[0], p[3]]).collect();
        car::icon_image_gray_csi(&filename, 1024, 1024, 1, &ga, car::MlecVersions::for_platform(platform))
    } else {
        car::icon_image_csi(&filename, 1024, 1024, 1, &bgra.data, car::MlecVersions::for_platform(platform))
    };
    let multi_csi = car::multisized_csi(&icon_name, &[(1024, 1024, 1)]);
    let rendition = |idiom: u16, part: u16, dim2: u16, csi: &Vec<u8>| car::Rendition {
        attrs: vec![(car::ATTR_SCALE, 1), (car::ATTR_IDIOM, idiom), (car::ATTR_DIMENSION2, dim2), (car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, part)],
        csi: csi.clone(),
    };
    let multi = |idiom| rendition(idiom, car::PART_MULTISIZED_IMAGE, 0, &multi_csi);
    let img = |idiom| rendition(idiom, car::PART_ICON_IMAGE, 1, &image_csi);
    // Dark and tinted images are the 1024 px image's for their appearance,
    // a grey one stored as GA8 (as image sets' are).
    let variant_csis: Vec<(u16, Vec<u8>)> = variants
        .iter()
        .map(|(appearance, file, image)| {
            // Their MLEC versions say whether they are opaque, as other iOS
            // icons' do (a translucent dark image, Chromium's).
            let versions = car::MlecVersions::opacity(image.data.chunks_exact(4).all(|p| p[3] == 255));
            let csi = if image.is_gray() {
                let ga: Vec<u8> = image.premultiplied_bgra().chunks_exact(4).flat_map(|p| [p[0], p[3]]).collect();
                car::icon_image_gray_csi(file, 1024, 1024, 1, &ga, versions)
            } else {
                car::icon_image_csi(file, 1024, 1024, 1, &image.to_bgra().data, versions)
            };
            (*appearance, csi)
        })
        .collect();
    let mut renditions = Vec::new();
    for (wanted, idiom) in [(phone, car::IDIOM_PHONE), (pad, car::IDIOM_PAD)] {
        if wanted {
            renditions.push(img(idiom));
            renditions.push(multi(idiom));
            for (appearance, csi) in &variant_csis {
                let mut r = rendition(idiom, car::PART_ICON_IMAGE, 1, csi);
                r.attrs.push((car::ATTR_APPEARANCE, *appearance));
                renditions.push(r);
            }
        }
    }
    if renditions.is_empty() {
        return Err("--target-device iphone and/or ipad is required".into());
    }
    finish(o, &out_dir, platform, Some(&icon_name), false, renditions, images, Kind::Ios { phone: Some(phone_icons), pad: Some(pad_icons) }, outputs)
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

/// The catalog's table of appearances: Any, and those its renditions use,
/// by name.
fn appearances(renditions: &[car::Rendition]) -> Vec<(String, u16)> {
    let mut values = vec![0];
    for r in renditions {
        if let Some(&(_, v)) = r.attrs.iter().find(|a| a.0 == car::ATTR_APPEARANCE) {
            if !values.contains(&v) {
                values.push(v);
            }
        }
    }
    let name = |v: u16| if v == TINTED { "ISAppearanceTintable".to_string() } else { colorset::APPEARANCES[v as usize].to_string() };
    let mut table: Vec<(String, u16)> = values.into_iter().map(|v| (name(v), v)).collect();
    table.sort();
    table
}

/// What an app icon's catalog and partial Info.plist are for.
enum Kind {
    Ios { phone: Option<iconset::DeviceIcons>, pad: Option<iconset::DeviceIcons> },
    Watch,
    Mac,
}

/// Writes Assets.car and the partial Info.plist of an app icon.
#[allow(clippy::too_many_arguments)]
fn finish(o: &Options, out_dir: &Path, platform: &str, icon_name: Option<&str>, has_dimension1: bool, mut renditions: Vec<car::Rendition>, images: imageset::Compiled, kind: Kind, mut outputs: Vec<PathBuf>) -> Result<Vec<PathBuf>, String> {
    renditions.extend(images.renditions);
    let has_dimension2 = renditions.iter().any(|r| r.attrs.iter().any(|a| a.0 == car::ATTR_DIMENSION2));
    let has_dimension1 = has_dimension1 || renditions.iter().any(|r| r.attrs.iter().any(|a| a.0 == car::ATTR_DIMENSION1 && a.1 != 0));
    // A wide-gamut image's 16-bit rendition brings the gamut attribute, after
    // dimension 1 (iOS; macOS catalogs aren't established).
    let has_gamut = renditions.iter().any(|r| r.attrs.iter().any(|a| a.0 == car::ATTR_GAMUT));
    if has_gamut && matches!(kind, Kind::Mac) {
        return Err("a 16-bit rendition of an image with colours outside sRGB in a macOS catalog is not supported by this actool".into());
    }
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
        if has_gamut {
            k.push(car::ATTR_GAMUT);
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
        // A catalog of data alone has no appearance table.
        appearances: if mac || renditions.iter().all(car::Rendition::is_data) { Vec::new() } else { appearances(&renditions) },
        schema_version: 2,
        key_semantics: if mac { 1 } else { 2 },
        renditions,
        xcode: xcode_identity(),
        deployment_target: o.minimum_deployment_target.clone(),
        platform: platform.into(),
    };
    if !catalog.bitmap_key_order_established() {
        return Err("this catalog's number of identifiers puts its bitmap keys in an order not established for this actool".into());
    }
    let car_path = out_dir.join("Assets.car");
    std::fs::write(&car_path, catalog.to_bytes()).map_err(|e| e.to_string())?;
    outputs.push(car_path);
    outputs.sort();

    let Some(icon_name) = icon_name else {
        write_partial_info_plist(o, Vec::new(), &mut outputs)?;
        return Ok(outputs);
    };
    let primary = |icons: &iconset::DeviceIcons| {
        let mut entry = Vec::new();
        if !icons.files.is_empty() {
            entry.push(("CFBundleIconFiles", Value::Array(icons.files.iter().map(|f| Value::string(f.clone())).collect())));
        }
        if icons.named {
            entry.push(("CFBundleIconName", Value::string(icon_name)));
        }
        Value::dict([("CFBundlePrimaryIcon", Value::dict(entry))])
    };
    let entries = match kind {
        Kind::Ios { phone, pad } => {
            let mut e = Vec::new();
            if let Some(icons) = &phone {
                e.push(("CFBundleIcons", primary(icons)));
            }
            if let Some(icons) = &pad {
                e.push(("CFBundleIcons~ipad", primary(icons)));
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
        Kind::Ios { phone: compiled.phone, pad: compiled.pad }
    };
    finish(o, out_dir, platform, Some(icon_name), compiled.has_dimension1, compiled.renditions, images, kind, outputs)
}

/// An Icon Composer document's macOS icon, relaxed: its flat rendering
/// compiled as a macOS icon set with an image per size, giving Assets.car,
/// the .icns and CFBundleIconFile and CFBundleIconName, as an icon set does.
fn document_mac_icon(
    o: &Options, out_dir: &Path, platform: &str, name: &str, flat: &image::Rgba, outputs: Vec<PathBuf>, images: imageset::Compiled,
) -> Result<Vec<PathBuf>, String> {
    let bgra = flat.to_bgra();
    let dir = std::env::temp_dir().join(format!("actool-icon-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut slots = Vec::new();
    for size in [16, 32, 128, 256, 512] {
        for scale in [1, 2] {
            let filename = if scale == 1 { format!("icon_{size}x{size}.png") } else { format!("icon_{size}x{size}@2x.png") };
            let px = (size * scale) as usize;
            let rgba = image::resize(&bgra, px, px).opaque_rgba().ok_or("a translucent Icon Composer rendering is not supported by this actool for macOS")?;
            let file = dir.join(&filename);
            std::fs::write(&file, image::plain_png(&rgba, false)).map_err(|e| format!("{}: {e}", file.display()))?;
            slots.push(iconset::Slot { idiom: "mac", subtype: 0, size: size as f64, scale, file, filename });
        }
    }
    let result = per_size_icon(o, out_dir, platform, name, &slots, false, false, outputs, images);
    let _ = std::fs::remove_dir_all(&dir);
    result
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
