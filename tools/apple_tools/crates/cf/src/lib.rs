//! Property-list values and the byte-exact encodings Apple's tools emit.

pub mod binary;
pub mod hash;
pub mod xml;

/// A property-list value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Array(Vec<Value>),
    Boolean(bool),
    Data(Vec<u8>),
    /// Seconds since 2001-01-01T00:00:00Z, as CFDate stores them.
    Date(f64),
    Dictionary(Vec<(String, Value)>),
    Integer(i64),
    Real(f64),
    String(String),
}

impl Value {
    pub fn dict<K: Into<String>>(entries: impl IntoIterator<Item = (K, Value)>) -> Value {
        Value::Dictionary(entries.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    pub fn string(s: impl Into<String>) -> Value {
        Value::String(s.into())
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Dictionary(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Reads any property list (XML, binary or OpenStep-free JSON-less text)
    /// the `plist` crate understands, keeping dictionary order as written.
    pub fn parse(bytes: &[u8]) -> Result<Value, String> {
        let value = plist::Value::from_reader(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
        Ok(Value::from_plist(value))
    }

    fn from_plist(value: plist::Value) -> Value {
        match value {
            plist::Value::Array(a) => Value::Array(a.into_iter().map(Value::from_plist).collect()),
            plist::Value::Boolean(b) => Value::Boolean(b),
            plist::Value::Data(d) => Value::Data(d),
            plist::Value::Date(d) => {
                let t: std::time::SystemTime = d.into();
                let unix = match t.duration_since(std::time::UNIX_EPOCH) {
                    Ok(d) => d.as_secs_f64(),
                    Err(e) => -e.duration().as_secs_f64(),
                };
                Value::Date(unix - 978_307_200.0)
            }
            plist::Value::Dictionary(d) => Value::Dictionary(d.into_iter().map(|(k, v)| (k, Value::from_plist(v))).collect()),
            plist::Value::Integer(i) => Value::Integer(i.as_signed().unwrap_or_else(|| i.as_unsigned().unwrap() as i64)),
            plist::Value::Real(r) => Value::Real(r),
            plist::Value::String(s) => Value::String(s),
            plist::Value::Uid(u) => Value::Integer(u.get() as i64),
            _ => Value::String(String::new()),
        }
    }
}
