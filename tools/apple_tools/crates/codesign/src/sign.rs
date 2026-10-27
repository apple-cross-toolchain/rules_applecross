//! The embedded signature of one Mach-O image, ad hoc or by an identity; see
//! docs/clean-room/codesign.md.

use crate::identity::Identity;
use macho::{Image, MH_EXECUTE};
use sha1::Sha1;
use sha2::{Digest, Sha256};

const CSMAGIC_EMBEDDED_SIGNATURE: u32 = 0xfade_0cc0;
const CSMAGIC_CODEDIRECTORY: u32 = 0xfade_0c02;
const CSMAGIC_EMBEDDED_ENTITLEMENTS: u32 = 0xfade_7171;
const CSMAGIC_EMBEDDED_DER_ENTITLEMENTS: u32 = 0xfade_7172;
const CSMAGIC_BLOBWRAPPER: u32 = 0xfade_0b01;

const CS_ADHOC: u32 = 0x2;
const CS_RUNTIME: u32 = 0x10000;
const CS_HASHTYPE_SHA256: u8 = 2;
const CD_VERSION: u32 = 0x20400;
/// The version that adds `runtime` and `preEncryptOffset`, which codesign
/// writes for the hardened runtime.
const CD_VERSION_RUNTIME: u32 = 0x20500;

/// What codesign reserves for the CMS blob of an identity signature, its
/// 8-byte header included, whatever the CMS itself comes to.
const CMS_RESERVATION: usize = 18000;

const CS_EXECSEG_MAIN_BINARY: u64 = 0x1;
const CS_EXECSEG_ALLOW_UNSIGNED: u64 = 0x10;
const CS_EXECSEG_DEBUGGER: u64 = 0x20;
const CS_EXECSEG_JIT: u64 = 0x40;
const CS_EXECSEG_SKIP_LV: u64 = 0x80;
const CS_EXECSEG_CAN_LOAD_CDHASH: u64 = 0x100;
const CS_EXECSEG_CAN_EXEC_CDHASH: u64 = 0x200;

/// Special slot numbers.
pub const SLOT_INFO: usize = 1;
pub const SLOT_REQUIREMENTS: usize = 2;
pub const SLOT_RESOURCES: usize = 3;
pub const SLOT_ENTITLEMENTS: usize = 5;
pub const SLOT_DER_ENTITLEMENTS: usize = 7;

pub struct Inputs<'a> {
    pub identifier: &'a str,
    pub info_plist: Option<&'a [u8]>,
    pub code_resources: Option<&'a [u8]>,
    pub entitlements: Option<&'a [u8]>,
    pub der_entitlements: Option<&'a [u8]>,
    /// Entitlement keys whose `true` value sets an executable-segment flag.
    pub entitlement_flags: u64,
    /// `--options runtime`: the hardened runtime.
    pub runtime: bool,
    /// The identity to sign with; ad hoc without one.
    pub identity: Option<&'a Identity>,
    /// Seconds since the Unix epoch, for an identity signature's signingTime.
    pub signing_time: u64,
    /// The team identifier, which Apple-issued identities carry.
    pub team: Option<&'a str>,
    /// The timestamp authority to ask, for a secure timestamp.
    pub timestamp_url: Option<&'a str>,
}


pub fn exec_seg_flags(entitlements: Option<&cf::Value>) -> u64 {
    let Some(e) = entitlements else { return 0 };
    let on = |k: &str| e.get(k).and_then(|v| v.as_bool()) == Some(true);
    let mut flags = 0;
    if on("get-task-allow") {
        flags |= CS_EXECSEG_ALLOW_UNSIGNED;
    }
    if on("com.apple.private.cs.debugger") {
        flags |= CS_EXECSEG_DEBUGGER;
    }
    if on("dynamic-codesigning") {
        flags |= CS_EXECSEG_JIT;
    }
    if on("com.apple.private.skip-library-validation") {
        flags |= CS_EXECSEG_SKIP_LV;
    }
    if on("com.apple.private.amfi.can-load-cdhash") {
        flags |= CS_EXECSEG_CAN_LOAD_CDHASH;
    }
    if on("com.apple.private.amfi.can-execute-cdhash") {
        flags |= CS_EXECSEG_CAN_EXEC_CDHASH;
    }
    flags
}

fn be32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn be64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn blob(magic: u32, payload: &[u8]) -> Vec<u8> {
    let mut b = Vec::with_capacity(payload.len() + 8);
    be32(&mut b, magic);
    be32(&mut b, (payload.len() + 8) as u32);
    b.extend_from_slice(payload);
    b
}

/// The page size an image's code is hashed in, as observed from codesign
/// (docs/clean-room/codesign.md): 16 KiB for arm64, except iOS devices below
/// iOS 16 and tvOS devices; 4 KiB otherwise.
pub fn page_size(image: &Image) -> usize {
    const IOS_16: u32 = 16 << 16;
    let small = match image.build_version {
        _ if image.cputype != macho::CPU_TYPE_ARM64 => true,
        Some((macho::PLATFORM_IOS, minos)) => minos < IOS_16,
        Some((macho::PLATFORM_TVOS, _)) => true,
        _ => false,
    };
    if small { 4096 } else { 16384 }
}

fn code_directory(image: &Image, code_limit: usize, inputs: &Inputs, requirements: &[u8]) -> Vec<u8> {
    let page = page_size(image);
    let mut special: Vec<Option<[u8; 32]>> = vec![None; 8];
    let digest = |d: &[u8]| -> [u8; 32] { Sha256::digest(d).into() };
    special[SLOT_REQUIREMENTS] = Some(digest(requirements));
    if let Some(d) = inputs.info_plist {
        special[SLOT_INFO] = Some(digest(d));
    }
    if let Some(d) = inputs.code_resources {
        special[SLOT_RESOURCES] = Some(digest(d));
    }
    if let Some(d) = inputs.entitlements {
        special[SLOT_ENTITLEMENTS] = Some(blob_digest(CSMAGIC_EMBEDDED_ENTITLEMENTS, d));
    }
    if let Some(d) = inputs.der_entitlements {
        special[SLOT_DER_ENTITLEMENTS] = Some(blob_digest(CSMAGIC_EMBEDDED_DER_ENTITLEMENTS, d));
    }
    let n_special = (1..special.len()).rev().find(|&i| special[i].is_some()).unwrap_or(0);
    let n_code = code_limit.div_ceil(page);

    let header_len = if inputs.runtime { 96usize } else { 88usize };
    let ident_offset = header_len;
    let team_offset = ident_offset + inputs.identifier.len() + 1;
    let team_len = inputs.team.map_or(0, |t| t.len() + 1);
    let hash_offset = team_offset + team_len + 32 * n_special;
    let length = hash_offset + 32 * n_code;

    let text = image.segment("__TEXT");
    let mut flags = inputs.entitlement_flags;
    if image.filetype == MH_EXECUTE {
        flags |= CS_EXECSEG_MAIN_BINARY;
    }

    let mut cd = Vec::with_capacity(length);
    be32(&mut cd, CSMAGIC_CODEDIRECTORY);
    be32(&mut cd, length as u32);
    be32(&mut cd, if inputs.runtime { CD_VERSION_RUNTIME } else { CD_VERSION });
    let mut cd_flags = if inputs.identity.is_some() { 0 } else { CS_ADHOC };
    if inputs.runtime {
        cd_flags |= CS_RUNTIME;
    }
    be32(&mut cd, cd_flags);
    be32(&mut cd, hash_offset as u32);
    be32(&mut cd, ident_offset as u32);
    be32(&mut cd, n_special as u32);
    be32(&mut cd, n_code as u32);
    be32(&mut cd, code_limit as u32);
    cd.push(32);
    cd.push(CS_HASHTYPE_SHA256);
    cd.push(0); // platform
    cd.push(page.trailing_zeros() as u8);
    be32(&mut cd, 0); // spare2
    be32(&mut cd, 0); // scatterOffset
    be32(&mut cd, if inputs.team.is_some() { team_offset as u32 } else { 0 });
    be32(&mut cd, 0); // spare3
    be64(&mut cd, 0); // codeLimit64
    be64(&mut cd, text.map_or(0, |t| t.fileoff));
    be64(&mut cd, text.map_or(0, |t| t.filesize));
    be64(&mut cd, flags);
    if inputs.runtime {
        // The runtime version is the SDK the image was built against.
        be32(&mut cd, image.sdk_version.unwrap_or(0));
        be32(&mut cd, 0); // preEncryptOffset
    }
    debug_assert_eq!(cd.len(), header_len);
    cd.extend_from_slice(inputs.identifier.as_bytes());
    cd.push(0);
    if let Some(team) = inputs.team {
        cd.extend_from_slice(team.as_bytes());
        cd.push(0);
    }
    for slot in (1..=n_special).rev() {
        cd.extend_from_slice(&special[slot].unwrap_or([0; 32]));
    }
    for chunk in image.data[..code_limit].chunks(page) {
        cd.extend_from_slice(&Sha256::digest(chunk));
    }
    cd
}

fn blob_digest(magic: u32, payload: &[u8]) -> [u8; 32] {
    Sha256::digest(blob(magic, payload)).into()
}

fn super_blob(blobs: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let header = 12 + 8 * blobs.len();
    let total = header + blobs.iter().map(|(_, b)| b.len()).sum::<usize>();
    let mut out = Vec::with_capacity(total);
    be32(&mut out, CSMAGIC_EMBEDDED_SIGNATURE);
    be32(&mut out, total as u32);
    be32(&mut out, blobs.len() as u32);
    let mut offset = header;
    for (slot, b) in blobs {
        be32(&mut out, *slot);
        be32(&mut out, offset as u32);
        offset += b.len();
    }
    for (_, b) in blobs {
        out.extend_from_slice(b);
    }
    out
}

/// Signs `image` in place. Returns the code directory, whose SHA-256 is the
/// image's cdhash.
pub fn sign(image: &mut Image, inputs: &Inputs) -> Result<Vec<u8>, String> {
    let requirements = match inputs.identity {
        Some(id) => crate::requirement::designated_set(&id.designated(inputs.identifier)?),
        None => crate::requirement::empty_set(),
    };
    let code_limit = image.unsigned_end();
    // The code directory hashes the image with its load commands already
    // describing the signature, so reserve first. The size depends only on
    // lengths, which are known before hashing.
    let page = page_size(image);
    let n_code = {
        // Reserving may add a 16-byte load command but never moves the code
        // limit, which ends where the signature begins.
        let dataoff = match image.code_signature {
            Some((_, off, _)) => off as usize,
            None => (code_limit + 15) & !15,
        };
        dataoff.div_ceil(page)
    };
    let mut n_special = SLOT_REQUIREMENTS;
    if inputs.info_plist.is_some() {
        n_special = n_special.max(SLOT_INFO);
    }
    if inputs.code_resources.is_some() {
        n_special = n_special.max(SLOT_RESOURCES);
    }
    if inputs.entitlements.is_some() {
        n_special = n_special.max(SLOT_ENTITLEMENTS);
    }
    if inputs.der_entitlements.is_some() {
        n_special = n_special.max(SLOT_DER_ENTITLEMENTS);
    }
    let header_len = if inputs.runtime { 96 } else { 88 };
    let cd_len = header_len + inputs.identifier.len() + 1 + inputs.team.map_or(0, |t| t.len() + 1) + 32 * (n_special + n_code);
    let mut lengths = vec![cd_len, requirements.len()];
    if let Some(e) = inputs.entitlements {
        lengths.push(e.len() + 8);
    }
    if let Some(e) = inputs.der_entitlements {
        lengths.push(e.len() + 8);
    }
    lengths.push(8);
    let super_len = 12 + 8 * lengths.len() + lengths.iter().sum::<usize>();
    // codesign estimates with the 88-byte code directory header even when it
    // writes the runtime's 96 bytes, so the estimate is 8 short of them.
    let estimate = super_len - (header_len - 88);
    let reserved = match inputs.identity {
        // Everything before the CMS blob, the fixed reservation for it, and
        // the same 8 bytes over an ad hoc signature gets.
        Some(_) => ((estimate - 8 + CMS_RESERVATION + 8 + 15) & !15) as u32,
        None => ((estimate + 8 + 15) & !15) as u32,
    };

    // __LINKEDIT's size in memory is rounded up to 16 KiB whatever the
    // architecture or the page size code is hashed in.
    let dataoff = image.allocate_signature(reserved, 0x4000).map_err(|e| e.to_string())? as usize;
    let cd = code_directory(image, dataoff, inputs, &requirements);
    debug_assert_eq!(cd.len(), cd_len);

    let mut blobs = vec![(0u32, cd.clone()), (2, requirements)];
    if let Some(e) = inputs.entitlements {
        blobs.push((5, blob(CSMAGIC_EMBEDDED_ENTITLEMENTS, e)));
    }
    if let Some(e) = inputs.der_entitlements {
        blobs.push((7, blob(CSMAGIC_EMBEDDED_DER_ENTITLEMENTS, e)));
    }
    let cms = match inputs.identity {
        Some(id) => crate::cms::signed_data(&cd, id, inputs.signing_time, inputs.timestamp_url)?,
        None => Vec::new(),
    };
    if 8 + cms.len() > CMS_RESERVATION {
        return Err(format!("the CMS signature ({} bytes) exceeds the {CMS_RESERVATION} bytes reserved for it", cms.len()));
    }
    blobs.push((0x10000, blob(CSMAGIC_BLOBWRAPPER, &cms)));
    let sb = super_blob(&blobs);
    image.data[dataoff..dataoff + sb.len()].copy_from_slice(&sb);
    Ok(cd)
}

/// The code directory of an already signed image, if it has one.
pub fn existing_code_directory(image: &Image) -> Option<Vec<u8>> {
    let (_, dataoff, datasize) = image.code_signature?;
    let sig = image.data.get(dataoff as usize..(dataoff + datasize) as usize)?;
    let be = |o: usize| u32::from_be_bytes(sig[o..o + 4].try_into().unwrap());
    if be(0) != CSMAGIC_EMBEDDED_SIGNATURE {
        return None;
    }
    for i in 0..be(8) as usize {
        let (slot, offset) = (be(12 + 8 * i), be(16 + 8 * i) as usize);
        if slot == 0 {
            let len = u32::from_be_bytes(sig[offset + 4..offset + 8].try_into().unwrap()) as usize;
            return Some(sig[offset..offset + len].to_vec());
        }
    }
    None
}

/// The code directories of an already signed image: the primary (slot 0)
/// and any alternates (slots 0x1000 on), in slot order.
pub fn code_directories(image: &Image) -> Vec<(u32, Vec<u8>)> {
    let Some((_, dataoff, datasize)) = image.code_signature else { return Vec::new() };
    let Some(sig) = image.data.get(dataoff as usize..(dataoff + datasize) as usize) else { return Vec::new() };
    let be = |o: usize| sig.get(o..o + 4).map(|b| u32::from_be_bytes(b.try_into().unwrap()));
    if be(0) != Some(CSMAGIC_EMBEDDED_SIGNATURE) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for i in 0..be(8).unwrap_or(0) as usize {
        let (Some(slot), Some(offset)) = (be(12 + 8 * i), be(16 + 8 * i)) else { break };
        if slot == 0 || (0x1000..0x1005).contains(&slot) {
            let offset = offset as usize;
            if let Some(len) = be(offset + 4) {
                if let Some(cd) = sig.get(offset..offset + len as usize) {
                    out.push((slot, cd.to_vec()));
                }
            }
        }
    }
    out
}

/// An already signed image's designated requirement in codesign's text form,
/// if its requirement set has one: what a bundle's seal records for it.
pub fn designated_requirement_text(image: &Image) -> Result<Option<String>, String> {
    let Some((_, dataoff, datasize)) = image.code_signature else { return Ok(None) };
    let Some(sig) = image.data.get(dataoff as usize..(dataoff + datasize) as usize) else { return Ok(None) };
    let be = |o: usize| sig.get(o..o + 4).map(|x| u32::from_be_bytes(x.try_into().unwrap()));
    for i in 0..be(8).unwrap_or(0) as usize {
        if be(12 + 8 * i) == Some(2) {
            let off = be(16 + 8 * i).unwrap_or(0) as usize;
            let len = be(off + 4).unwrap_or(0) as usize;
            let set = sig.get(off..off + len).ok_or("nested code's requirement set is truncated")?;
            return Ok(crate::requirement::designated_in_set(set)?.as_ref().map(crate::requirement::text));
        }
    }
    Ok(None)
}

/// A code directory's cdhash: its digest by its own hash type, the first 20
/// bytes of it.
pub fn cdhash(cd: &[u8]) -> Option<Vec<u8>> {
    let digest = match *cd.get(37)? {
        1 => Sha1::digest(cd).to_vec(),
        CS_HASHTYPE_SHA256 => Sha256::digest(cd).to_vec(),
        _ => return None,
    };
    Some(digest[..20].to_vec())
}

/// The slice of a universal file codesign reads first: the one the signing
/// Mac would run. Xcode 27's Macs are all Apple silicon, preferring arm64e
/// to arm64; a file without either is not supported.
pub fn preferred_slice(images: &[&Image]) -> Option<usize> {
    const CPU_TYPE_ARM64: u32 = 0x0100_000c;
    if images.len() == 1 {
        return Some(0);
    }
    let arm64e = images.iter().position(|i| i.cputype == CPU_TYPE_ARM64 && i.cpusubtype & 0xff == 2);
    arm64e.or_else(|| images.iter().position(|i| i.cputype == CPU_TYPE_ARM64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn designated_requirement_is_codesigns() {
        let identity = Identity::from_pem(
            include_str!("../tests/data/identity/test-key.pem"),
            include_str!("../tests/data/identity/test-chain.pem"),
        )
        .unwrap();
        assert!(identity.matches("RAC Test Developer"));
        assert!(!identity.matches("Apple Development"));
        assert_eq!(
            crate::requirement::designated_set(&identity.designated("com.example.CleanRoom").unwrap()),
            include_bytes!("../tests/data/identity/apple-requirements.bin").to_vec()
        );
    }
}
