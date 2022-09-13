use std::string::FromUtf8Error;
use std::{fmt, io};

mod read;

pub struct LogFile {
    version: u16,
    /// time in ms
    time: Vec<u32>,
    entries: Vec<DataEntry>,
}

pub struct DataEntry {
    name: String,
    kind: EntryKind,
}

#[derive(Clone, Debug)]
pub enum EntryKind {
    Bool(Vec<bool>),

    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    U64(Vec<u64>),

    I8(Vec<i8>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    I64(Vec<i64>),

    F32(Vec<f32>),
    F64(Vec<f64>),
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Utf8(FromUtf8Error),
    InvalidMagic(String),
    UnknownVersion(u16),
    UnknownDatatype(u8),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(error) => write!(f, "Error reading files: {}", error),
            Self::Utf8(error) => write!(f, "Error decoding utf8 string: {}", error),
            Self::InvalidMagic(magic) => write!(f, "Invalid magic number: {}", magic),
            Self::UnknownVersion(version) => write!(f, "Unknown version: {}", version),
            Self::UnknownDatatype(code) => write!(f, "Unknown datatype code: {}", code),
        }
    }
}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Self {
        Self::IO(inner)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(inner: FromUtf8Error) -> Self {
        Self::Utf8(inner)
    }
}

pub struct SanityError(String);

fn sanity_check(entries: &[DataEntry]) -> Option<SanityError> {
    for e in entries {
        match &e.kind {
            EntryKind::Bool(v) => None,
            EntryKind::U8(v) => check_all(v, &e.name, sanity_check_u8),
            EntryKind::U16(v) => check_all(v, &e.name, sanity_check_u16),
            EntryKind::U32(v) => check_all(v, &e.name, sanity_check_u32),
            EntryKind::U64(v) => check_all(v, &e.name, sanity_check_u64),
            EntryKind::I8(v) => check_all(v, &e.name, sanity_check_i8),
            EntryKind::I16(v) => check_all(v, &e.name, sanity_check_i16),
            EntryKind::I32(v) => check_all(v, &e.name, sanity_check_i32),
            EntryKind::I64(v) => check_all(v, &e.name, sanity_check_i64),
            EntryKind::F32(v) => check_all(v, &e.name, sanity_check_f32),
            EntryKind::F64(v) => check_all(v, &e.name, sanity_check_f64),
        }
    }
}

fn check_all<T>(
    values: &[T],
    name: &str,
    check: impl Fn(&T, &str) -> Result<(), SanityError>,
) -> Result<(), SanityError> {
    for entry in values {
        check(entry, name)?;
    }
    Ok(())
}

macro_rules! impl_sanity_check_int {
    ($ident:ident, $ty:ty) => {
        fn $ident(val: $ty, name: &str) -> Result<(), SanityError> {
            if val == <$ty>::MAX {
                return Err(SanityError(format!("'{name}' is max")));
            }
            if val == <$ty>::MIN {
                return Err(SanityError(format!("'{name}' is min")));
            }
            Ok(())
        }
    };
}
impl_sanity_check_int!(sanity_check_u8, u8);
impl_sanity_check_int!(sanity_check_u16, u16);
impl_sanity_check_int!(sanity_check_u32, u32);
impl_sanity_check_int!(sanity_check_u64, u64);
impl_sanity_check_int!(sanity_check_i8, i8);
impl_sanity_check_int!(sanity_check_i16, i16);
impl_sanity_check_int!(sanity_check_i32, i32);
impl_sanity_check_int!(sanity_check_i64, i64);

macro_rules! Impl_sanity_check_float {
    ($ident:ident, $ty:ty) => {
        fn $ident(val: $ty, name: &str) -> Result<(), SanityError> {
            if val.is_nan() {
                return Err(SanityError(format!("'{name}' is nan")));
            }
            if val.is_infinite() {
                return Err(SanityError(format!("'{name}' is infinite")));
            }
            Ok(())
        }
    };
}
Impl_sanity_check_float!(sanity_check_f32, f32);
Impl_sanity_check_float!(sanity_check_f64, f64);
