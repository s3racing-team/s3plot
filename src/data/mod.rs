use std::string::FromUtf8Error;
use std::sync::Arc;
use std::{fmt, io};

use chrono::NaiveDateTime;

use crate::app::{PlotData, PlotValues};
pub use crate::data::read::read_file;
pub use crate::data::sanity::sanity_check;
use crate::eval;
use crate::plot::Config;

mod read;
mod sanity;

#[derive(Debug)]
pub struct LogStream {
    pub version: Version,
    pub start: Option<NaiveDateTime>,
    /// time in ms
    pub time: Vec<u32>,
    pub entries: Vec<DataEntry>,
}

impl LogStream {
    pub fn len(&self) -> usize {
        self.time.len()
    }

    pub fn header_matches(&self, other: &Self) -> bool {
        if self.entries.len() != other.entries.len() {
            return false;
        }

        for (a, b) in self.entries.iter().zip(other.entries.iter()) {
            if !a.kind.matches(&b.kind) {
                return false;
            }
        }

        true
    }

    pub fn reserve(&mut self, additional: usize) {
        self.time.reserve(additional);
        for e in self.entries.iter_mut() {
            e.kind.reserve(additional);
        }
    }

    pub fn extend(&mut self, other: &Self) {
        self.time.extend_from_slice(&other.time);
        for (e, o) in self.entries.iter_mut().zip(other.entries.iter()) {
            e.kind.extend(&o.kind);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Version {
    V1,
    V2,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::V1 => write!(f, "v1"),
            Version::V2 => write!(f, "v2"),
        }
    }
}

#[derive(Debug)]
pub struct DataEntry {
    pub name: String,
    pub kind: EntryKind,
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

impl EntryKind {
    pub fn reserve(&mut self, additional: usize) {
        match self {
            EntryKind::Bool(v) => v.reserve(additional),
            EntryKind::U8(v) => v.reserve(additional),
            EntryKind::U16(v) => v.reserve(additional),
            EntryKind::U32(v) => v.reserve(additional),
            EntryKind::U64(v) => v.reserve(additional),
            EntryKind::I8(v) => v.reserve(additional),
            EntryKind::I16(v) => v.reserve(additional),
            EntryKind::I32(v) => v.reserve(additional),
            EntryKind::I64(v) => v.reserve(additional),
            EntryKind::F32(v) => v.reserve(additional),
            EntryKind::F64(v) => v.reserve(additional),
        }
    }

    pub fn matches(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (EntryKind::Bool(_), EntryKind::Bool(_))
                | (EntryKind::U8(_), EntryKind::U8(_))
                | (EntryKind::U16(_), EntryKind::U16(_))
                | (EntryKind::U32(_), EntryKind::U32(_))
                | (EntryKind::U64(_), EntryKind::U64(_))
                | (EntryKind::I8(_), EntryKind::I8(_))
                | (EntryKind::I16(_), EntryKind::I16(_))
                | (EntryKind::I32(_), EntryKind::I32(_))
                | (EntryKind::I64(_), EntryKind::I64(_))
                | (EntryKind::F32(_), EntryKind::F32(_))
                | (EntryKind::F64(_), EntryKind::F64(_))
        )
    }

    pub fn extend(&mut self, other: &Self) {
        match (self, other) {
            (EntryKind::Bool(a), EntryKind::Bool(b)) => a.extend_from_slice(b),
            (EntryKind::U8(a), EntryKind::U8(b)) => a.extend_from_slice(b),
            (EntryKind::U16(a), EntryKind::U16(b)) => a.extend_from_slice(b),
            (EntryKind::U32(a), EntryKind::U32(b)) => a.extend_from_slice(b),
            (EntryKind::U64(a), EntryKind::U64(b)) => a.extend_from_slice(b),
            (EntryKind::I8(a), EntryKind::I8(b)) => a.extend_from_slice(b),
            (EntryKind::I16(a), EntryKind::I16(b)) => a.extend_from_slice(b),
            (EntryKind::I32(a), EntryKind::I32(b)) => a.extend_from_slice(b),
            (EntryKind::I64(a), EntryKind::I64(b)) => a.extend_from_slice(b),
            (EntryKind::F32(a), EntryKind::F32(b)) => a.extend_from_slice(b),
            (EntryKind::F64(a), EntryKind::F64(b)) => a.extend_from_slice(b),
            _ => (),
        }
    }

    pub fn get_f64(&self, index: usize) -> f64 {
        match self {
            EntryKind::Bool(v) => v[index] as u8 as f64,
            EntryKind::U8(v) => v[index] as f64,
            EntryKind::U16(v) => v[index] as f64,
            EntryKind::U32(v) => v[index] as f64,
            EntryKind::U64(v) => v[index] as f64,
            EntryKind::I8(v) => v[index] as f64,
            EntryKind::I16(v) => v[index] as f64,
            EntryKind::I32(v) => v[index] as f64,
            EntryKind::I64(v) => v[index] as f64,
            EntryKind::F32(v) => v[index] as f64,
            EntryKind::F64(v) => v[index],
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Utf8(FromUtf8Error),
    InvalidMagic([u8; 4]),
    UnknownVersion(u16),
    UnknownDatatype(u8),
    InvalidTimestamp(i64),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(error) => write!(f, "Error reading files: {}", error),
            Self::Utf8(error) => write!(f, "Error decoding utf8 string: {}", error),
            Self::InvalidMagic(magic) => match std::str::from_utf8(magic) {
                Ok(m) => write!(f, "Invalid magic number: {m}"),
                Err(_) => write!(f, "Invalid magic number: {magic:?}"),
            },
            Self::UnknownVersion(version) => write!(f, "Unknown version: {version}"),
            Self::UnknownDatatype(code) => write!(f, "Unknown datatype code: {code}"),
            Self::InvalidTimestamp(timestamp) => write!(f, "Invalid unix timestamp: {timestamp}"),
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

#[derive(Debug)]
pub struct SanityError(pub String);

pub fn process_data(streams: Vec<LogStream>, config: &Config) -> PlotData {
    let streams = streams.into();
    let plots = config
        .tabs
        .iter()
        .map(|t| {
            t.plots
                .iter()
                .map(|p| PlotValues::Result(eval::eval(&p.expr, Arc::clone(&streams))))
                .collect()
        })
        .collect();
    PlotData { streams, plots }
}
