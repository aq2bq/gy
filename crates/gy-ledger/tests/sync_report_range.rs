//! A reversed write span reads as zero writes (n-8b91 3): `report.rs` must not
//! underflow when `from > to`.
use gy_ledger::{Range, Sync};

#[test]
fn a_reversed_range_reads_as_zero_writes() {
    let sync = Sync {
        pushed: Some(Range { from: 5, to: 3 }),
        rebased: Some(Range { from: 2, to: 0 }),
        ..Default::default()
    };

    let line = sync.to_string();

    assert!(line.contains("pushed: 0 writes (seq 5 → 3)"), "{line}");
    assert!(line.contains("rebased: 0 writes (seq 2 → 0)"), "{line}");
}

#[test]
fn a_normal_range_reads_as_its_size() {
    let sync = Sync {
        pushed: Some(Range { from: 3, to: 5 }),
        ..Default::default()
    };

    assert!(sync.to_string().contains("pushed: 3 writes (seq 3 → 5)"));
}

#[test]
fn a_single_write_range_reads_as_one() {
    let sync = Sync {
        rebased: Some(Range { from: 4, to: 4 }),
        ..Default::default()
    };

    assert!(sync.to_string().contains("rebased: 1 writes (seq 4 → 4)"));
}
