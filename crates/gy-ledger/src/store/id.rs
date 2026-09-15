//! Short-hash ID minting: the seed, the widening, and the letter guarantee
//! (D-74, n-f088).
use super::{Error, Result};
use std::{
    collections::BTreeSet,
    time::{SystemTime, UNIX_EPOCH},
};

/// The seed a new id hashes: the prefix, the wall clock in nanoseconds, the
/// process id, the node count, and a per-process counter. Two processes in the
/// same second still differ (D-74).
pub(crate) fn id_seed(prefix: &str, count: usize, salt: u64) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    format!("{prefix}:{nanos}:{}:{count}:{salt}", std::process::id())
}

/// Widen the short hash until it is unique among `used` full ids (D-74):
/// four hex digits, then six, then eight. Each width hashes a different seed,
/// so widening is not a longer window on the same bits. An all-digit hash is
/// re-minted (n-f088): a hash always carries a letter a-f, so it cannot look
/// like an old id and the zero-padded aliases cannot collide with it.
pub fn unique_hash(prefix: &str, seed: &str, used: &BTreeSet<String>) -> Result<String> {
    for length in [4, 6, 8] {
        let Some(hash) = lettered_hash(seed, length) else {
            continue;
        };
        if !used.contains(&format!("{prefix}-{hash}")) {
            return Ok(hash);
        }
    }
    Err(Error::invalid("no unused id hash remains"))
}

/// The first hash of this width that carries a letter; an all-digit candidate
/// is skipped. A collision is handled by widening, not by another attempt.
fn lettered_hash(seed: &str, length: usize) -> Option<String> {
    for attempt in 0..16 {
        let hex = format!(
            "{:08x}",
            super::fnv1a(&format!("{seed}:{length}:{attempt}"))
        );
        let hash = &hex[..length];
        if !hash.bytes().all(|byte| byte.is_ascii_digit()) {
            return Some(hash.to_string());
        }
    }
    None
}
