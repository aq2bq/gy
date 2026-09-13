//! One-to-one matching of reported paths against bounded file declarations.
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TokenClass {
    AsciiDigits,
    LowercaseHex,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Declaration<'a> {
    Literal {
        path: &'a str,
    },
    Generated {
        directory: &'a str,
        prefix: &'a str,
        suffix: &'a str,
        token_class: TokenClass,
        token_length: u64,
    },
}

fn relative_path(path: &str) -> bool {
    let drive_prefix = path.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
        && path.as_bytes().get(1) == Some(&b':');
    if path.trim().is_empty() || path.contains('\\') || drive_prefix {
        return false;
    }
    path.split('/')
        .all(|p| !p.is_empty() && p != "." && p != "..")
}

impl Declaration<'_> {
    fn validate(&self) -> Result<(), String> {
        match self {
            Self::Literal { path } if relative_path(path) => Ok(()),
            Self::Literal { .. } => Err("literal path must be a canonical relative path".into()),
            Self::Generated {
                directory,
                prefix,
                suffix,
                token_length,
                ..
            } => {
                if (!directory.is_empty() && !relative_path(directory))
                    || prefix.contains(['/', '\\'])
                    || suffix.contains(['/', '\\'])
                    || (prefix.is_empty() && suffix.is_empty())
                    || *token_length == 0
                {
                    return Err("generated declaration requires a fixed relative directory (or empty for root), separator-free prefix/suffix with at least one nonempty, and a positive token_length".into());
                }
                Ok(())
            }
        }
    }

    fn matches(&self, path: &str) -> bool {
        match self {
            Self::Literal { path: expected } => path == *expected,
            Self::Generated {
                directory,
                prefix,
                suffix,
                token_class,
                token_length,
            } => {
                let (parent, filename) = path.rsplit_once('/').unwrap_or(("", path));
                if parent != *directory {
                    return false;
                }
                let Some(token) = filename
                    .strip_prefix(prefix)
                    .and_then(|s| s.strip_suffix(suffix))
                else {
                    return false;
                };
                token.len() as u64 == *token_length
                    && token.bytes().all(|b| match token_class {
                        TokenClass::AsciiDigits => b.is_ascii_digit(),
                        TokenClass::LowercaseHex => {
                            b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
                        }
                    })
            }
        }
    }
}

pub(super) fn match_declared_files(left: &[&Value], right: &[&Value]) -> Result<(), String> {
    let ([left], [right]) = (left, right) else {
        return Err("operands must each resolve to one array".into());
    };
    let (Some(files), Some(declarations)) = (left.as_array(), right.as_array()) else {
        return Err("operands must each resolve to one array".into());
    };
    let files = files
        .iter()
        .enumerate()
        .map(|(i, value)| {
            value
                .as_str()
                .filter(|p| relative_path(p))
                .ok_or_else(|| format!("left[{i}] must be a canonical relative path string"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut matches_per_file = vec![0; files.len()];
    for (i, value) in declarations.iter().enumerate() {
        let declaration = Declaration::deserialize(value)
            .map_err(|e| format!("right[{i}]: invalid file declaration: {e}"))?;
        declaration
            .validate()
            .map_err(|e| format!("right[{i}]: {e}"))?;
        let mut count = 0;
        for (j, file) in files.iter().enumerate() {
            if declaration.matches(file) {
                count += 1;
                matches_per_file[j] += 1;
            }
        }
        if count != 1 {
            return Err(format!(
                "right[{i}] matches {count} reported files; expected exactly one"
            ));
        }
    }
    for (i, count) in matches_per_file.into_iter().enumerate() {
        if count != 1 {
            return Err(format!(
                "left[{i}] ({}) matches {count} declarations; expected exactly one",
                files[i]
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn compare(files: Value, declarations: Value) -> Result<(), String> {
        match_declared_files(&[&files], &[&declarations])
    }
    fn generated() -> Value {
        json!({"kind":"generated", "directory":"db/migrate", "prefix":"",
            "suffix":"_add_keys.rb", "token_class":"ascii-digits", "token_length":14})
    }

    #[test]
    fn matches_only_the_declared_location_shape_and_count() {
        let path = "db/migrate/20260913090000_add_keys.rb";
        let declarations = json!([{"kind":"literal","path":"src/api.rs"}, generated()]);
        assert!(compare(json!([path, "src/api.rs"]), declarations.clone()).is_ok());
        for files in [
            json!([path, "src/api.rs", "other.rs"]),
            json!([path]),
            json!([path, path, "src/api.rs"]),
            json!([path, "db/migrate/20260913090001_add_keys.rb", "src/api.rs"]),
            json!(["other/20260913090000_add_keys.rb", "src/api.rs"]),
            json!(["db/migrate/20260913090000_drop_keys.rb", "src/api.rs"]),
            json!(["db/migrate/2026091309000_add_keys.rb", "src/api.rs"]),
            json!(["db/migrate/202609130900000_add_keys.rb", "src/api.rs"]),
            json!(["db/migrate/2026091309000a_add_keys.rb", "src/api.rs"]),
            json!([
                "db/migrate/２０２６０９１３０９００００_add_keys.rb",
                "src/api.rs"
            ]),
        ] {
            assert!(
                compare(files.clone(), declarations.clone()).is_err(),
                "{files}"
            );
        }
        for declarations in [
            json!([generated(), generated()]),
            json!([generated(), {"kind":"literal", "path":path}]),
        ] {
            assert!(
                compare(json!([path]), declarations)
                    .unwrap_err()
                    .contains("matches 2 declarations")
            );
        }
    }

    #[test]
    fn hex_root_paths_and_metacharacters_are_literal() {
        let declaration = json!([{"kind":"generated", "directory":"", "prefix":"asset-",
            "suffix":".js", "token_class":"lowercase-hex", "token_length":4}]);
        assert!(compare(json!(["asset-0a1f.js"]), declaration.clone()).is_ok());
        for path in [
            "asset-0A1F.js",
            "asset-0g1f.js",
            "asset-0a1f.js.map",
            "dir/asset-0a1f.js",
        ] {
            assert!(compare(json!([path]), declaration.clone()).is_err());
        }
        let literal = json!([{"kind":"literal", "path":"src/*.rs"}]);
        assert!(compare(json!(["src/*.rs"]), literal.clone()).is_ok());
        assert!(compare(json!(["src/api.rs"]), literal).is_err());
        assert!(compare(json!([]), json!([])).is_ok()); // Schemas govern empty arrays.
        assert!(compare(json!(["x"]), json!([])).is_err());
        assert!(compare(json!([]), json!([generated()])).is_err());
    }

    #[test]
    fn declarations_and_operands_reject_invalid_shapes() {
        for path in [
            "", " ", "/root", "./x", "a/../x", "a//x", "a/", "a\\x", "C:/x",
        ] {
            assert!(
                compare(json!([path]), json!([{"kind":"literal", "path":path}])).is_err(),
                "{path}"
            );
        }
        for (key, value) in [
            ("directory", json!("../db")),
            ("prefix", json!("a/")),
            ("suffix", json!("a\\b")),
            ("suffix", json!("")),
            ("token_class", json!("any")),
            ("token_length", json!(0)),
            ("token_length", json!(-1)),
            ("token_length", json!(1.5)),
            ("token_length", json!("14")),
            ("unexpected", json!(true)),
        ] {
            let mut d = generated();
            d[key] = value;
            assert!(
                compare(json!(["db/migrate/20260913090000_add_keys.rb"]), json!([d])).is_err(),
                "{key}"
            );
        }
        for key in [
            "directory",
            "prefix",
            "suffix",
            "token_class",
            "token_length",
            "kind",
        ] {
            let mut d = generated();
            d.as_object_mut().unwrap().remove(key);
            assert!(compare(json!(["x"]), json!([d])).is_err());
        }
        for (files, declarations) in [
            (json!([7]), json!([generated()])),
            (json!("x"), json!([])),
            (json!([]), json!({})),
            (json!(["x"]), json!(["x"])),
            (json!(["x"]), json!([{"kind":"glob", "path":"**"}])),
        ] {
            assert!(compare(files, declarations).is_err());
        }
    }
}
