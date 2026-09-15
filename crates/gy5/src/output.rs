//! Printing results: `Display` by default, `Serialize` under `--json`, and the
//! JSON error shape (`{"code", "message"}` on stdout).
use gy_ledger::{Error, Result};
use serde::Serialize;
use std::fmt::Display;

pub fn emit<T: Display + Serialize>(json: bool, value: &T) -> Result<()> {
    if json {
        println!("{}", to_json(value)?);
    } else {
        print!("{value}");
    }
    Ok(())
}

pub fn emit_list<T: Display + Serialize>(json: bool, values: &[T]) -> Result<()> {
    if json {
        println!("{}", to_json(values)?);
    } else {
        for value in values {
            print!("{value}");
        }
    }
    Ok(())
}

pub fn to_json<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|error| Error::invalid(error.to_string()))
}

pub fn report(json: bool, error: &Error) {
    if json {
        println!(
            "{}",
            serde_json::json!({"code": 2, "message": error.message})
        );
    } else {
        eprintln!("{}", error.message);
    }
}
