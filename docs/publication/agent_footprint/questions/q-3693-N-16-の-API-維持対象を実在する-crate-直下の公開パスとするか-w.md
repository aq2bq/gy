# q-3693 (Q-28) N-16 の API 維持対象を実在する crate 直下の公開パスとするか、workflow 配下の公開パスも追加するか

state: closed
decider: master
options: 実在する gy_core::RecordSchema 等のパスと形を維持し、要求のパス表記を訂正する（推奨）, 既存の crate 直下のパスを維持し、gy_core::workflow 配下も新たに公開する
scope: agent_footprint
created: 2026-09-13
  closes d-75c2 (D-35) N-16 の分割は gy_core 直下の既存の公開パスと型の形を維持し、workflow モジュールは非公開のまま内部で分割する
N-16 の要求2は gy_core::workflow::RecordSchema を既存の公開パスとして維持する指定だが、基準 d334001 の lib.rs:10 は mod workflow、同17行は pub use workflow::* である。
下流クレートで指定のパスを cargo check --locked すると E0603（workflow is private）、実在する gy_core::RecordSchema に置き換えると exit 0。ログは /private/tmp/gy-n16-api-probe.log と /private/tmp/gy-n16-root-api-probe.log。
案Aは実在する crate 直下の公開パスと型の形を維持し、要求のパス表記を訂正する。内部分割のみになり、公開面を増やさない。案Bは既存のパスに加えて workflow 配下を公開する。指定の import を可能にするが、公開 API が追加される。
推奨は案A。N-16 の目的である関心ごとの物理分割を、既存の公開契約を維持して実現できる。
分割前の cargo test --workspace --locked は68 passed、0 failed、0 ignored（15+7+37+9）。製品ファイルは未変更。API 方針の回答後に実装へ進む。

閉じ方: 決定（2026-09-13T11:56:27.304070+00:00）

