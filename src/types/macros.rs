#[macro_export]
macro_rules! compare {
    ($x:ident, $y:ident, $closure1:tt, $closure2:tt) => {{
        Some(match $x.content {
            Types::Boolean(_) => $closure1($x.get_as_bool(), $y.get_as_bool()),
            Types::TinyInt(_) => $closure1($x.get_as_i8(), $y.get_as_i8()),
            Types::SmallInt(_) => $closure1($x.get_as_i16(), $y.get_as_i16()),
            Types::Integer(_) => $closure1($x.get_as_i32(), $y.get_as_i32()),
            Types::BigInt(_) => $closure1($x.get_as_i64(), $y.get_as_i64()),
            Types::Timestamp(_) => $closure1($x.get_as_u64(), $y.get_as_u64()),
            Types::Decimal(_) => $closure2($x.subtract($y)),
            Types::Varchar(ref varlen) => $closure1(varlen_value_cmp(varlen, $y), 0),
        })
    }};
}
