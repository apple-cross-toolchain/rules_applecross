// cgprobe W H: reads cov commands (m l c h f e n) and prints the model's alpha, top row first, like cov.
#[path = "/tmp/cgprobe/cgfill_base.rs"]
#[allow(dead_code)]
mod cgfill;
use cgfill::{coverage, polygons, Element, Point, Rule};
use std::io::BufRead;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (w, h): (usize, usize) = (a[1].parse().unwrap(), a[2].parse().unwrap());
    let mut path = Vec::new();
    let mut rule = Rule::NonZero;
    let mut out = String::new();
    let mut dump = |path: &Vec<Element>, rule: Rule, out: &mut String| {
        let cov = coverage(&polygons(path), w, h, rule);
        if std::env::var("CG_RAW").is_ok() {
            let raw = cgfill::RAW.lock().unwrap().clone();
            for y in (0..h).rev() { out.push_str(&(0..w).map(|x| format!("{:.6}", raw[y * w + x])).collect::<Vec<_>>().join(" ")); out.push('\n'); }
        } else {
        for r in cov.iter().rev() { out.push_str(&r.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ")); out.push('\n'); }
        }
        out.push_str("--\n");
    };
    let mut fills: Vec<Element> = Vec::new();
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.is_empty() { continue; }
        let v: Vec<f64> = t[1..].iter().map(|s| s.parse().unwrap()).collect();
        let p = |i: usize| Point { x: v[i], y: v[i + 1] };
        match t[0] {
            "m" => path.push(Element::Move(p(0))),
            "l" => path.push(Element::Line(p(0))),
            "c" => path.push(Element::Cubic(p(0), p(2), p(4))),
            "h" => path.push(Element::Close),
            "f" => { rule = Rule::NonZero; fills = std::mem::take(&mut path); }
            "e" => { rule = Rule::EvenOdd; fills = std::mem::take(&mut path); }
            "n" => { dump(&fills, rule, &mut out); fills.clear(); }
            _ => {}
        }
    }
    dump(&fills, rule, &mut out);
    print!("{out}");
}
