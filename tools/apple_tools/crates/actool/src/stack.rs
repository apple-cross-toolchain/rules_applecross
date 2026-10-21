//! Image stacks as app icons: tvOS brand assets' layered icon and visionOS's
//! solid image stack (docs/clean-room/actool.md). Each layer's image is
//! stored as `dmp2`, with the stack, its flattened composite and, by
//! platform, a radiosity image (tvOS) or two textures per layer (visionOS).
//!

use crate::car::{self, Rendition, StackLayer};
use crate::image::{self, Rgba};
use crate::{dmp2, name_identifier, read_json};
use std::path::Path;

pub struct Layer {
    /// The layer's facet, "Stack/Layer/Content".
    pub facet: String,
    pub filename: String,
    pub image: Rgba,
    /// Whether its PNG has alpha (a colour type with it, or tRNS).
    pub alpha_channel: bool,
}

/// A stack's layers, front to back as its Contents.json lists them.
pub struct Stack {
    pub name: String,
    pub layers: Vec<Layer>,
}

/// Reads an image stack whose layers (`layer_ext` directories) each hold
/// one image of `idiom` at `scale`.
pub fn read(dir: &Path, layer_ext: &str, idiom: &str, scale: &str) -> Result<Stack, String> {
    let name = dir.file_stem().unwrap().to_string_lossy().into_owned();
    let contents = read_json(&dir.join("Contents.json"))?;
    let listed = contents["layers"].as_array().ok_or_else(|| format!("{}: image stack has no layers list", dir.display()))?;
    let mut layers = Vec::new();
    for l in listed {
        let file = l["filename"].as_str().ok_or_else(|| format!("{}: a layer without a filename", dir.display()))?;
        let (stem, ext) = file.rsplit_once('.').unwrap_or((file, ""));
        if ext != layer_ext {
            return Err(format!("{}: {file} is not a {layer_ext} directory, which this actool doesn't support", dir.display()));
        }
        let set = dir.join(file).join("Content.imageset");
        let images = read_json(&set.join("Contents.json"))?;
        let images: Vec<&serde_json::Value> = images["images"].as_array().map(|a| a.iter().filter(|i| !i["filename"].is_null()).collect()).unwrap_or_default();
        let [image] = images[..] else {
            return Err(format!("{}: layers with other than one image are not supported by this actool", set.display()));
        };
        if image["idiom"].as_str() != Some(idiom) || image["scale"].as_str() != Some(scale) {
            return Err(format!("{}: only a {idiom} image at {scale} is supported by this actool", set.display()));
        }
        let filename = image["filename"].as_str().unwrap().to_string();
        let path = set.join(&filename);
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if bytes.get(25).is_some_and(|t| t & 3 == 0) {
            // Xcode keeps grey layers grey (GA8), which this actool does not write.
            return Err(format!("{}: grey layers are not supported by this actool", path.display()));
        }
        let image = image::decode_png(&bytes)?;
        layers.push(Layer { facet: format!("{name}/{stem}/Content"), filename, image, alpha_channel: image::png_has_alpha(&bytes) });
    }
    if layers.len() < 2 {
        return Err(format!("{}: image stacks of fewer than two layers are not supported by this actool", dir.display()));
    }
    Ok(Stack { name, layers })
}

/// The layers composited back to front, centred on a `width` x `height`
/// canvas: source over, in premultiplied 8-bit, as Xcode rounds it.
fn flatten(stack: &Stack, width: usize, height: usize) -> Result<Vec<u8>, String> {
    let mut out = vec![0u8; width * height * 4];
    for l in stack.layers.iter().rev() {
        let (w, h) = (l.image.width, l.image.height);
        if w > width || h > height || !(width - w).is_multiple_of(2) || !(height - h).is_multiple_of(2) {
            return Err(format!("{}: a {w}x{h} layer on a {width}x{height} stack is not supported by this actool", l.filename));
        }
        let (x0, y0) = ((width - w) / 2, (height - h) / 2);
        let src = l.image.premultiplied_bgra();
        for y in 0..h {
            for x in 0..w {
                let s = &src[(y * w + x) * 4..][..4];
                let o = ((y + y0) * width + x + x0) * 4;
                let a = s[3] as u32;
                for c in 0..4 {
                    out[o + c] = s[c] + ((out[o + c] as u32 * (255 - a) + 127) / 255) as u8;
                }
            }
        }
    }
    Ok(out)
}

/// The smallest image CoreUI keeps unpacked at 1x (iconset.rs).
const PACKING_AREA: usize = 18050;

fn layer_images(stack: &Stack, scale: u32, idiom: u16, renditions: &mut Vec<Rendition>) -> Result<(), String> {
    for (i, l) in stack.layers.iter().enumerate() {
        let (w, h) = (l.image.width, l.image.height);
        if w * h <= PACKING_AREA * scale as usize {
            return Err(format!("{}: layers small enough to pack into an atlas are not supported by this actool", l.filename));
        }
        let bgra = l.image.premultiplied_bgra();
        // Xcode flags the back layer opaque by its pixels, the others by
        // their PNG having no alpha.
        let back = i + 1 == stack.layers.len();
        let opaque = if back { bgra.chunks_exact(4).all(|p| p[3] == 255) } else { !l.alpha_channel };
        let payload = dmp2::encode(w, h, &bgra, opaque).map_err(|e| format!("{}: {e}", l.filename))?;
        renditions.push(Rendition {
            attrs: vec![(car::ATTR_SCALE, scale as u16), (car::ATTR_IDIOM, idiom), (car::ATTR_IDENTIFIER, name_identifier(&l.facet)), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_STACK)],
            csi: car::dmp2_image_csi(&l.filename, w as u32, h as u32, scale, &payload),
        });
    }
    Ok(())
}

fn flattened(stack: &Stack, flat: &[u8], width: usize, height: usize, scale: u32, idiom: u16) -> Rendition {
    let stride = car::stride(width);
    let payload = car::bitmap_payload(&car::padded(flat, width, height), stride, height, 3);
    Rendition {
        attrs: vec![(car::ATTR_SCALE, scale as u16), (car::ATTR_IDIOM, idiom), (car::ATTR_IDENTIFIER, name_identifier(&stack.name)), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_FLATTENED_IMAGE)],
        csi: car::stack_image_csi(&format!("ZZZZFlattenedImage-{scale}.1.0-gamut0"), width as u32, height as u32, scale, &payload),
    }
}

fn stack_rendition(stack: &Stack, width: u32, height: u32, layout: u16, layers: Vec<StackLayer>) -> Rendition {
    Rendition {
        attrs: vec![(car::ATTR_SCALE, 1), (car::ATTR_IDENTIFIER, name_identifier(&stack.name)), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_STACK)],
        csi: car::stack_csi(width, height, layout, &layers),
    }
}

/// tvOS's radiosity image (docs/clean-room/actool.md): on a canvas 80 px
/// larger than the stack, the composite's lower two thirds 40 px in from
/// the left and the top, blurred (a Gaussian of 20 px to 40 px each side,
/// in 14-bit weights rounded down, across then down, rounding to 8 bits
/// after each), then per 32 px cell its 8 px blocks' sums weighed
/// -1331, 1843, 1843, -1331 (in 1/1024) each way: 16-bit
/// signed BGRA, 64 to a level.
fn radiosity(flat: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    const MARGIN: usize = 40;
    const CELL: usize = 32;
    const R: isize = 40;
    let (cw, ch) = (width + 2 * MARGIN, height + 2 * MARGIN);
    let top = height / 3;
    let gauss: Vec<f64> = (-R..=R).map(|d| (-(d * d) as f64 / 800.0).exp()).collect();
    let total: f64 = gauss.iter().sum();
    let weights: Vec<i64> = gauss.iter().map(|g| (16384.0 * g / total).floor() as i64).collect();
    let blur = |src: &[u8], across: bool| -> Vec<u8> {
        let mut out = vec![0u8; cw * ch];
        for y in 0..ch as isize {
            for x in 0..cw as isize {
                let mut acc = 0i64;
                for d in -R..=R {
                    let (xx, yy) = if across { (x + d, y) } else { (x, y + d) };
                    if xx >= 0 && yy >= 0 && (xx as usize) < cw && (yy as usize) < ch {
                        acc += weights[(d + R) as usize] * src[yy as usize * cw + xx as usize] as i64;
                    }
                }
                out[y as usize * cw + x as usize] = ((acc + 8192) >> 14).min(255) as u8;
            }
        }
        out
    };
    const W: [i64; 4] = [-1331, 1843, 1843, -1331];
    // Levels of flat cells Xcode stores as 64 * level - 1 (measured).
    const FLAT_ONE_LESS: [i64; 36] = [3, 6, 12, 24, 37, 47, 48, 71, 74, 77, 93, 94, 95, 96, 103, 105, 129, 142, 143, 147, 148, 149, 154, 155, 185, 186, 187, 188, 189, 190, 192, 193, 206, 209, 210, 249];
    let mut cells = vec![[0i16; 4]; (cw / CELL) * (ch / CELL)];
    for c in 0..4 {
        let mut canvas = vec![0u8; cw * ch];
        for y in top..height {
            for x in 0..width {
                canvas[(y - top + MARGIN) * cw + x + MARGIN] = flat[(y * width + x) * 4 + c];
            }
        }
        let blurred = blur(&blur(&canvas, true), false);
        for gy in 0..ch / CELL {
            for gx in 0..cw / CELL {
                let mut sum = 0i64;
                let mut blocks = [0i64; 16];
                for by in 0..4 {
                    for bx in 0..4 {
                        let mut block = 0i64;
                        for y in 0..8 {
                            for x in 0..8 {
                                block += blurred[(gy * CELL + 8 * by + y) * cw + gx * CELL + 8 * bx + x] as i64;
                            }
                        }
                        blocks[4 * by + bx] = block;
                        sum += W[by] * W[bx] * block;
                    }
                }
                // Divided, truncating towards zero. Xcode's arithmetic is
                // not established: flat cells of some levels come out one
                // less (measured), and about one other value in 1,700, within
                // single-precision rounding of a whole number, can differ by
                // one.
                let mut v = sum / (1 << 20);
                if blocks.iter().all(|b| *b == blocks[0]) && blocks[0] % 64 == 0 && FLAT_ONE_LESS.contains(&(blocks[0] / 64)) {
                    v -= 1;
                }
                cells[gy * (cw / CELL) + gx][c] = v as i16;
            }
        }
    }
    let mut body = Vec::new();
    body.extend((cw as u16).to_le_bytes());
    body.extend((ch as u16).to_le_bytes());
    body.extend((CELL as u32).to_le_bytes());
    body.extend(0.725_551_07f32.to_le_bytes());
    for cell in &cells {
        for v in cell {
            body.extend(v.to_le_bytes());
        }
    }
    let mut out = b"MLEC".to_vec();
    out.extend(0u32.to_le_bytes());
    out.extend(6u32.to_le_bytes());
    out.extend((body.len() as u32).to_le_bytes());
    out.extend(body);
    Ok(out)
}

/// A tvOS layered app icon (brand assets' primary app icon), `width` x
/// `height` points at 1x.
pub fn tv(stack: &Stack, width: usize, height: usize) -> Result<Vec<Rendition>, String> {
    if (width, height) != (400, 240) {
        return Err(format!("{}: tvOS app icons other than 400x240 are not supported by this actool", stack.name));
    }
    let id = name_identifier(&stack.name);
    let mut renditions = Vec::new();
    let mut layers = Vec::new();
    for l in stack.layers.iter().rev() {
        let (w, h) = (l.image.width as u32, l.image.height as u32);
        layers.push(StackLayer { x: (width as u32).saturating_sub(w) / 2, y: (height as u32).saturating_sub(h) / 2, width: w, height: h, identifier: name_identifier(&l.facet) });
    }
    renditions.push(stack_rendition(stack, width as u32, height as u32, car::LAYOUT_LAYER_STACK, layers));
    layer_images(stack, 1, car::IDIOM_TV, &mut renditions)?;
    let flat = flatten(stack, width, height)?;
    renditions.push(flattened(stack, &flat, width, height, 1, car::IDIOM_TV));
    renditions.push(Rendition {
        attrs: vec![(car::ATTR_SCALE, 1), (car::ATTR_IDIOM, car::IDIOM_TV), (car::ATTR_IDENTIFIER, id), (car::ATTR_ELEMENT, car::ELEMENT_ICON), (car::ATTR_PART, car::PART_RADIOSITY_IMAGE)],
        csi: car::stack_image_csi("ZZZZRadiosityImage-1.0.0", width as u32 + 80, height as u32 + 80, 1, &radiosity(&flat, width, height)?),
    });
    Ok(renditions)
}

/// visionOS's distance texture of an opaque layer: each pixel centre's
/// distance to the image's edge.
fn distances(width: usize, height: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let d = (x as f32 + 0.5).min(y as f32 + 0.5).min(width as f32 - 0.5 - x as f32).min(height as f32 - 0.5 - y as f32);
            out.extend(d.to_le_bytes());
        }
    }
    out
}

/// A visionOS solid image stack, its layers 1024 px square at 2x.
pub fn vision(stack: &Stack) -> Result<Vec<Rendition>, String> {
    const SIZE: usize = crate::directions::SIZE;
    for l in &stack.layers {
        if (l.image.width, l.image.height) != (SIZE, SIZE) {
            return Err(format!("{}: visionOS app icon layers other than 1024x1024 are not supported by this actool", l.filename));
        }
        if l.image.data.chunks_exact(4).any(|p| p[3] != 255) {
            return Err(format!("{}: translucent visionOS app icon layers are not supported by this actool", l.filename));
        }
    }
    let mut renditions = Vec::new();
    let layers = stack.layers.iter().rev().map(|l| StackLayer { x: 0, y: 0, width: SIZE as u32 / 2, height: SIZE as u32 / 2, identifier: name_identifier(&l.facet) }).collect();
    renditions.push(stack_rendition(stack, SIZE as u32 / 2, SIZE as u32 / 2, car::LAYOUT_SOLID_LAYER_STACK, layers));
    layer_images(stack, 2, car::IDIOM_VISION, &mut renditions)?;
    renditions.push(flattened(stack, &flatten(stack, SIZE, SIZE)?, SIZE, SIZE, 2, car::IDIOM_VISION));
    let textures = [(1u16, car::TEXTURE_R32_FLOAT, 4u32, distances(SIZE, SIZE)), (2, car::TEXTURE_RG8_SNORM, 2, crate::directions::opaque())];
    for l in &stack.layers {
        let lid = name_identifier(&l.facet);
        for (level, format, bytes, data) in &textures {
            let key = |part: Option<u16>| {
                let mut k = vec![(car::ATTR_SCALE, 2), (car::ATTR_IDIOM, car::IDIOM_VISION), (car::ATTR_DIMENSION1, *level), (car::ATTR_IDENTIFIER, lid), (car::ATTR_ELEMENT, car::ELEMENT_TEXTURE)];
                k.extend(part.map(|p| (car::ATTR_PART, p)));
                k
            };
            let image_key = [(car::ATTR_ELEMENT, car::ELEMENT_TEXTURE), (car::ATTR_PART, car::PART_STACK), (car::ATTR_DIMENSION1, *level), (car::ATTR_SCALE, 2), (car::ATTR_IDENTIFIER, lid), (car::ATTR_IDIOM, car::IDIOM_VISION)];
            renditions.push(Rendition { attrs: key(None), csi: car::texture_csi(&l.facet, SIZE as u32, SIZE as u32, *format, &image_key) });
            renditions.push(Rendition { attrs: key(Some(car::PART_STACK)), csi: car::texture_image_csi(&l.filename, SIZE as u32, SIZE as u32, 2, SIZE as u32 * bytes, data) });
        }
    }
    Ok(renditions)
}
