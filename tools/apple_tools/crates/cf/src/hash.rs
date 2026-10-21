//! CoreFoundation's string hash (`__CFStrHashCharacters`, CFString.c) and
//! the hash-table orders that writers following Apple's output need.

/// Strings up to this many UTF-16 units hash every unit; longer ones hash the
/// first, middle and last 32.
const HASH_EVERYTHING_LIMIT: usize = 96;

/// `CFHash` of a string, for 64-bit CoreFoundation.
pub fn string_hash(s: &str) -> u64 {
    let units: Vec<u16> = s.encode_utf16().collect();
    hash_units(&units)
}

fn four(result: u64, u: &[u16]) -> u64 {
    result
        .wrapping_mul(67_503_105)
        .wrapping_add((u[0] as u64).wrapping_mul(16_974_593))
        .wrapping_add((u[1] as u64).wrapping_mul(66_049))
        .wrapping_add((u[2] as u64).wrapping_mul(257))
        .wrapping_add(u[3] as u64)
}

pub fn hash_units(u: &[u16]) -> u64 {
    let len = u.len();
    let mut result = len as u64;
    if len <= HASH_EVERYTHING_LIMIT {
        let end4 = len & !3;
        let mut i = 0;
        while i < end4 {
            result = four(result, &u[i..i + 4]);
            i += 4;
        }
        while i < len {
            result = result.wrapping_mul(257).wrapping_add(u[i] as u64);
            i += 1;
        }
    } else {
        for start in [0, (len >> 1) - 16, len - 32] {
            let mut i = start;
            while i < start + 32 {
                result = four(result, &u[i..i + 4]);
                i += 4;
            }
        }
    }
    result.wrapping_add(result << (len & 31))
}

/// Bucket counts of CoreFoundation's hash tables and the entries each holds
/// before growing, from the published CFBasicHash sources.
const TABLE_SIZES: [usize; 22] = [0, 3, 7, 13, 23, 41, 71, 127, 191, 251, 383, 631, 1087, 1723, 2803, 4523, 7351, 11959, 19447, 31231, 50683, 81919];
const TABLE_CAPACITIES: [usize; 22] = [0, 3, 6, 11, 19, 32, 52, 85, 118, 155, 237, 390, 672, 1065, 1732, 2795, 4543, 7391, 12019, 19302, 31324, 50629];

/// Enumeration order (indices into `keys`) of a CoreFoundation hash table
/// filled in `keys` order.
pub fn table_order(keys: &[&str]) -> Vec<usize> {
    let hashes: Vec<u64> = keys.iter().map(|k| string_hash(k)).collect();
    let mut level = 0;
    let mut table: Vec<Option<usize>> = Vec::new();
    for (n, &h) in hashes.iter().enumerate() {
        if n + 1 > TABLE_CAPACITIES[level] {
            while n + 1 > TABLE_CAPACITIES[level] {
                level += 1;
            }
            let old: Vec<usize> = table.iter().flatten().copied().collect();
            table = vec![None; TABLE_SIZES[level]];
            for k in old {
                insert(&mut table, hashes[k], k);
            }
        }
        insert(&mut table, h, n);
    }
    table.into_iter().flatten().collect()
}

/// The same for a table created at the size its entries need, as
/// CoreFoundation creates a dictionary from all its entries at once (reading
/// a binary property list): no growth, so no rehashing.
pub fn sized_table_order(keys: &[&str]) -> Vec<usize> {
    let mut level = 0;
    while TABLE_CAPACITIES[level] < keys.len() {
        level += 1;
    }
    let mut table: Vec<Option<usize>> = vec![None; TABLE_SIZES[level]];
    for (n, k) in keys.iter().enumerate() {
        insert(&mut table, string_hash(k), n);
    }
    table.into_iter().flatten().collect()
}

fn insert(table: &mut [Option<usize>], hash: u64, key: usize) {
    let size = table.len();
    let mut i = (hash % size as u64) as usize;
    while table[i].is_some() {
        i = (i + 1) % size;
    }
    table[i] = Some(key);
}

/// NSMutableSet bucket counts and growth thresholds; see
/// docs/clean-room/foundation-hashing.md.
const SET_SIZES: [usize; 12] = [3, 7, 13, 23, 41, 71, 127, 191, 251, 383, 631, 1087];
const SET_GROWTH: [usize; 12] = [3, 7, 13, 21, 34, 54, 87, 120, 157, 239, usize::MAX, usize::MAX];

/// The order in which an NSMutableSet of strings, filled in `keys` order,
/// enumerates them. Returns indices into `keys`.
pub fn set_order(keys: &[&str]) -> Vec<usize> {
    assert!(keys.len() < 400, "set enumeration order is established for fewer than 400 entries");
    let hashes: Vec<u64> = keys.iter().map(|k| string_hash(k)).collect();
    let mut level = 0;
    let mut table: Vec<Option<usize>> = vec![None; SET_SIZES[0]];
    let rehash = |table: &mut Vec<Option<usize>>, level: usize| {
        let old: Vec<usize> = table.iter().flatten().copied().collect();
        *table = vec![None; SET_SIZES[level]];
        for k in old {
            insert(table, hashes[k], k);
        }
    };
    for (n, &h) in hashes.iter().enumerate() {
        let count = n + 1;
        if level < 2 && count > SET_GROWTH[level] {
            while level < 2 && count > SET_GROWTH[level] {
                level += 1;
            }
            rehash(&mut table, level);
        }
        insert(&mut table, h, n);
        if level >= 2 && count >= SET_GROWTH[level] {
            while count >= SET_GROWTH[level] {
                level += 1;
            }
            rehash(&mut table, level);
        }
    }
    table.into_iter().flatten().collect()
}
