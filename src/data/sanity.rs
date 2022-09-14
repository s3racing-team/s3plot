use super::{SanityError, DataEntry, EntryKind};

pub fn sanity_check(entries: &[DataEntry]) -> Result<(), SanityError> {
    for e in entries {
        let r = match &e.kind {
            EntryKind::Bool(v) => Ok(()),
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
        };

        r?;
    }
    Ok(())
}

fn check_all<T>(
    values: &[T],
    name: &str,
    check: impl Fn(T, &str) -> Result<(), SanityError>,
) -> Result<(), SanityError> {
    for entry in values {
        check(*entry, name)?;
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
