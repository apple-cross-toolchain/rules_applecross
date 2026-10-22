//! XML property lists, as CoreFoundation writes them.

use crate::Value;
use base64::Engine;

const HEADER: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n";

pub fn to_string(value: &Value) -> String {
    let mut out = String::from(HEADER);
    write_value(&mut out, value, 0);
    out.push_str("</plist>\n");
    out
}

pub fn to_bytes(value: &Value) -> Vec<u8> {
    to_string(value).into_bytes()
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push('\t');
    }
}

fn escape(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
}

/// Orders strings as CFStringCompare does without options: by UTF-16 code unit.
pub fn compare_keys(a: &str, b: &str) -> std::cmp::Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

fn write_value(out: &mut String, value: &Value, level: usize) {
    indent(out, level);
    match value {
        Value::String(s) => {
            out.push_str("<string>");
            escape(out, s);
            out.push_str("</string>\n");
        }
        Value::Integer(i) => out.push_str(&format!("<integer>{i}</integer>\n")),
        Value::Real(r) => out.push_str(&format!("<real>{}</real>\n", format_real(*r))),
        Value::Boolean(true) => out.push_str("<true/>\n"),
        Value::Boolean(false) => out.push_str("<false/>\n"),
        Value::Date(t) => out.push_str(&format!("<date>{}</date>\n", format_date(*t))),
        Value::Data(d) => {
            out.push_str("<data>\n");
            let encoded = base64::engine::general_purpose::STANDARD.encode(d);
            let width = line_width(level);
            for line in encoded.as_bytes().chunks(width) {
                indent(out, level);
                out.push_str(std::str::from_utf8(line).unwrap());
                out.push('\n');
            }
            indent(out, level);
            out.push_str("</data>\n");
        }
        Value::Array(items) if items.is_empty() => out.push_str("<array/>\n"),
        Value::Array(items) => {
            out.push_str("<array>\n");
            for item in items {
                write_value(out, item, level + 1);
            }
            indent(out, level);
            out.push_str("</array>\n");
        }
        Value::Dictionary(entries) if entries.is_empty() => out.push_str("<dict/>\n"),
        Value::Dictionary(entries) => {
            out.push_str("<dict>\n");
            let mut sorted: Vec<&(String, Value)> = entries.iter().collect();
            sorted.sort_by(|a, b| compare_keys(&a.0, &b.0));
            for (key, item) in sorted {
                indent(out, level + 1);
                out.push_str("<key>");
                escape(out, key);
                out.push_str("</key>\n");
                write_value(out, item, level + 1);
            }
            indent(out, level);
            out.push_str("</dict>\n");
        }
    }
}

/// Base64 characters per line at an indentation level: 76 columns with a tab
/// counted as eight, rounded down to whole 4-character groups.
fn line_width(level: usize) -> usize {
    let columns = 76usize.saturating_sub(8 * level);
    (columns / 4 * 4).max(4)
}

/// `%.17g`, the format CFNumber's description uses for doubles.
pub fn format_real(r: f64) -> String {
    if r.is_nan() {
        return "nan".into();
    }
    if r.is_infinite() {
        return if r > 0.0 { "+infinity".into() } else { "-infinity".into() };
    }
    if r == 0.0 {
        return if r.is_sign_negative() { "-0".into() } else { "0".into() };
    }
    let precision = 17i32;
    // %e with 16 digits after the point gives the 17 significant digits and
    // the decimal exponent %g decides with.
    let sci = format!("{:.*e}", (precision - 1) as usize, r);
    let (mantissa, exp) = sci.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    if exp < -4 || exp >= precision {
        let mantissa = trim_fraction(mantissa);
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{mantissa}e{sign}{:02}", exp.abs())
    } else {
        let decimals = (precision - 1 - exp).max(0) as usize;
        trim_fraction(&format!("{:.*}", decimals, r)).to_string()
    }
}

fn trim_fraction(s: &str) -> &str {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        s
    }
}

/// ISO 8601 in UTC with whole seconds, as CF writes `<date>`.
fn format_date(t: f64) -> String {
    let unix = (t + 978_307_200.0).floor() as i64;
    let days = unix.div_euclid(86_400);
    let secs = unix.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", secs / 3600, secs / 60 % 60, secs % 60)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reals_match_cf() {
        for (r, s) in [(1000.0, "1000"), (0.5, "0.5"), (0.1, "0.10000000000000001"), (1e20, "1e+20"),
                       (1e-7, "9.9999999999999995e-08"), (-2.5, "-2.5"), (3.0, "3"), (123456789.123, "123456789.123")] {
            assert_eq!(format_real(r), s);
        }
    }
}
