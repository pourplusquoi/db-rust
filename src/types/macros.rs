macro_rules! value {
    ($x:expr, $variant:ident) => {
        Value::new(Types::$variant($x))
    };
}

macro_rules! arithmetic_tinyint {
    ($x:ident, $y:ident, $closure:tt) => {{
        match $y.content {
            Types::TinyInt(rhs) => value!($closure($x, rhs), TinyInt),
            Types::SmallInt(rhs) => value!($closure($x as i16, rhs), SmallInt),
            Types::Integer(rhs) => value!($closure($x as i32, rhs), Integer),
            Types::BigInt(rhs) => value!($closure($x as i64, rhs), BigInt),
            Types::Decimal(rhs) => value!($closure($x as f64, rhs), Decimal),
            _ => {
                let mut rhs = Value::new(Types::tinyint());
                $y.cast_to(&mut rhs);
                value!($closure($x, rhs.get_as_i8()), TinyInt)
            }
        }
    }};
}

macro_rules! compare_tinyint {
    ($x:ident, $y:ident, $closure1:tt, $closure2:tt) => {{
        match $y.content {
            Types::TinyInt(rhs) => $closure1($x, rhs),
            Types::SmallInt(rhs) => $closure1($x as i16, rhs),
            Types::Integer(rhs) => $closure1($x as i32, rhs),
            Types::BigInt(rhs) => $closure1($x as i64, rhs),
            Types::Decimal(rhs) => $closure2($x as f64 - rhs),
            _ => {
                let mut rhs = Value::new(Types::tinyint());
                $y.cast_to(&mut rhs);
                $closure1($x, rhs.get_as_i8())
            }
        }
    }};
}

macro_rules! arithmetic_smallint {
    ($x:ident, $y:ident, $closure:tt) => {{
        match $y.content {
            Types::TinyInt(rhs) => value!($closure($x, rhs as i16), SmallInt),
            Types::SmallInt(rhs) => value!($closure($x, rhs), SmallInt),
            Types::Integer(rhs) => value!($closure($x as i32, rhs), Integer),
            Types::BigInt(rhs) => value!($closure($x as i64, rhs), BigInt),
            Types::Decimal(rhs) => value!($closure($x as f64, rhs), Decimal),
            _ => {
                let mut rhs = Value::new(Types::smallint());
                $y.cast_to(&mut rhs);
                value!($closure($x, rhs.get_as_i16()), SmallInt)
            }
        }
    }};
}

macro_rules! compare_smallint {
    ($x:ident, $y:ident, $closure1:tt, $closure2:tt) => {{
        match $y.content {
            Types::TinyInt(rhs) => $closure1($x, rhs as i16),
            Types::SmallInt(rhs) => $closure1($x, rhs),
            Types::Integer(rhs) => $closure1($x as i32, rhs),
            Types::BigInt(rhs) => $closure1($x as i64, rhs),
            Types::Decimal(rhs) => $closure2($x as f64 - rhs),
            _ => {
                let mut rhs = Value::new(Types::smallint());
                $y.cast_to(&mut rhs);
                $closure1($x, rhs.get_as_i16())
            }
        }
    }};
}

macro_rules! arithmetic_integer {
    ($x:ident, $y:ident, $closure:tt) => {{
        match $y.content {
            Types::TinyInt(rhs) => value!($closure($x, rhs as i32), Integer),
            Types::SmallInt(rhs) => value!($closure($x, rhs as i32), Integer),
            Types::Integer(rhs) => value!($closure($x, rhs), Integer),
            Types::BigInt(rhs) => value!($closure($x as i64, rhs), BigInt),
            Types::Decimal(rhs) => value!($closure($x as f64, rhs), Decimal),
            _ => {
                let mut rhs = Value::new(Types::integer());
                $y.cast_to(&mut rhs);
                value!($closure($x, rhs.get_as_i32()), Integer)
            }
        }
    }};
}

macro_rules! compare_integer {
    ($x:ident, $y:ident, $closure1:tt, $closure2:tt) => {{
        match $y.content {
            Types::TinyInt(rhs) => $closure1($x, rhs as i32),
            Types::SmallInt(rhs) => $closure1($x, rhs as i32),
            Types::Integer(rhs) => $closure1($x, rhs),
            Types::BigInt(rhs) => $closure1($x as i64, rhs),
            Types::Decimal(rhs) => $closure2($x as f64 - rhs),
            _ => {
                let mut rhs = Value::new(Types::integer());
                $y.cast_to(&mut rhs);
                $closure1($x, rhs.get_as_i32())
            }
        }
    }};
}

macro_rules! arithmetic_bigint {
    ($x:ident, $y:ident, $closure:tt) => {{
        match $y.content {
            Types::TinyInt(rhs) => value!($closure($x, rhs as i64), BigInt),
            Types::SmallInt(rhs) => value!($closure($x, rhs as i64), BigInt),
            Types::Integer(rhs) => value!($closure($x, rhs as i64), BigInt),
            Types::BigInt(rhs) => value!($closure($x, rhs), BigInt),
            Types::Decimal(rhs) => value!($closure($x as f64, rhs), Decimal),
            _ => {
                let mut rhs = Value::new(Types::bigint());
                $y.cast_to(&mut rhs);
                value!($closure($x, rhs.get_as_i64()), BigInt)
            }
        }
    }};
}

macro_rules! compare_bigint {
    ($x:ident, $y:ident, $closure1:tt, $closure2:tt) => {{
        match $y.content {
            Types::TinyInt(rhs) => $closure1($x, rhs as i64),
            Types::SmallInt(rhs) => $closure1($x, rhs as i64),
            Types::Integer(rhs) => $closure1($x, rhs as i64),
            Types::BigInt(rhs) => $closure1($x, rhs),
            Types::Decimal(rhs) => $closure2($x as f64 - rhs),
            _ => {
                let mut rhs = Value::new(Types::bigint());
                $y.cast_to(&mut rhs);
                $closure1($x, rhs.get_as_i64())
            }
        }
    }};
}

macro_rules! arithmetic_decimal {
    ($x:ident, $y:ident, $closure:tt) => {{
        match $y.content {
            Types::TinyInt(rhs) => value!($closure($x, rhs as f64), Decimal),
            Types::SmallInt(rhs) => value!($closure($x, rhs as f64), Decimal),
            Types::Integer(rhs) => value!($closure($x, rhs as f64), Decimal),
            Types::BigInt(rhs) => value!($closure($x, rhs as f64), Decimal),
            Types::Decimal(rhs) => value!($closure($x, rhs), Decimal),
            _ => {
                let mut rhs = Value::new(Types::decimal());
                $y.cast_to(&mut rhs);
                value!($closure($x, rhs.get_as_f64()), Decimal)
            }
        }
    }};
}

macro_rules! compare_decimal {
    ($x:ident, $y:ident, $closure:tt) => {{
        match $y.content {
            Types::TinyInt(rhs) => $closure($x - rhs as f64),
            Types::SmallInt(rhs) => $closure($x - rhs as f64),
            Types::Integer(rhs) => $closure($x - rhs as f64),
            Types::BigInt(rhs) => $closure($x - rhs as f64),
            Types::Decimal(rhs) => $closure($x - rhs),
            _ => {
                let mut rhs = Value::new(Types::decimal());
                $y.cast_to(&mut rhs);
                $closure($x - rhs.get_as_f64())
            }
        }
    }};
}

macro_rules! compare_varchar {
    ($x:ident, $y:ident, $closure:tt) => {{
        match $y.content {
            Types::Varchar(ref rhs) => $closure(varlen_cmp($x, rhs), 0),
            _ => {
                let mut rhs = Value::new(Types::owned());
                $y.cast_to(&mut rhs);
                $closure(varlen_value_cmp($x, &rhs), 0)
            }
        }
    }};
}

macro_rules! compare {
    ($x:ident, $y:ident, $closure1:tt, $closure2:tt) => {{
        assert_comparable($x, $y);
        if $x.is_null() || $y.is_null() {
            None
        } else {
            Some(match $x.content {
                Types::Boolean(lhs) => {
                    let mut rhs = Value::new(Types::boolean());
                    $y.cast_to(&mut rhs);
                    $closure1(lhs, rhs.get_as_bool())
                }
                Types::TinyInt(lhs) => compare_tinyint!(lhs, $y, $closure1, $closure2),
                Types::SmallInt(lhs) => compare_smallint!(lhs, $y, $closure1, $closure2),
                Types::Integer(lhs) => compare_integer!(lhs, $y, $closure1, $closure2),
                Types::BigInt(lhs) => compare_bigint!(lhs, $y, $closure1, $closure2),
                Types::Timestamp(lhs) => $closure1(lhs, $y.get_as_u64()),
                Types::Decimal(lhs) => compare_decimal!(lhs, $y, $closure2),
                Types::Varchar(ref lhs) => compare_varchar!(lhs, $y, $closure1),
            })
        }
    }};
}

macro_rules! arithmetic {
    ($x:ident, $y:ident, $closure:tt) => {{
        assert_numeric($x);
        assert_comparable($x, $y);
        if $x.is_null() || $y.is_null() {
            $x.null($y)
        } else {
            match $x.content {
                Types::TinyInt(lhs) => arithmetic_tinyint!(lhs, $y, $closure),
                Types::SmallInt(lhs) => arithmetic_smallint!(lhs, $y, $closure),
                Types::Integer(lhs) => arithmetic_integer!(lhs, $y, $closure),
                Types::BigInt(lhs) => arithmetic_bigint!(lhs, $y, $closure),
                Types::Decimal(lhs) => arithmetic_decimal!(lhs, $y, $closure),
                _ => panic!("Type error for arithmetic"),
            }
        }
    }};
}

macro_rules! forward {
    ($x:ident, $y:ident, $z:ty) => {
        fn $y(&self) -> $z {
            self.$x.$y()
        }
    };
}

macro_rules! nullas {
    ($x:ident) => {{
        Value::new($x.content.clone().null_val())
    }};
}

macro_rules! generate_match {
    ($x:expr, $default:expr, $( { [$( $variant:ident ),*], $val:expr } ),*) => {{
        match $x {
            $( $( Types::$variant(_) )|* => $val, )*
            _ => $default,
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::types::types::Types;

    #[test]
    fn generate_match_macro() {
        let value = Types::Integer(42);
        assert_eq!(
            None,
            generate_match!(value, None, {[TinyInt, SmallInt], Some(3)})
        );
        assert_eq!(
            Some(3),
            generate_match!(value, None, {[TinyInt, SmallInt, Integer, BigInt], Some(3)})
        );
        assert_eq!(
            Some(3),
            generate_match!(
                value, None,
                {[TinyInt, Integer], Some(3)}, {[SmallInt, BigInt], Some(5)})
        );
        assert_eq!(
            Some(5),
            generate_match!(
                value, None,
                {[TinyInt, SmallInt, BigInt, Decimal], Some(3)}, {[Integer], Some(5)})
        );
        assert_eq!(
            Some(5),
            generate_match!(
                value, None,
                {[TinyInt, SmallInt, BigInt], Some(3)},
                {[Integer, Decimal, Timestamp], Some(5)})
        );
        assert_eq!(
            Some(7),
            generate_match!(
                value, None,
                {[TinyInt], Some(3)}, {[SmallInt], Some(5)},
                {[Integer], Some(7)}, {[BigInt], Some(9)})
        );
    }
}
