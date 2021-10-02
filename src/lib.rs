//! A canonical JSON serializer that tries to be
//! compliant with [the OLPC minimal specification for canonical JSON][olpc].
//! Additionally, the implementation also tries to be fully compatible with the [Go
//! canonical JSON implementation][docker/go/canonical] used across the Docker and
//! Notary ecosystem.
//! Example - reading a JSON file and printing its canonical representation:

//! ```rust
//! let res: serde_json::Value =
//!     serde_json::from_reader(input).expect("cannot deserialize input file");

//! println!(
//!     "{}",
//!     cjson::to_string(&res).expect("cannot write canonical JSON")
//! );
//! ```
#![deny(missing_docs)]

use serde_json::{self, Value};
use std::{collections::BTreeMap, io};

#[cfg(test)]
mod tests;

/// Serialize the given data structure as canonical JSON into the IO stream.
pub fn to_writer<W, T: Sized>(mut writer: W, value: &T) -> Result<(), Error>
where
    W: io::Write,
    T: serde::ser::Serialize,
{
    let v = serde_json::to_value(value)?;
    let bytes = canonicalize(&v)?;
    writer.write_all(&bytes)?;
    Ok(())
}

/// Serialize the given data structure as a canonical JSON byte vector.
pub fn to_vec<T: Sized>(value: &T) -> Result<Vec<u8>, Error>
where
    T: serde::Serialize,
{
    let mut writer = Vec::new();
    to_writer(&mut writer, value)?;
    Ok(writer)
}

/// Serialize the given data structure as a String of canonical JSON.
pub fn to_string<T: Sized>(value: &T) -> Result<String, Error>
where
    T: serde::Serialize,
{
    Ok(unsafe { String::from_utf8_unchecked(to_vec(value)?) })
}

fn canonicalize(val: &serde_json::Value) -> Result<Vec<u8>, Error> {
    let cv = from_value(val)?;
    let mut buf = Vec::new();
    let _ = cv.write(&mut buf);
    Ok(buf)
}

enum CanonicalValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<CanonicalValue>),
    Object(BTreeMap<String, CanonicalValue>),
}

enum Number {
    I64(i64),
}

impl CanonicalValue {
    fn write(&self, mut buf: &mut Vec<u8>) -> Result<(), Error> {
        match *self {
            CanonicalValue::Null => {
                buf.extend(b"null");
                Ok(())
            }
            CanonicalValue::Bool(true) => {
                buf.extend(b"true");
                Ok(())
            }
            CanonicalValue::Bool(false) => {
                buf.extend(b"false");
                Ok(())
            }
            CanonicalValue::Number(Number::I64(n)) => match itoa::write(buf, n) {
                Ok(_) => Ok(()),
                Err(err) => Err(Error::Custom(format!("cannot write number: {}", err))),
            },
            CanonicalValue::String(ref s) => {
                let s = serde_json::to_string(&Value::String(s.clone()))?;
                buf.extend(s.as_bytes());
                Ok(())
            }
            CanonicalValue::Array(ref arr) => {
                buf.push(b'[');
                let mut first = true;
                for a in arr.iter() {
                    if !first {
                        buf.push(b',');
                    }
                    a.write(&mut buf)?;
                    first = false;
                }
                buf.push(b']');
                Ok(())
            }
            CanonicalValue::Object(ref obj) => {
                buf.push(b'{');
                let mut first = true;
                for (k, v) in obj.iter() {
                    if !first {
                        buf.push(b',');
                    }
                    first = false;
                    let k = serde_json::to_string(&Value::String(k.clone()))?;
                    buf.extend(k.as_bytes());
                    buf.push(b':');
                    v.write(&mut buf)?;
                }
                buf.push(b'}');
                Ok(())
            }
        }
    }
}

fn from_value(val: &Value) -> Result<CanonicalValue, Error> {
    match *val {
        Value::Null => Ok(CanonicalValue::Null),
        Value::Bool(b) => Ok(CanonicalValue::Bool(b)),
        Value::Number(ref n) => {
            let x = n.as_i64();
            match x {
                Some(x) => Ok(CanonicalValue::Number(Number::I64(x))),
                None => Err(Error::Custom(String::from(format!(
                    "unsupported value in canonical JSON: {}",
                    n
                )))),
            }
        }
        Value::String(ref s) => Ok(CanonicalValue::String(s.clone())),
        Value::Array(ref arr) => {
            let mut out = Vec::new();
            for res in arr.iter().map(|v| from_value(v)) {
                out.push(res?)
            }
            Ok(CanonicalValue::Array(out))
        }
        Value::Object(ref obj) => {
            let mut out = BTreeMap::new();
            for (k, v) in obj.iter() {
                let _ = out.insert(k.clone(), from_value(v)?);
            }
            Ok(CanonicalValue::Object(out))
        }
    }
}

/// This enum represents all errors that can be returned when
/// trying to serialize something as canonical JSON.
#[derive(Debug)]
pub enum Error {
    /// Custom generic error.
    Custom(String),
    /// IO error.
    Io(io::Error),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        use serde_json::error::Category;
        match err.classify() {
            Category::Io => Error::Io(err.into()),
            Category::Syntax | Category::Data | Category::Eof => {
                Error::Custom(String::from(format!("{}", err)))
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}
