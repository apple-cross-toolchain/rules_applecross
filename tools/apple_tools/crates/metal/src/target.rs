//! Deployment targets: AIR and language versions, triples, metallib header fields.
//! The tables are from Xcode's output for our probes (FORMAT.md).

use regex::Regex;
use std::sync::LazyLock;

#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub os: String, // ios, macosx, tvos, xros
    pub version: (u32, u32, u32),
    pub simulator: bool,
}

/// (AIR major, minor) and (language major, minor).
pub type Versions = ((u32, u32), (u32, u32));

impl Target {
    /// `air64-apple-ios27.2[-simulator]` (as rules_apple passes to `metal -target`).
    pub fn parse(t: &str) -> Result<Target, String> {
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^air64-apple-(ios|macosx|tvos|xros)(\d+)(?:\.(\d+))?(?:\.(\d+))?(-simulator)?$").unwrap());
        let c = RE.captures(t).ok_or_else(|| format!("unsupported target {t}"))?;
        let n = |i: usize| c.get(i).map(|m| m.as_str().parse().unwrap()).unwrap_or(0);
        Ok(Target { os: c[1].to_string(), version: (n(2), n(3), n(4)), simulator: c.get(5).is_some() })
    }

    /// From an AIR triple: `air64_v29-apple-ios27.2.0[-simulator]`.
    pub fn from_triple(t: &str) -> Result<Target, String> {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^air64_v\d+-apple-(.*)$").unwrap());
        let c = RE.captures(t).ok_or_else(|| format!("not an AIR triple: {t}"))?;
        Target::parse(&format!("air64-apple-{}", &c[1]))
    }

    pub fn versions(&self) -> Versions {
        let (major, minor) = (self.version.0, self.version.1);
        let air_minor = if self.os == "macosx" {
            match major {
                10 => match minor {
                    15 => 2,
                    _ => 1,
                },
                11 => 3,
                12 => 4,
                13 => 5,
                14 => 6,
                15 => 7,
                26 => 8,
                27 => 9,
                m if m > 27 => 9,
                _ => 7,
            }
        } else if self.os == "xros" {
            match major {
                1 => 6,
                2 => 7,
                26 => 8,
                27 => 9,
                m if m > 27 => 9,
                _ => 6,
            }
        } else {
            // iOS and tvOS share numbering
            match major {
                11 => 0,
                12 => 1,
                13 => 2,
                14 => 3,
                15 => 4,
                16 => 5,
                17 => 6,
                18 => 7,
                26 => 8,
                27 => 9,
                m if m > 27 => 9,
                _ => 0,
            }
        };
        let lang = [(2, 0), (2, 1), (2, 2), (2, 3), (2, 4), (3, 0), (3, 1), (3, 2), (4, 0), (4, 1)][air_minor as usize];
        ((2, air_minor), lang)
    }

    pub fn air(&self) -> (u32, u32) {
        self.versions().0
    }

    /// `air64_v29-apple-ios27.2.0[-simulator]`
    pub fn triple(&self) -> String {
        let (v0, v1, v2) = self.version;
        format!("air64_v2{}-apple-{}{}.{}.{}{}", self.air().1, self.os, v0, v1, v2, if self.simulator { "-simulator" } else { "" })
    }

    /// The six u16s after `MTLB`.
    pub fn header(&self) -> [u16; 6] {
        let air = self.air();
        let family = if self.os == "macosx" { 0x8001 } else { 1 };
        let filever = match air.1 {
            1 => 3,
            3 => 5,
            5 | 6 => 7,
            7 => 8,
            8 | 9 => 9,
            _ => 7,
        };
        if air < (2, 5) {
            // no platform or deployment target before file version 7
            return [family, 2, filever, 0, 0, 0];
        }
        let mut plat = match self.os.as_str() {
            "macosx" => 1,
            "ios" => 2,
            "tvos" => 3,
            _ => 11,
        };
        if self.simulator && self.os == "ios" {
            plat = 7;
        }
        [family, 2, filever, (0x80 | plat) << 8, self.version.0 as u16, self.version.1 as u16]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_and_headers() {
        let t = Target::parse("air64-apple-ios27.2").unwrap();
        assert_eq!(t.versions(), ((2, 9), (4, 1)));
        assert_eq!(t.triple(), "air64_v29-apple-ios27.2.0");
        assert_eq!(t.header(), [1, 2, 9, 0x8200, 27, 2]);
        assert_eq!(Target::parse("air64-apple-ios12.0").unwrap().header(), [1, 2, 3, 0, 0, 0]);
        assert_eq!(Target::parse("air64-apple-macosx11.0").unwrap().header(), [0x8001, 2, 5, 0, 0, 0]);
        assert_eq!(Target::parse("air64-apple-ios17.0-simulator").unwrap().header(), [1, 2, 7, 0x8700, 17, 0]);
        assert_eq!(Target::parse("air64-apple-xros2.0").unwrap().header(), [1, 2, 8, 0x8b00, 2, 0]);
        assert_eq!(Target::from_triple("air64_v29-apple-ios27.2.0").unwrap(), t);
    }
}
