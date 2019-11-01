use serde_json;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io;

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

pub fn to_vec<T: Sized>(value: &T) -> Result<Vec<u8>, Error>
where
    T: serde::Serialize,
{
    let mut writer = Vec::with_capacity(128);
    to_writer(&mut writer, value)?;
    Ok(writer)
}

pub fn to_string<T: Sized>(value: &T) -> Result<String, Error>
where
    T: serde::Serialize,
{
    Ok(unsafe { String::from_utf8_unchecked(to_vec(value)?) })
}

pub fn canonicalize(val: &serde_json::Value) -> Result<Vec<u8>, Error> {
    let cj = convert(val)?;
    let mut buf = Vec::new();
    let _ = cj.write(&mut buf);
    Ok(buf)
}

enum CanonicalValue {
    Array(Vec<CanonicalValue>),
    Bool(bool),
    Null,
    Number(Number),
    Object(BTreeMap<String, CanonicalValue>),
    String(String),
}

enum Number {
    I64(i64),
}

impl CanonicalValue {
    fn write(&self, mut buf: &mut Vec<u8>) -> Result<(), String> {
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
            CanonicalValue::Number(Number::I64(n)) => itoa::write(buf, n)
                .map(|_| ())
                .map_err(|err| format!("Write error: {}", err)),
            CanonicalValue::String(ref s) => {
                let s = serde_json::Value::String(s.clone());
                let s = serde_json::to_string(&s).map_err(|e| format!("{:?}", e))?;
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

                    let k = serde_json::Value::String(k.clone());
                    let k = serde_json::to_string(&k).map_err(|e| format!("{:?}", e))?;
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

fn convert(val: &Value) -> Result<CanonicalValue, Error> {
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
        Value::Array(ref arr) => {
            let mut out = Vec::new();
            for res in arr.iter().map(|v| convert(v)) {
                out.push(res?)
            }
            Ok(CanonicalValue::Array(out))
        }
        Value::Object(ref obj) => {
            let mut out = BTreeMap::new();
            for (k, v) in obj.iter() {
                let _ = out.insert(k.clone(), convert(v)?);
            }
            Ok(CanonicalValue::Object(out))
        }
        Value::String(ref s) => Ok(CanonicalValue::String(s.clone())),
    }
}

#[derive(Debug)]
pub enum Error {
    Custom(String),
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
