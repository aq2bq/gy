//! Release numbers in three numbers, and nothing else (n-670a). gy ships no
//! pre-releases: anything but `N.N.N` is unreadable, never comparable. Both
//! the release check and the peer notice read through here, so there is one
//! comparison.
use std::cmp::Ordering;

/// Three numbers, or nothing.
pub fn triple(text: &str) -> Option<(u64, u64, u64)> {
    let parts: Vec<&str> = text.split('.').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || part.bytes().any(|b| !b.is_ascii_digit()))
    {
        return None;
    }
    let numbers: Vec<u64> = parts.iter().filter_map(|part| part.parse().ok()).collect();
    if numbers.len() != 3 {
        return None;
    }
    Some((numbers[0], numbers[1], numbers[2]))
}

/// True when `latest` is strictly newer than `own`.
pub fn newer_than(own: &str, latest: &str) -> bool {
    match (triple(own), triple(latest)) {
        (Some(own), Some(latest)) => latest.cmp(&own) == Ordering::Greater,
        _ => false,
    }
}

/// The greatest release in `versions`, ignoring what does not parse.
pub fn max_version<'a, I>(versions: I) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
{
    versions
        .into_iter()
        .filter_map(|version| triple(version).map(|triple| (triple, version)))
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, version)| version.to_string())
}

#[cfg(test)]
mod tests {
    use super::{max_version, newer_than, triple};

    #[test]
    fn numbers_compare_numerically() {
        assert_eq!(triple("1.10.0"), Some((1, 10, 0)));
        assert!(newer_than("1.9.0", "1.10.0"));
        assert!(!newer_than("1.10.0", "1.10.0"));
        assert!(!newer_than("1.10.0", "1.9.0"));
    }

    #[test]
    fn anything_else_is_unreadable() {
        assert_eq!(triple("1.2"), None);
        assert_eq!(triple("1.2.3.4"), None);
        assert_eq!(triple("1.2.x"), None);
        assert_eq!(triple("1.2.3 (from ./x)"), None);
        assert!(!newer_than("1.1.1", "garbage"));
        assert_eq!(
            max_version(["garbage", "9.9.9", "2.0.0"]),
            Some("9.9.9".to_string())
        );
        assert_eq!(max_version(["garbage"]), None);
    }
}
