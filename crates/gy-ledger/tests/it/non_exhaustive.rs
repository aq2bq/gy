//! Every public read type is `#[non_exhaustive]` (n-d903, r-c7a8, d-d62d,
//! ac-9a48): a new field on a view type is then a minor change, not a breaking
//! one. The check reads the source, as the asset guards read the shipped files.
use std::fs;
use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("the views directory") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn every_public_view_struct_is_non_exhaustive() {
    let views = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/views");
    let mut files = Vec::new();
    rust_files(&views, &mut files);
    assert!(!files.is_empty(), "no view source found under {views:?}");
    let mut missing = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).expect("a view source");
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if !line.trim_start().starts_with("pub struct ") {
                continue;
            }
            // The attribute sits directly above the item, under its docs.
            let mut at = index;
            let mut found = false;
            while at > 0 {
                at -= 1;
                let above = lines[at].trim();
                if above.is_empty() {
                    break;
                }
                if above == "#[non_exhaustive]" {
                    found = true;
                    break;
                }
                if above.starts_with("#[") || above.starts_with("///") || above.starts_with("//!") {
                    continue;
                }
                break;
            }
            if !found {
                missing.push(format!("{}:{}", file.display(), index + 1));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "public view structs without #[non_exhaustive]:\n{}",
        missing.join("\n")
    );
}
