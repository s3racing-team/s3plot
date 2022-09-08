use std::io::{self, Read, Seek, SeekFrom};

use super::Error;

pub struct DataEntry {
    name: String,
    kind: EntryType,
}

#[derive(Clone, Debug)]
pub enum EntryType {
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

impl TryFrom<u8> for EntryType {
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

fn read_header(reader: &mut impl Read) -> Result<Vec<DataEntry>, Error> {
    let magic = read_string(reader, 4)?;
    if magic != "s3lg" {
        return Err(Error::InvalidMagic(magic));
    }

    let num_entries = read_u16(reader)?;
    let entries = Vec::with_capacity(num_entries as usize);

    for _ in 0..num_entries {
        let code = read_u8(reader)?;
        let kind = EntryType::try_from(code)?;
        let name_len = read_u8(reader)?;
        let name = read_string(reader, name_len as usize)?;

        entries.push(DataEntry { name, kind })
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

macro_rules! impl_sanity_check_int {
    ($ident:ident, $ty:ty) => {
        fn $ident(val: $ty) -> Result<(), Error> {
            if val == <$ty>::MAX {
                return Err(Error::SanityCheck("Value is max"));
            }
            if val == <$ty>::MIN {
                return Err(Error::SanityCheck("Value is min"));
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
        fn $ident(val: $ty) -> Result<(), Error> {
            if val.is_nan() {
                return Err(Error::SanityCheck("Value is nan"));
            }
            if val.is_infinite() {
                return Err(Error::SanityCheck("Value is infinite"));
            }
            Ok(())
        }
    };
}
Impl_sanity_check_float!(sanity_check_f32, f32);
Impl_sanity_check_float!(sanity_check_f64, f64);
