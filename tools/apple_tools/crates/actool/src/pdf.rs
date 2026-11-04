//! A vector set's PDF as Xcode stores it: not the file, but its page drawn
//! again into a new PDF by CoreGraphics (docs/clean-room/actool.md, "Exact
//! PDFs"). This replays a page of paths, fills, strokes, clips, colours,
//! line state, transforms and constant alpha, and writes what CoreGraphics
//! writes for them; anything else is an error, and the file is stored as it
//! is (relaxed).
//!
//! What CoreGraphics writes, from probes of Xcode 27's actool:
//! - PDF 1.3: the content stream (object 4), the page (1), its resources
//!   (3), the alpha states, each colour space's ICC profile and
//!   `[ /ICCBased … ]` array, the pages (2, with the media box), the catalog
//!   and an empty Info; a zero ID. Streams are zlib level 1.
//! - The content is `q Q`, then the drawing in `q … Q`. Its tokens are
//!   separated by a space, or a newline once a line has 73 characters; a
//!   rectangle's four operands are one token, as are a matrix's six.
//! - Numbers are written as `number` says: 7 significant digits, or fewer
//!   for a value very near a shorter decimal.
//! - State is written only where it changes, before each painting: the line
//!   width, cap, join, miter limit and dash for strokes, then the fill
//!   colour and its alpha, then the stroke colour and its alpha. A colour is
//!   written again whenever its alpha changes; each alpha is an ExtGState
//!   of its own (`ca` or `CA`), shared by value. DeviceRGB and DeviceGray
//!   become ICC spaces (sRGB, Generic Gray Gamma 2.2), DeviceCMYK stays.
//! - A fill's path is transformed by the CTM; a stroke's is kept in user
//!   space inside `q <ctm> cm … Q` unless the CTM is the identity. A path
//!   that is one rectangle stays `re` where the CTM keeps it an upright
//!   rectangle; otherwise rectangles are written as their four lines.
//!   `v` and `y` become `c`; `F` is `f`; `s`, `b` and `b*` close the path.
//! - A clip is written before the painting it applies to: its paths in
//!   order, then the intersection of its rectangles. A clip restored away,
//!   or added other than after the written ones before anything is painted,
//!   takes `Q q` and every state again.
//! - A stroke's lone rectangle stays `re` (in user space) only where its
//!   CTM keeps it upright.

use std::collections::HashMap;

use hayro::hayro_syntax::Pdf;

type Matrix = [f64; 6];
const IDENTITY: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

fn fail<T>(msg: impl Into<String>) -> Result<T, String> {
    Err(msg.into())
}

/// A number as CoreGraphics writes it (961 probes): 0 under 1e-6; within
/// 1e-6 of an integer, that integer; from 1 up, 7 significant digits without
/// trailing zeros; under 1, with the fewest decimals (from its first
/// significant digit's, up to 8) at which it is within 1e-5 of a whole
/// number of their unit, keeping zeros (0.0999999999 is 0.10), else 7
/// significant digits, zeros kept (0.6939910). A value too near one of
/// those thresholds, or a tie in rounding, isn't established: an error.
pub fn number(x: f64) -> Result<String, String> {
    let unsure = || Err(format!("the number {x:e} in a PDF rounds where CoreGraphics' choice isn't established"));
    let a = x.abs();
    let near = |v: f64, t: f64| (v - t).abs() <= t * 0.01;
    if near(a, 1e-6) || near((a - a.round()).abs(), 1e-6) {
        return unsure();
    }
    if a < 1e-6 {
        return Ok("0".into());
    }
    let fixed = |decimals: usize| -> Result<String, String> {
        // A tie at the last digit rounds as the C library does, which isn't
        // established.
        let exact = format!("{a:.60}");
        let point = exact.find('.').expect("a point");
        let rest = &exact[point + 1 + decimals..];
        if rest.starts_with('5') && rest[1..].bytes().all(|c| c == b'0') {
            return unsure();
        }
        Ok(format!("{a:.decimals$}"))
    };
    let s = if (a - a.round()).abs() < 1e-6 {
        format!("{}", a.round())
    } else {
        let magnitude = a.log10().floor() as i32;
        let decimals = (6 - magnitude).max(0) as usize;
        if a >= 1.0 {
            let s = fixed(decimals)?;
            if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s }
        } else {
            let mut chosen = None;
            for k in (-magnitude).max(1) as usize..=decimals.min(9) {
                let y = a * 10f64.powi(k as i32);
                let off = (y - y.round()).abs();
                if near(off, 1e-5) {
                    return unsure();
                }
                if off <= 1e-5 {
                    // Whether a ninth decimal is tried isn't established.
                    if k == 9 {
                        return unsure();
                    }
                    chosen = Some(k);
                    break;
                }
            }
            fixed(chosen.unwrap_or(decimals))?
        }
    };
    Ok(if x < 0.0 && s != "0" { format!("-{s}") } else { s })
}

fn multiply(a: &Matrix, b: &Matrix) -> Matrix {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

fn apply(m: &Matrix, (x, y): (f64, f64)) -> (f64, f64) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

fn invert(m: &Matrix) -> Matrix {
    let det = m[0] * m[3] - m[1] * m[2];
    [m[3] / det, -m[1] / det, -m[2] / det, m[0] / det, (m[2] * m[5] - m[3] * m[4]) / det, (m[1] * m[4] - m[0] * m[5]) / det]
}

/// A stroke's path back in user space, as CoreGraphics keeps paths: each
/// point through the CTM and its inverse (0.1 under a translation of 11.6
/// is 0.0999999999999996; a probe).
fn round_trip(path: &[Segment], m: &Matrix) -> Vec<Segment> {
    let inverse = invert(m);
    let back = |p: (f64, f64)| apply(&inverse, apply(m, p));
    path.iter()
        .map(|s| match *s {
            Segment::Move(x, y) => {
                let (x, y) = back((x, y));
                Segment::Move(x, y)
            }
            Segment::Line(x, y) => {
                let (x, y) = back((x, y));
                Segment::Line(x, y)
            }
            Segment::Curve(c) => {
                let (a, b, e) = (back((c[0], c[1])), back((c[2], c[3])), back((c[4], c[5])));
                Segment::Curve([a.0, a.1, b.0, b.1, e.0, e.1])
            }
            ref other => other.clone(),
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Colour {
    Gray(f64),
    Rgb([f64; 3]),
    Cmyk([f64; 4]),
}

#[derive(Clone, PartialEq, Debug)]
enum Segment {
    Move(f64, f64),
    Line(f64, f64),
    Curve([f64; 6]),
    Close,
    Rect(f64, f64, f64, f64),
}

#[derive(Clone, PartialEq, Debug)]
struct Clip {
    path: Vec<Segment>,
    even_odd: bool,
}

#[derive(Clone)]
struct State {
    ctm: Matrix,
    fill: (Colour, f64),
    stroke: (Colour, f64),
    width: f64,
    cap: i64,
    join: i64,
    miter: f64,
    dash: (Vec<f64>, f64),
    /// The clipping paths, in order.
    clip: Vec<Clip>,
    /// The rectangles clipped to, as their intersection.
    clip_rect: Option<[f64; 4]>,
}

impl State {
    /// The clip as written: its paths, then its rectangle.
    fn clips(&self) -> Vec<Clip> {
        let mut clips = self.clip.clone();
        if let Some([x, y, w, h]) = self.clip_rect {
            clips.push(Clip { path: vec![Segment::Rect(x, y, w, h)], even_odd: false });
        }
        clips
    }
}

impl State {
    fn initial() -> State {
        State {
            ctm: IDENTITY,
            fill: (Colour::Gray(0.0), 1.0),
            stroke: (Colour::Gray(0.0), 1.0),
            width: 1.0,
            cap: 0,
            join: 0,
            miter: 10.0,
            dash: (Vec::new(), 0.0),
            clip: Vec::new(),
            clip_rect: None,
        }
    }
}

enum Token {
    Number(f64),
    Name(String),
    Array(Vec<f64>),
    Operator(String),
}

fn tokens(data: &[u8]) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let mut i = 0;
    let delimiter = |c: u8| b"()<>[]{}/%".contains(&c);
    let space = |c: u8| b"\0\t\n\x0c\r ".contains(&c);
    let word = |i: &mut usize| {
        let start = *i;
        while *i < data.len() && !space(data[*i]) && !delimiter(data[*i]) {
            *i += 1;
        }
        &data[start..*i]
    };
    let parse_number = |w: &[u8]| -> Option<f64> {
        let s = std::str::from_utf8(w).ok()?;
        if s.is_empty() || !s.bytes().all(|c| c.is_ascii_digit() || b"+-.".contains(&c)) {
            return None;
        }
        s.parse::<f64>().ok().or_else(|| if s.ends_with('.') { s.trim_end_matches('.').parse().ok() } else { None })
    };
    while i < data.len() {
        let c = data[i];
        if space(c) {
            i += 1;
        } else if c == b'%' {
            while i < data.len() && data[i] != b'\n' && data[i] != b'\r' {
                i += 1;
            }
        } else if c == b'/' {
            i += 1;
            let w = word(&mut i);
            if w.contains(&b'#') {
                return fail("a PDF name with an escape is not supported by this actool");
            }
            out.push(Token::Name(String::from_utf8_lossy(w).into_owned()));
        } else if c == b'[' {
            i += 1;
            let mut values = Vec::new();
            loop {
                while i < data.len() && space(data[i]) {
                    i += 1;
                }
                if i >= data.len() {
                    return fail("an unterminated array in a PDF's content");
                }
                if data[i] == b']' {
                    i += 1;
                    break;
                }
                let w = word(&mut i);
                values.push(parse_number(w).ok_or("an array of other than numbers in a PDF's content is not supported by this actool")?);
            }
            out.push(Token::Array(values));
        } else if delimiter(c) {
            return fail(format!("'{}' in a PDF's content is not supported by this actool", c as char));
        } else {
            let w = word(&mut i);
            match parse_number(w) {
                Some(n) => out.push(Token::Number(n)),
                None => out.push(Token::Operator(String::from_utf8_lossy(w).into_owned())),
            }
        }
    }
    Ok(out)
}

/// The colour spaces and alpha states the content names, in order of use.
#[derive(Default)]
struct Resources {
    spaces: Vec<bool>,
    alphas: Vec<(&'static str, f64)>,
}

impl Resources {
    /// `/CsN`: `gray` for Generic Gray Gamma 2.2, else sRGB.
    fn space(&mut self, gray: bool) -> String {
        let n = match self.spaces.iter().position(|&g| g == gray) {
            Some(n) => n,
            None => {
                self.spaces.push(gray);
                self.spaces.len() - 1
            }
        };
        format!("/Cs{}", n + 1)
    }

    fn alpha(&mut self, key: &'static str, value: f64) -> String {
        let n = match self.alphas.iter().position(|a| a.0 == key && a.1 == value) {
            Some(n) => n,
            None => {
                self.alphas.push((key, value));
                self.alphas.len() - 1
            }
        };
        format!("/Gs{}", n + 1)
    }
}

/// The content CoreGraphics writes, as tokens.
struct Writer {
    out: Vec<String>,
    /// What the written content has set, inside its `q`.
    written: State,
    open: bool,
    /// Whether anything is painted since the last `q`.
    painted: bool,
    /// The first number that can't be written as CoreGraphics would.
    error: Option<String>,
    resources: Resources,
}

impl Writer {
    fn push(&mut self, s: impl Into<String>) {
        self.out.push(s.into());
    }

    /// A rectangle's operands are written as one, never across lines.
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        let operands = [x, y, w, h].map(|v| self.num(v)).join(" ");
        self.push(operands);
        self.push("re");
    }

    /// A number, or an empty token after noting the first that can't be
    /// written as CoreGraphics would.
    fn num(&mut self, x: f64) -> String {
        number(x).unwrap_or_else(|e| {
            self.error.get_or_insert(e);
            String::new()
        })
    }

    fn numbers(&mut self, v: &[f64]) {
        for &x in v {
            let n = self.num(x);
            self.push(n);
        }
    }

    fn colour(&mut self, (colour, alpha): (Colour, f64), stroke: bool) {
        let (cs, sc, k, a) = if stroke { ("CS", "SC", "K", "CA") } else { ("cs", "sc", "k", "ca") };
        match colour {
            Colour::Gray(g) => {
                let name = self.resources.space(true);
                self.push(name);
                self.push(cs);
                self.numbers(&[g]);
                self.push(sc);
            }
            Colour::Rgb(c) => {
                let name = self.resources.space(false);
                self.push(name);
                self.push(cs);
                self.numbers(&c);
                self.push(sc);
            }
            Colour::Cmyk(c) => {
                self.numbers(&c);
                self.push(k);
            }
        }
        let written = if stroke { self.written.stroke.1 } else { self.written.fill.1 };
        if alpha != written {
            let name = self.resources.alpha(a, alpha);
            self.push(name);
            self.push("gs");
        }
        if stroke {
            self.written.stroke = (colour, alpha);
        } else {
            self.written.fill = (colour, alpha);
        }
    }

    /// `path` under `m`; a lone rectangle stays one where `drawn` (the CTM
    /// it's drawn with) keeps it upright.
    fn path(&mut self, path: &[Segment], m: &Matrix, drawn: &Matrix) {
        if let [Segment::Rect(x, y, w, h)] = path {
            if upright(drawn) && *w >= 0.0 && *h >= 0.0 {
                let (ax, ay) = apply(m, (*x, *y));
                let (bx, by) = apply(m, (x + w, y + h));
                self.rect(ax.min(bx), ay.min(by), (bx - ax).abs(), (by - ay).abs());
                return;
            }
        }
        for s in path {
            match *s {
                Segment::Move(x, y) => {
                    let (x, y) = apply(m, (x, y));
                    self.numbers(&[x, y]);
                    self.push("m");
                }
                Segment::Line(x, y) => {
                    let (x, y) = apply(m, (x, y));
                    self.numbers(&[x, y]);
                    self.push("l");
                }
                Segment::Curve(c) => {
                    let a = apply(m, (c[0], c[1]));
                    let b = apply(m, (c[2], c[3]));
                    let e = apply(m, (c[4], c[5]));
                    self.numbers(&[a.0, a.1, b.0, b.1, e.0, e.1]);
                    self.push("c");
                }
                Segment::Close => self.push("h"),
                Segment::Rect(x, y, w, h) => {
                    for (px, py, op) in [(x, y, "m"), (x + w, y, "l"), (x + w, y + h, "l"), (x, y + h, "l")] {
                        let (px, py) = apply(m, (px, py));
                        self.numbers(&[px, py]);
                        self.push(op);
                    }
                    self.push("h");
                }
            }
        }
    }

    /// The clip `state` has, written: added to the written one before
    /// anything is painted under it, or else after `Q q` and every state
    /// again.
    fn clip(&mut self, clip: &[Clip]) {
        if self.written.clip.as_slice() == clip {
            return;
        }
        // (`written.clip` holds the written clips, rectangle included.)
        let from = if clip.starts_with(&self.written.clip) && !self.painted {
            self.written.clip.len()
        } else {
            self.push("Q");
            self.push("q");
            self.written = State::initial();
            self.painted = false;
            0
        };
        for c in &clip[from..] {
            self.path(&c.path, &IDENTITY, &IDENTITY);
            self.push(if c.even_odd { "W*" } else { "W" });
            self.push("n");
        }
        self.written.clip = clip.to_vec();
    }

    fn paint(&mut self, state: &State, path: &[Segment], op: &str) {
        if path.is_empty() {
            return;
        }
        if !self.open {
            self.push("q");
            self.open = true;
        }
        self.clip(&state.clips());
        let fills = matches!(op, "f" | "f*" | "B" | "B*");
        let strokes = matches!(op, "S" | "B" | "B*");
        if strokes {
            if state.width != self.written.width {
                self.numbers(&[state.width]);
                self.push("w");
            }
            if state.cap != self.written.cap {
                self.push(state.cap.to_string());
                self.push("J");
            }
            if state.join != self.written.join {
                self.push(state.join.to_string());
                self.push("j");
            }
            if state.miter != self.written.miter {
                self.numbers(&[state.miter]);
                self.push("M");
            }
            if state.dash != self.written.dash {
                self.push("[");
                let dash = state.dash.0.clone();
                self.numbers(&dash);
                self.push("]");
                self.numbers(&[state.dash.1]);
                self.push("d");
            }
            self.written.width = state.width;
            self.written.cap = state.cap;
            self.written.join = state.join;
            self.written.miter = state.miter;
            self.written.dash = state.dash.clone();
        }
        if fills && state.fill != self.written.fill {
            self.colour(state.fill, false);
        }
        if strokes && state.stroke != self.written.stroke {
            self.colour(state.stroke, true);
        }
        if strokes && state.ctm != IDENTITY {
            self.push("q");
            let matrix = state.ctm.map(|v| self.num(v)).join(" ");
            self.push(matrix);
            self.push("cm");
            self.path(&round_trip(path, &state.ctm), &IDENTITY, &state.ctm);
            self.push(op);
            self.push("Q");
        } else {
            self.path(path, &state.ctm, &state.ctm);
            self.push(op);
        }
        self.painted = true;
    }
}

/// Whether `m` keeps a rectangle an upright one, the same way round.
fn upright(m: &Matrix) -> bool {
    ((m[1] == 0.0 && m[2] == 0.0) || (m[0] == 0.0 && m[3] == 0.0)) && m[0] * m[3] - m[1] * m[2] > 0.0
}

/// `path` transformed, for a clip: its one rectangle where it stays one.
fn transformed(path: &[Segment], m: &Matrix) -> Vec<Segment> {
    if let [Segment::Rect(x, y, w, h)] = path {
        if upright(m) && *w >= 0.0 && *h >= 0.0 {
            let (ax, ay) = apply(m, (*x, *y));
            let (bx, by) = apply(m, (x + w, y + h));
            return vec![Segment::Rect(ax.min(bx), ay.min(by), (bx - ax).abs(), (by - ay).abs())];
        }
    }
    let mut out = Vec::new();
    for s in path {
        match *s {
            Segment::Move(x, y) => {
                let (x, y) = apply(m, (x, y));
                out.push(Segment::Move(x, y));
            }
            Segment::Line(x, y) => {
                let (x, y) = apply(m, (x, y));
                out.push(Segment::Line(x, y));
            }
            Segment::Curve(c) => {
                let a = apply(m, (c[0], c[1]));
                let b = apply(m, (c[2], c[3]));
                let e = apply(m, (c[4], c[5]));
                out.push(Segment::Curve([a.0, a.1, b.0, b.1, e.0, e.1]));
            }
            Segment::Close => out.push(Segment::Close),
            Segment::Rect(x, y, w, h) => {
                for (i, (px, py)) in [(x, y), (x + w, y), (x + w, y + h), (x, y + h)].into_iter().enumerate() {
                    let (px, py) = apply(m, (px, py));
                    out.push(if i == 0 { Segment::Move(px, py) } else { Segment::Line(px, py) });
                }
                out.push(Segment::Close);
            }
        }
    }
    out
}

/// The ICC profile of a colour space, from the SDKs' toolchain
/// (tools/package-sdks.sh copies them from the Mac that packaged it).
fn profile(gray: bool) -> Result<Vec<u8>, String> {
    let developer = std::env::var("DEVELOPER_DIR").unwrap_or_else(|_| "/Applications/Xcode.app/Contents/Developer".into());
    let name = if gray { "generic_gray_gamma_2_2.icc" } else { "srgb.icc" };
    let path = std::path::Path::new(&developer).join("Toolchains/XcodeDefault.xctoolchain/usr/share/colorsync").join(name);
    std::fs::read(&path).map_err(|e| format!("{}: {e} (an SDK archive packaged before it kept the Mac's colour profiles)", path.display()))
}

/// The PDF Xcode stores for a vector set's `data`.
pub fn rewrite(data: &[u8]) -> Result<Vec<u8>, String> {
    let (content, resources, size) = draw(data)?;
    assemble(&content, &resources, size, &profile)
}

/// The page of `data` drawn again: the content stream, the resources it
/// names and the page's size.
fn draw(data: &[u8]) -> Result<(String, Resources, (f64, f64)), String> {
    let pdf = Pdf::new(data.to_vec()).map_err(|e| format!("not a PDF Xcode could read ({e:?})"))?;
    let pages = pdf.pages();
    let [page] = pages.as_ref() else {
        return fail("a PDF of other than one page");
    };
    let media = page.media_box();
    let crop = page.crop_box();
    if media.x0 != 0.0 || media.y0 != 0.0 || (crop.x0, crop.y0, crop.x1, crop.y1) != (media.x0, media.y0, media.x1, media.y1) {
        return fail("a PDF page whose media box isn't at the origin, or with a crop box of its own, is not supported by this actool");
    }
    if page.raw().contains_key(b"Rotate") || page.raw().contains_key(b"UserUnit") {
        return fail("a rotated or scaled PDF page is not supported by this actool");
    }
    let content = page.page_stream().ok_or("a PDF page without content")?;
    let resources = page.resources();

    let mut w = Writer { out: vec!["q".into(), "Q".into()], written: State::initial(), open: false, painted: false, error: None, resources: Resources::default() };
    let mut state = State::initial();
    let mut stack: Vec<State> = Vec::new();
    let mut path: Vec<Segment> = Vec::new();
    let mut current = (0.0, 0.0);
    let mut start = (0.0, 0.0);
    let mut pending_clip: Option<bool> = None;
    let mut operands: Vec<Token> = Vec::new();

    for t in tokens(content)? {
        let Token::Operator(op) = t else {
            operands.push(t);
            continue;
        };
        let nums: Vec<f64> = operands.iter().filter_map(|t| if let Token::Number(n) = t { Some(*n) } else { None }).collect();
        let want = |n: usize| -> Result<(), String> {
            if nums.len() != n || operands.len() != n {
                return fail(format!("PDF operator {op} with these operands is not supported by this actool"));
            }
            Ok(())
        };
        match op.as_str() {
            "q" => {
                want(0)?;
                stack.push(state.clone());
            }
            "Q" => {
                want(0)?;
                state = stack.pop().ok_or("a PDF's content restores more than it saves")?;
            }
            "cm" => {
                want(6)?;
                let m: Matrix = nums[..6].try_into().expect("six");
                state.ctm = multiply(&m, &state.ctm);
            }
            "w" => {
                want(1)?;
                state.width = nums[0];
            }
            "J" | "j" => {
                want(1)?;
                let v = nums[0];
                if v.fract() != 0.0 || !(0.0..=2.0).contains(&v) {
                    return fail(format!("{op} {v} in a PDF is not supported by this actool"));
                }
                if op == "J" { state.cap = v as i64 } else { state.join = v as i64 }
            }
            "M" => {
                want(1)?;
                state.miter = nums[0];
            }
            "d" => {
                let [Token::Array(a), Token::Number(phase)] = operands.as_slice() else {
                    return fail("a dash other than an array and a phase is not supported by this actool");
                };
                state.dash = (a.clone(), *phase);
            }
            "g" | "G" => {
                want(1)?;
                set_colour(&mut state, op == "G", Colour::Gray(nums[0]));
            }
            "rg" | "RG" => {
                want(3)?;
                set_colour(&mut state, op == "RG", Colour::Rgb([nums[0], nums[1], nums[2]]));
            }
            "k" | "K" => {
                want(4)?;
                set_colour(&mut state, op == "K", Colour::Cmyk([nums[0], nums[1], nums[2], nums[3]]));
            }
            "cs" | "CS" => {
                let [Token::Name(name)] = operands.as_slice() else {
                    return fail(format!("{op} with these operands is not supported by this actool"));
                };
                let colour = match name.as_str() {
                    "DeviceGray" => Colour::Gray(0.0),
                    "DeviceRGB" => Colour::Rgb([0.0; 3]),
                    "DeviceCMYK" => Colour::Cmyk([0.0, 0.0, 0.0, 1.0]),
                    other => return fail(format!("colour space {other} in a PDF is not supported by this actool")),
                };
                set_colour(&mut state, op == "CS", colour);
            }
            "sc" | "SC" | "scn" | "SCN" => {
                let stroke = op.starts_with('S');
                let current = if stroke { state.stroke.0 } else { state.fill.0 };
                let colour = match (current, nums.len()) {
                    (Colour::Gray(_), 1) => Colour::Gray(nums[0]),
                    (Colour::Rgb(_), 3) => Colour::Rgb([nums[0], nums[1], nums[2]]),
                    (Colour::Cmyk(_), 4) => Colour::Cmyk([nums[0], nums[1], nums[2], nums[3]]),
                    _ => return fail(format!("{op} with these operands is not supported by this actool")),
                };
                if operands.len() != nums.len() {
                    return fail(format!("{op} with a pattern is not supported by this actool"));
                }
                set_colour(&mut state, stroke, colour);
            }
            "gs" => {
                let [Token::Name(name)] = operands.as_slice() else {
                    return fail("gs with these operands is not supported by this actool");
                };
                let n = hayro::hayro_syntax::object::Name::new_unescaped(name.as_bytes());
                let dict = resources.get_ext_g_state(&n).ok_or_else(|| format!("ExtGState {name} isn't in the PDF's resources"))?;
                for key in dict.keys() {
                    match key.as_str() {
                        "Type" => {}
                        "ca" => state.fill.1 = dict.get::<f64>(b"ca").ok_or("an ExtGState's ca isn't a number")?,
                        "CA" => state.stroke.1 = dict.get::<f64>(b"CA").ok_or("an ExtGState's CA isn't a number")?,
                        other => return fail(format!("ExtGState key {other} in a PDF is not supported by this actool")),
                    }
                }
            }
            "m" => {
                want(2)?;
                current = (nums[0], nums[1]);
                start = current;
                path.push(Segment::Move(nums[0], nums[1]));
            }
            "l" => {
                want(2)?;
                if path.is_empty() {
                    return fail("a PDF path that doesn't begin with m or re is not supported by this actool");
                }
                current = (nums[0], nums[1]);
                path.push(Segment::Line(nums[0], nums[1]));
            }
            "c" | "v" | "y" => {
                want(if op == "c" { 6 } else { 4 })?;
                if path.is_empty() {
                    return fail("a PDF path that doesn't begin with m or re is not supported by this actool");
                }
                let c = match op.as_str() {
                    "c" => [nums[0], nums[1], nums[2], nums[3], nums[4], nums[5]],
                    "v" => [current.0, current.1, nums[0], nums[1], nums[2], nums[3]],
                    _ => [nums[0], nums[1], nums[2], nums[3], nums[2], nums[3]],
                };
                current = (c[4], c[5]);
                path.push(Segment::Curve(c));
            }
            "h" => {
                want(0)?;
                if !matches!(path.last(), None | Some(Segment::Close | Segment::Rect(..))) {
                    path.push(Segment::Close);
                }
                current = start;
            }
            "re" => {
                want(4)?;
                path.push(Segment::Rect(nums[0], nums[1], nums[2], nums[3]));
                current = (nums[0], nums[1]);
                start = current;
            }
            "W" | "W*" => {
                want(0)?;
                pending_clip = Some(op == "W*");
            }
            "f" | "F" | "f*" | "S" | "s" | "B" | "B*" | "b" | "b*" | "n" => {
                want(0)?;
                let closes = matches!(op.as_str(), "s" | "b" | "b*");
                if closes && !matches!(path.last(), None | Some(Segment::Close | Segment::Rect(..))) {
                    path.push(Segment::Close);
                }
                let paint = match op.as_str() {
                    "F" => "f",
                    "s" => "S",
                    "b" => "B",
                    "b*" => "B*",
                    other => other,
                };
                if paint != "n" {
                    w.paint(&state, &path, paint);
                }
                if let Some(even_odd) = pending_clip.take() {
                    add_clip(&mut state, &path, even_odd)?;
                }
                path.clear();
            }
            other => return fail(format!("PDF operator {other} is not supported by this actool")),
        }
        operands.clear();
    }
    if !path.is_empty() || !operands.is_empty() {
        return fail("a PDF's content ends inside a path");
    }
    if w.open {
        w.push("Q");
    }
    if let Some(e) = w.error.take() {
        return Err(e);
    }

    let mut content = String::new();
    let mut column = 0;
    for (i, t) in w.out.iter().enumerate() {
        if i > 0 {
            if column >= 73 {
                content.push('\n');
                column = 0;
            } else {
                content.push(' ');
                column += 1;
            }
        }
        content.push_str(t);
        column += t.len();
    }
    Ok((content, w.resources, (media.x1, media.y1)))
}

/// The file CoreGraphics writes for a content stream, with `profile`'s ICC
/// profiles.
fn assemble(content: &str, res: &Resources, (width, height): (f64, f64), profile: &dyn Fn(bool) -> Result<Vec<u8>, String>) -> Result<Vec<u8>, String> {
    let spaces = res.spaces.len();
    let alphas = res.alphas.len();
    let space_object = |i: usize| 5 + i;
    let alpha_object = |i: usize| 5 + spaces + i;
    let profile_object = |i: usize| 5 + spaces + alphas + i;
    let catalog = 5 + 2 * spaces + alphas;

    let mut file = b"%PDF-1.3\n%\xc4\xe5\xf2\xe5\xeb\xa7\xf3\xa0\xd0\xc4\xc6\n".to_vec();
    let mut offsets: HashMap<usize, usize> = HashMap::new();
    let mut object = |file: &mut Vec<u8>, n: usize, body: &[u8]| {
        offsets.insert(n, file.len());
        file.extend_from_slice(format!("{n} 0 obj\n").as_bytes());
        file.extend_from_slice(body);
        file.extend_from_slice(b"\nendobj\n");
    };
    let stream = |dict: &str, data: &[u8]| -> Vec<u8> {
        let deflated = crate::image::zlib_level_1(data);
        let mut s = wrap(&format!("<< {dict}/Length {} /Filter /FlateDecode >>", deflated.len())).into_bytes();
        s.extend_from_slice(b"\nstream\n");
        s.extend_from_slice(&deflated);
        s.extend_from_slice(b"\nendstream");
        s
    };
    object(&mut file, 4, &stream("", content.as_bytes()));
    object(&mut file, 1, b"<< /Type /Page /Parent 2 0 R /Resources 3 0 R /Contents 4 0 R >>");
    let mut dict = "<< /ProcSet [ /PDF ] ".to_string();
    if spaces > 0 {
        dict.push_str("/ColorSpace << ");
        for i in 0..spaces {
            dict.push_str(&format!("/Cs{} {} 0 R ", i + 1, space_object(i)));
        }
        dict.push_str(">> ");
    }
    if alphas > 0 {
        dict.push_str("/ExtGState << ");
        for i in 0..alphas {
            dict.push_str(&format!("/Gs{} {} 0 R ", i + 1, alpha_object(i)));
        }
        dict.push_str(">> ");
    }
    dict.push_str(">>");
    object(&mut file, 3, wrap(&dict).as_bytes());
    for (i, (key, value)) in res.alphas.iter().enumerate() {
        object(&mut file, alpha_object(i), wrap(&format!("<< /Type /ExtGState /{key} {} >>", number(*value)?)).as_bytes());
    }
    for (i, &gray) in res.spaces.iter().enumerate() {
        let icc = profile(gray)?;
        let dict = if gray { "/N 1 /Alternate /DeviceGray " } else { "/N 3 /Alternate /DeviceRGB " };
        object(&mut file, profile_object(i), &stream(dict, &icc));
        object(&mut file, space_object(i), format!("[ /ICCBased {} 0 R ]", profile_object(i)).as_bytes());
    }
    let pages = format!("<< /Type /Pages /MediaBox [0 0 {} {}] /Count 1 /Kids [ 1 0 R ] >>", number(width)?, number(height)?);
    object(&mut file, 2, wrap(&pages).as_bytes());
    object(&mut file, catalog, b"<< /Type /Catalog /Pages 2 0 R >>");
    object(&mut file, catalog + 1, b"<< >>");
    let size = catalog + 2;
    let xref = file.len();
    file.extend_from_slice(format!("xref\n0 {size}\n0000000000 65535 f \n").as_bytes());
    for n in 1..size {
        file.extend_from_slice(format!("{:010} 00000 n \n", offsets[&n]).as_bytes());
    }
    let zero = "<00000000000000000000000000000000>";
    let trailer = format!("<< /Size {size} /Root {catalog} 0 R /Info {} 0 R /ID [ {zero} {zero} ] >>", catalog + 1);
    file.extend_from_slice(format!("trailer\n{}\nstartxref\n{xref}\n%%EOF\n", wrap(&trailer)).as_bytes());
    Ok(file)
}

/// `text`'s tokens, a newline after one that ends a line of 73 characters.
fn wrap(text: &str) -> String {
    let mut out = String::new();
    let mut column = 0;
    for (i, t) in text.split(' ').enumerate() {
        if i > 0 {
            if column >= 73 {
                out.push('\n');
                column = 0;
            } else {
                out.push(' ');
                column += 1;
            }
        }
        out.push_str(t);
        column += t.len();
    }
    out
}

fn set_colour(state: &mut State, stroke: bool, colour: Colour) {
    if stroke {
        state.stroke.0 = colour;
    } else {
        state.fill.0 = colour;
    }
}

/// `path`, under the CTM, added to the clip: a rectangle is intersected
/// with the clip's rectangle, which is written after its paths (two
/// probes).
fn add_clip(state: &mut State, path: &[Segment], even_odd: bool) -> Result<(), String> {
    let path = transformed(path, &state.ctm);
    if let [Segment::Rect(x, y, w, h)] = path.as_slice() {
        let (x0, y0, x1, y1) = match state.clip_rect {
            Some([a, b, c, d]) => (a.max(*x), b.max(*y), (a + c).min(x + w), (b + d).min(y + h)),
            None => (*x, *y, x + w, y + h),
        };
        if x1 <= x0 || y1 <= y0 {
            return fail("an empty clip in a PDF is not supported by this actool");
        }
        state.clip_rect = Some([x0, y0, x1 - x0, y1 - y0]);
        return Ok(());
    }
    state.clip.push(Clip { path, even_odd });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A one-page PDF of `content`, as the probes were written.
    fn pdf(content: &str, resources: &str) -> Vec<u8> {
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 32 32] /Contents 4 0 R /Resources << {resources} >> >>"),
            format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
        ];
        let mut out = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for (i, o) in objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
        }
        let xref = out.len();
        out.extend_from_slice(format!("xref\n0 5\n0000000000 65535 f \n").as_bytes());
        for o in offsets {
            out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes());
        out
    }

    const ALPHA: &str = "/ExtGState << /A << /ca 0.5 >> /B << /ca 1 >> /S << /CA 0.25 >> /D << /CA 0.25 /ca 0.75 >> >>";

    /// Xcode 27.2's content streams for these pages.
    #[test]
    fn content_is_xcode_s() {
        let cases: &[(&str, &str, &str)] = &[
            ("0 0 1 rg 4 4 24 24 re f 1 0 0 rg 10 10 m 22 10 l 16 22 l h f", "",
             "q Q q /Cs1 cs 0 0 1 sc 4 4 24 24 re f /Cs1 cs 1 0 0 sc 10 10 m 22 10 l 16\n22 l h f Q"),
            ("0 g 1 1 30 30 re f", "", "q Q q 1 1 30 30 re f Q"),
            ("1 1 30 30 re n", "", "q Q"),
            ("0.123456789 0.5 0.25 rg 1.123456 2.5 29.987654 30.0001 re f", "",
             "q Q q /Cs1 cs 0.1234568 0.5 0.25 sc 1.123456 2.5 29.98765 30.0001 re f Q"),
            ("1 J 1 j 4 M [2 1] 0 d 3 w 0 0 1 RG 4 4 m 28 4 l 28 28 l S", "",
             "q Q q 3 w 1 J 1 j 4 M [ 2 1 ] 0 d /Cs1 CS 0 0 1 SC 4 4 m 28 4 l 28 28 l S\nQ"),
            ("q 2 0 0 2 0 0 cm 0 0 1 rg 1 1 10 10 re f Q", "", "q Q q /Cs1 cs 0 0 1 sc 2 2 20 20 re f Q"),
            ("0 1 0 rg 4 16 m 10 30 28 16 v 4 4 28 4 y h f", "",
             "q Q q /Cs1 cs 0 1 0 sc 4 16 m 4 16 10 30 28 16 c 4 4 28 4 28 4 c h f Q"),
            ("0 0 1 rg 2 2 28 28 re 8 8 16 16 re f*", "",
             "q Q q /Cs1 cs 0 0 1 sc 2 2 m 30 2 l 30 30 l 2 30 l h 8 8 m 24 8 l 24 24 l\n8 24 l h f* Q"),
            ("q 2 0 0 2 0 0 cm 3 w 1 0 0 RG 2 2 m 10 10 l S Q 3 w 2 20 m 30 20 l S", "",
             "q Q q 3 w /Cs1 CS 1 0 0 SC q 2 0 0 2 0 0 cm 2 2 m 10 10 l S Q /Cs2 CS 0 SC\n2 20 m 30 20 l S Q"),
            ("q 0 1 -1 0 32 0 cm 1 0 0 rg 2 2 10 10 re f Q", "", "q Q q /Cs1 cs 1 0 0 sc 20 2 10 10 re f Q"),
            ("q 1 0 0 -1 0 32 cm 1 0 0 rg 2 2 10 5 re f Q", "", "q Q q /Cs1 cs 1 0 0 sc 2 30 m 12 30 l 12 25 l 2 25 l h f Q"),
            ("q 0.8660254 0.5 -0.5 0.8660254 16 4 cm 1 0 0 rg 0 0 10 10 re f Q", "",
             "q Q q /Cs1 cs 1 0 0 sc 16 4 m 24.66025 9 l 19.66025 17.66025 l 11 12.66025\nl h f Q"),
            ("q 2 2 20 20 re W n q 4 4 20 20 re W n 1 0 0 rg 0 0 32 32 re f Q 0 0 1 rg 0 0 32 32 re f Q 0 1 0 rg 1 1 2 2 re f", "",
             "q Q q 4 4 18 18 re W n /Cs1 cs 1 0 0 sc 0 0 32 32 re f Q q 2 2 20 20 re W\nn /Cs1 cs 0 0 1 sc 0 0 32 32 re f Q q /Cs1 cs 0 1 0 sc 1 1 2 2 re f Q"),
            ("/A gs 1 0 0 rg 1 1 5 5 re f /B gs 10 10 5 5 re f /A gs 20 20 5 5 re f", ALPHA,
             "q Q q /Cs1 cs 1 0 0 sc /Gs1 gs 1 1 5 5 re f /Cs1 cs 1 0 0 sc /Gs2 gs 10 10 5 5\nre f /Cs1 cs 1 0 0 sc /Gs1 gs 20 20 5 5 re f Q"),
            ("/D gs 1 0 0 rg 0 0 1 RG 4 4 20 20 re B", ALPHA,
             "q Q q /Cs1 cs 1 0 0 sc /Gs1 gs /Cs1 CS 0 0 1 SC /Gs2 gs 4 4 20 20 re B Q"),
            ("1 0 0 rg 0 0 1 RG 2 2 m 20 2 l 20 20 l b", "", "q Q q /Cs1 cs 1 0 0 sc /Cs1 CS 0 0 1 SC 2 2 m 20 2 l 20 20 l h B Q"),
            ("0 1 1 0 k 1 1 30 30 re f", "", "q Q q 0 1 1 0 k 1 1 30 30 re f Q"),
            // A path clip after a rectangle's is written before it.
            ("2 2 20 20 re W n 4 4 m 20 4 l 12 20 l h W n 1 0 0 rg 0 0 32 32 re f", "",
             "q Q q 4 4 m 20 4 l 12 20 l h W n 2 2 20 20 re W n /Cs1 cs 1 0 0 sc 0 0 32 32\nre f Q"),
            // Through a stroke's CTM and back: 0.1 is 0.0999999999999996.
            ("q 1 0 0 1 11.6 22.313 cm 0 0 1 RG 21.465 26 m 0.1 27.197 29 17.1 11.5 5.885 c S Q", "",
             "q Q q /Cs1 CS 0 0 1 SC q 1 0 0 1 11.6 22.313 cm 21.465 26 m 0.10 27.197 29\n17.1 11.5 5.885 c S Q Q"),
        ];
        for (content, resources, want) in cases {
            let (got, _, _) = draw(&pdf(content, resources)).unwrap_or_else(|e| panic!("{content}: {e}"));
            assert_eq!(&got, want, "{content}");
        }
    }

    #[test]
    fn numbers_are_xcode_s() {
        for (x, want) in [
            (0.0000005, "0"), (0.000001929900001, "0.000001929900"), (0.0000015, "0.0000015"), (0.0999981986, "0.09999820"),
            (0.0999999999999996, "0.10"), (0.5000004118, "0.5"), (0.8999997235, "0.9"), (0.0100000635, "0.01"),
            (0.1318000099, "0.1318000"), (0.6939910042, "0.6939910"), (0.9999995, "1"), (6.2105602811, "6.21056"),
            (123456.789, "123456.8"), (12345678.0, "12345678"), (-1.5, "-1.5"), (-0.0000001, "0"), (29.987654, "29.98765"),
        ] {
            assert_eq!(number(x).unwrap(), want, "{x}");
        }
        // At a threshold: not established.
        assert!(number(0.999999).is_err());
        assert!(number(0.0999999).is_err());
    }

    /// Whole files where no colour profile is needed: Xcode 27.2's, by
    /// length and CRC-32.
    #[test]
    fn files_are_xcode_s() {
        let no_profile = |_: bool| -> Result<Vec<u8>, String> { panic!("no profile is needed") };
        for (content, length, crc) in [("0 g 1 1 30 30 re f", 682, 0x23b6e06au32), ("0 1 1 0 k 1 1 30 30 re f", 689, 0xfe77bc9f)] {
            let (c, res, size) = draw(&pdf(content, "")).unwrap();
            let file = assemble(&c, &res, size, &no_profile).unwrap();
            assert_eq!((file.len(), crate::image::crc32_of(&file)), (length, crc), "{content}");
        }
    }
}
