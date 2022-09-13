use std::io::{self, Read, Seek, SeekFrom};

use super::Error;

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

impl EntryKind {
    fn size(&self) -> u8 {
        match self {
            Self::Bool(_) => 1,
            Self::U8(_) => 1,
            Self::U16(_) => 2,
            Self::U32(_) => 4,
            Self::U64(_) => 8,
            Self::I8(_) => 1,
            Self::I16(_) => 2,
            Self::I32(_) => 4,
            Self::I64(_) => 8,
            Self::F32(_) => 4,
            Self::F64(_) => 8,
        }
    }
}

impl TryFrom<u8> for EntryKind {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let data_type = match value {
            0 => Self::Bool(Vec::new()),
            1 => Self::U8(Vec::new()),
            2 => Self::U16(Vec::new()),
            3 => Self::U32(Vec::new()),
            4 => Self::U64(Vec::new()),
            5 => Self::I8(Vec::new()),
            6 => Self::I16(Vec::new()),
            7 => Self::I32(Vec::new()),
            8 => Self::I64(Vec::new()),
            9 => Self::F32(Vec::new()),
            10 => Self::F64(Vec::new()),
            _ => return Err(Error::UnknownDatatype(value)),
        };
        Ok(data_type)
    }
}

struct BoolContext {
    bit_fields: u8,
    mask: u8,
}

fn read_data(reader: &mut (impl Read + Seek)) -> Result<Vec<DataEntry>, Error> {
    let mut stream_len = reader.len()?;

    let magic = read_string(reader, 4)?;
    if magic != "s3lg" {
        return Err(Error::InvalidMagic(magic));
    }

    let num_entries = read_u16(reader)?;
    let entries = Vec::with_capacity(num_entries as usize);

    let mut pos = 6;

    for _ in 0..num_entries {
        let code = read_u8(reader)?;
        let kind = EntryKind::try_from(code)?;
        let name_len = read_u8(reader)? as usize;
        let name = read_string(reader, name_len)?;

        entries.push(DataEntry { name, kind });

        pos += 2 + name_len;
    }

    let mut bool_ctx = None;
    while pos < stream_len {
        let mut read_bool = false;

        for e in entries.iter_mut() {
            let mut read_bytes = e.kind.size();

            match &mut e.kind {
                EntryKind::Bool(v) => {
                    read_bool = true;
                    read_bytes = 0;

                    let ctx = bool_ctx.get_or_insert_with(|| {
                        read_bytes = 1;
                        BoolContext {
                            bit_fields: read_u8(reader)?,
                            mask: 1,
                        }
                    });

                    let masked = ctx.bit_fields & ctx.mask;
                    v.push(masked != 0);

                    if ctx.mask >= 0x80 {
                        bool_ctx = None;
                    } else {
                        ctx.mask <<= 1;
                    }
                }
                EntryKind::U8(v) => v.push(read_u8(reader)?),
                EntryKind::U16(v) => v.push(read_u16(reader)?),
                EntryKind::U32(v) => v.push(read_u32(reader)?),
                EntryKind::U64(v) => v.push(read_u64(reader)?),
                EntryKind::I8(v) => v.push(read_i8(reader)?),
                EntryKind::I16(v) => v.push(read_i16(reader)?),
                EntryKind::I32(v) => v.push(read_i32(reader)?),
                EntryKind::I64(v) => v.push(read_i64(reader)?),
                EntryKind::F32(v) => v.push(read_f32(reader)?),
                EntryKind::F64(v) => v.push(read_f64(reader)?),
            }

            pos += read_bytes;
        }

        if !read_bool {
            bool_ctx = None;
        }
    }

    Ok(entries)
}

fn read_entries(reader: &mut (impl Read + Seek), entries: &mut Vec<DataEntry>) {
    let len = reader.len();
}

impl<T: Seek> SeekUtils for T {}
pub trait SeekUtils: Seek {
    fn len(&mut self) -> io::Result<u64> {
        let pos = self.seek(SeekFrom::Current(0))?;
        let len = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(pos))?;
        Ok(len)
    }
}

macro_rules! impl_read_num {
    ($ident:ident, $ty:ty) => {
        fn $ident(reader: &mut impl Read) -> Result<$ty, Error> {
            let mut buf = [0; std::mem::size_of::<$ty>()];
            reader.read_exact(&mut buf)?;
            Ok(<$ty>::from_be_bytes(buf))
        }
    };
}
impl_read_num!(read_u8, u8);
impl_read_num!(read_u16, u16);
impl_read_num!(read_u32, u32);
impl_read_num!(read_u64, u64);
impl_read_num!(read_i8, i8);
impl_read_num!(read_i16, i16);
impl_read_num!(read_i32, i32);
impl_read_num!(read_i64, i64);
impl_read_num!(read_f32, f32);
impl_read_num!(read_f64, f64);

fn read_string(reader: &mut impl Read, len: usize) -> Result<String, Error> {
    let mut buf = vec![0; len];
    reader.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}
