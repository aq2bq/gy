# d-75c2 (D-35) N-16 の分割は gy_core 直下の既存の公開パスと型の形を維持し、workflow モジュールは非公開のまま内部で分割する

## 関係
- 無し

decision_scope: crates/gy-core の lib.rs は mod workflow と pub use workflow::* のまま。下流の gy_core::RecordSchema 等の import と型の形は d334001 と同一。公開 API の追加は含めない（Rust 公開 API は 0.4.0 と互換、z の変更）。
scope: agent_footprint
created: 2026-09-13
  closed-by q-3693 (Q-28) N-16 の API 維持対象を実在する crate 直下の公開パスとするか、workflow 配下の公開パスも追加するか

## Context

## Decision

## Consequences


