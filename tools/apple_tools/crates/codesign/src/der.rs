//! Entitlements in the DER form `--generate-entitlement-der` embeds.

use cf::Value;

fn length(out: &mut Vec<u8>, n: usize) {
    if n < 0x80 {
        out.push(n as u8);
    } else {
        let bytes: Vec<u8> = n.to_be_bytes().iter().copied().skip_while(|&b| b == 0).collect();
        out.push(0x80 | bytes.len() as u8);
        out.extend(bytes);
    }
}

fn tlv(out: &mut Vec<u8>, tag: &[u8], content: &[u8]) {
    out.extend_from_slice(tag);
    length(out, content.len());
    out.extend_from_slice(content);
}

fn integer(i: i64) -> Vec<u8> {
    let mut bytes = i.to_be_bytes().to_vec();
    while bytes.len() > 1 && ((bytes[0] == 0 && bytes[1] & 0x80 == 0) || (bytes[0] == 0xff && bytes[1] & 0x80 != 0)) {
        bytes.remove(0);
    }
    bytes
}

fn value(out: &mut Vec<u8>, v: &Value) -> Result<(), String> {
    match v {
        Value::Boolean(b) => tlv(out, &[0x01], &[if *b { 0xff } else { 0x00 }]),
        Value::Integer(i) => tlv(out, &[0x02], &integer(*i)),
        Value::String(s) => tlv(out, &[0x0c], s.as_bytes()),
        Value::Array(items) => {
            let mut content = Vec::new();
            for item in items {
                value(&mut content, item)?;
            }
            tlv(out, &[0x30], &content);
        }
        Value::Dictionary(entries) => {
            let mut sorted: Vec<&(String, Value)> = entries.iter().collect();
            sorted.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            let mut content = Vec::new();
            for (k, item) in sorted {
                let mut pair = Vec::new();
                tlv(&mut pair, &[0x0c], k.as_bytes());
                value(&mut pair, item)?;
                tlv(&mut content, &[0x30], &pair);
            }
            tlv(out, &[0xb0], &content);
        }
        other => return Err(format!("entitlement values of this kind have no DER form: {other:?}")),
    }
    Ok(())
}

pub fn encode(entitlements: &Value) -> Result<Vec<u8>, String> {
    let mut content = Vec::new();
    tlv(&mut content, &[0x02], &[0x01]);
    value(&mut content, entitlements)?;
    let mut out = Vec::new();
    tlv(&mut out, &[0x70], &content);
    Ok(out)
}
