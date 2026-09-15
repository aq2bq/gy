//! Printing results: `Display` by default, `Serialize` under `--json`, and the
//! JSON error shape (`{"code", "message"}` on stdout).
use gy_ledger::{Error, Result};
use serde::Serialize;
use std::fmt::Display;
use std::io::{self, Write};

/// Write one string to stdout. A closed pipe (`gy ... | head`) ends the
/// process quietly instead of panicking (N-70).
pub fn write(text: &str) -> Result<()> {
    let mut out = io::stdout().lock();
    match out.write_all(text.as_bytes()).and_then(|()| out.flush()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn emit<T: Display + Serialize>(json: bool, value: &T) -> Result<()> {
    if json {
        write(&format!("{}\n", to_json(value)?))
    } else {
        write(&value.to_string())
    }
}

pub fn emit_list<T: Display + Serialize>(json: bool, values: &[T]) -> Result<()> {
    if json {
        return write(&format!("{}\n", to_json(values)?));
    }
    let mut text = String::new();
    for value in values {
        text.push_str(&value.to_string());
    }
    write(&text)
}

pub fn to_json<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|error| Error::invalid(error.to_string()))
}

pub fn report(json: bool, error: &Error) {
    if json {
        let _ = write(&format!(
            "{}\n",
            serde_json::json!({"code": 2, "message": error.message})
        ));
    } else {
        eprintln!("{}", error.message);
    }
}
