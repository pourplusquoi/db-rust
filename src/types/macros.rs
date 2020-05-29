macro_rules! value {
    ($x:expr, $variant:ident) => {
        Value::new(Types::$variant($x))
    };
}

// Unwrap or return.
macro_rules! unwrapor {
    ($x:expr) => {
        match $x {
            Ok(r) => r,
            Err(e) => return Err(e).log_and().ok(),
        }
    };
}

macro_rules! arithmetic_tinyint {
    ($x:ident, $y:ident, $closure:tt) => {{
        let res = match $y.content {
            Types::TinyInt(rhs) => value!($closure($x, rhs)?, TinyInt),
            Types::SmallInt(rhs) => value!($closure($x as i16, rhs)?, SmallInt),
            Types::Integer(rhs) => value!($closure($x as i32, rhs)?, Integer),
            Types::BigInt(rhs) => value!($closure($x as i64, rhs)?, BigInt),
            Types::Decimal(rhs) => value!($closure($x as f64, rhs)?, Decimal),
            _ => {
                let mut rhs = Value::new(Types::tinyint());
                $y.cast_to(&mut rhs)?;
                value!($closure($x, rhs.get_as_i8()?)?, TinyInt)
            }
        };
        Ok(res)
    }};
}

macro_rules! compare_tinyint {
    ($x:ident, $y:ident, $closure1:tt, $closure2:tt) => {{
        let res = match $y.content {
            Types::TinyInt(rhs) => $closure1($x, rhs),
            Types::SmallInt(rhs) => $closure1($x as i16, rhs),
            Types::Integer(rhs) => $closure1($x as i32, rhs),
            Types::BigInt(rhs) => $closure1($x as i64, rhs),
            Types::Decimal(rhs) => $closure2($x as f64 - rhs),
            _ => {
                let mut rhs = Value::new(Types::tinyint());
                unwrapor!($y.cast_to(&mut rhs));
                $closure1($x, unwrapor!(rhs.get_as_i8()))
            }
        };
        Ok(res) as Result<_, Error>
    }};
}

macro_rules! arithmetic_smallint {
    ($x:ident, $y:ident, $closure:tt) => {{
        let res = match $y.content {
            Types::TinyInt(rhs) => value!($closure($x, rhs as i16)?, SmallInt),
            Types::SmallInt(rhs) => value!($closure($x, rhs)?, SmallInt),
            Types::Integer(rhs) => value!($closure($x as i32, rhs)?, Integer),
            Types::BigInt(rhs) => value!($closure($x as i64, rhs)?, BigInt),
            Types::Decimal(rhs) => value!($closure($x as f64, rhs)?, Decimal),
            _ => {
                let mut rhs = Value::new(Types::smallint());
                $y.cast_to(&mut rhs)?;
                value!($closure($x, rhs.get_as_i16()?)?, SmallInt)
            }
        };
        Ok(res)
    }};
}

macro_rules! compare_smallint {
    ($x:ident, $y:ident, $closure1:tt, $closure2:tt) => {{
        let res = match $y.content {
            Types::TinyInt(rhs) => $closure1($x, rhs as i16),
            Types::SmallInt(rhs) => $closure1($x, rhs),
            Types::Integer(rhs) => $closure1($x as i32, rhs),
            Types::BigInt(rhs) => $closure1($x as i64, rhs),
            Types::Decimal(rhs) => $closure2($x as f64 - rhs),
            _ => {
                let mut rhs = Value::new(Types::smallint());
                unwrapor!($y.cast_to(&mut rhs));
                $closure1($x, unwrapor!(rhs.get_as_i16()))
            }
        };
        Ok(res) as Result<_, Error>
    }};
}

macro_rules! arithmetic_integer {
    ($x:ident, $y:ident, $closure:tt) => {{
        let res = match $y.content {
            Types::TinyInt(rhs) => value!($closure($x, rhs as i32)?, Integer),
            Types::SmallInt(rhs) => value!($closure($x, rhs as i32)?, Integer),
            Types::Integer(rhs) => value!($closure($x, rhs)?, Integer),
            Types::BigInt(rhs) => value!($closure($x as i64, rhs)?, BigInt),
            Types::Decimal(rhs) => value!($closure($x as f64, rhs)?, Decimal),
            _ => {
                let mut rhs = Value::new(Types::integer());
                $y.cast_to(&mut rhs)?;
                value!($closure($x, rhs.get_as_i32()?)?, Integer)
            }
        };
        Ok(res)
    }};
}

macro_rules! compare_integer {
    ($x:ident, $y:ident, $closure1:tt, $closure2:tt) => {{
        let res = match $y.content {
            Types::TinyInt(rhs) => $closure1($x, rhs as i32),
            Types::SmallInt(rhs) => $closure1($x, rhs as i32),
            Types::Integer(rhs) => $closure1($x, rhs),
            Types::BigInt(rhs) => $closure1($x as i64, rhs),
            Types::Decimal(rhs) => $closure2($x as f64 - rhs),
            _ => {
                let mut rhs = Value::new(Types::integer());
                unwrapor!($y.cast_to(&mut rhs));
                $closure1($x, unwrapor!(rhs.get_as_i32()))
            }
        };
        Ok(res) as Result<_, Error>
    }};
}

macro_rules! arithmetic_bigint {
    ($x:ident, $y:ident, $closure:tt) => {{
        let res = match $y.content {
            Types::TinyInt(rhs) => value!($closure($x, rhs as i64)?, BigInt),
            Types::SmallInt(rhs) => value!($closure($x, rhs as i64)?, BigInt),
            Types::Integer(rhs) => value!($closure($x, rhs as i64)?, BigInt),
            Types::BigInt(rhs) => value!($closure($x, rhs)?, BigInt),
            Types::Decimal(rhs) => value!($closure($x as f64, rhs)?, Decimal),
            _ => {
                let mut rhs = Value::new(Types::bigint());
                $y.cast_to(&mut rhs)?;
                value!($closure($x, rhs.get_as_i64()?)?, BigInt)
            }
        };
        Ok(res)
    }};
}

macro_rules! compare_bigint {
    ($x:ident, $y:ident, $closure1:tt, $closure2:tt) => {{
        let res = match $y.content {
            Types::TinyInt(rhs) => $closure1($x, rhs as i64),
            Types::SmallInt(rhs) => $closure1($x, rhs as i64),
            Types::Integer(rhs) => $closure1($x, rhs as i64),
            Types::BigInt(rhs) => $closure1($x, rhs),
            Types::Decimal(rhs) => $closure2($x as f64 - rhs),
            _ => {
                let mut rhs = Value::new(Types::bigint());
                unwrapor!($y.cast_to(&mut rhs));
                $closure1($x, unwrapor!(rhs.get_as_i64()))
            }
        };
        Ok(res) as Result<_, Error>
    }};
}

macro_rules! arithmetic_decimal {
    ($x:ident, $y:ident, $closure:tt) => {{
        let res = match $y.content {
            Types::TinyInt(rhs) => value!($closure($x, rhs as f64)?, Decimal),
            Types::SmallInt(rhs) => value!($closure($x, rhs as f64)?, Decimal),
            Types::Integer(rhs) => value!($closure($x, rhs as f64)?, Decimal),
            Types::BigInt(rhs) => value!($closure($x, rhs as f64)?, Decimal),
            Types::Decimal(rhs) => value!($closure($x, rhs)?, Decimal),
            _ => {
                let mut rhs = Value::new(Types::decimal());
                $y.cast_to(&mut rhs)?;
                value!($closure($x, rhs.get_as_f64()?)?, Decimal)
            }
        };
        Ok(res)
    }};
}

macro_rules! compare_decimal {
    ($x:ident, $y:ident, $closure:tt) => {{
        let res = match $y.content {
            Types::TinyInt(rhs) => $closure($x - rhs as f64),
            Types::SmallInt(rhs) => $closure($x - rhs as f64),
            Types::Integer(rhs) => $closure($x - rhs as f64),
            Types::BigInt(rhs) => $closure($x - rhs as f64),
            Types::Decimal(rhs) => $closure($x - rhs),
            _ => {
                let mut rhs = Value::new(Types::decimal());
                unwrapor!($y.cast_to(&mut rhs));
                $closure($x - unwrapor!(rhs.get_as_f64()))
            }
        };
        Ok(res) as Result<_, Error>
    }};
}

macro_rules! compare_bool {
    ($x:ident, $y:ident, $closure:tt) => {{
        let mut rhs = Value::new(Types::boolean());
        unwrapor!($y.cast_to(&mut rhs));
        Ok($closure($x, unwrapor!(rhs.get_as_bool()))) as Result<_, Error>
    }};
}

macro_rules! compare_timestamp {
    ($x:ident, $y:ident, $closure:tt) => {{
        Ok($closure($x, unwrapor!($y.get_as_u64()))) as Result<_, Error>
    }};
}

macro_rules! compare_varchar {
    ($x:ident, $y:ident, $closure:tt) => {{
        match $y.content {
            Types::Varchar(ref rhs) => Ok($closure(varlen_cmp($x, rhs), 0)),
            _ => {
                let mut rhs = Value::new(Types::owned());
                unwrapor!($y.cast_to(&mut rhs));
                match varlen_value_cmp($x, &rhs) {
                    Ok(r) => Ok($closure(r, 0)),
                    Err(e) => Err(e),
                }
            }
        }
    }};
}

macro_rules! compare {
    ($x:ident, $y:ident, $closure1:tt, $closure2:tt) => {{
        unwrapor!(assert_comparable($x, $y));
        if $x.is_null() || $y.is_null() {
            None
        } else {
            match $x.content {
                Types::Boolean(lhs) => compare_bool!(lhs, $y, $closure1).log_and().ok(),
                Types::TinyInt(lhs) => compare_tinyint!(lhs, $y, $closure1, $closure2)
                    .log_and()
                    .ok(),
                Types::SmallInt(lhs) => compare_smallint!(lhs, $y, $closure1, $closure2)
                    .log_and()
                    .ok(),
                Types::Integer(lhs) => compare_integer!(lhs, $y, $closure1, $closure2)
                    .log_and()
                    .ok(),
                Types::BigInt(lhs) => compare_bigint!(lhs, $y, $closure1, $closure2)
                    .log_and()
                    .ok(),
                Types::Timestamp(lhs) => compare_timestamp!(lhs, $y, $closure1).log_and().ok(),
                Types::Decimal(lhs) => compare_decimal!(lhs, $y, $closure2).log_and().ok(),
                Types::Varchar(ref lhs) => compare_varchar!(lhs, $y, $closure1).log_and().ok(),
            }
        }
    }};
}

macro_rules! arithmetic {
    ($x:ident, $y:ident, $closure:tt) => {{
        assert_numeric($x)?;
        assert_comparable($x, $y)?;
        if $x.is_null() || $y.is_null() {
            $x.null($y)
        } else {
            match $x.content {
                Types::TinyInt(lhs) => arithmetic_tinyint!(lhs, $y, $closure),
                Types::SmallInt(lhs) => arithmetic_smallint!(lhs, $y, $closure),
                Types::Integer(lhs) => arithmetic_integer!(lhs, $y, $closure),
                Types::BigInt(lhs) => arithmetic_bigint!(lhs, $y, $closure),
                Types::Decimal(lhs) => arithmetic_decimal!(lhs, $y, $closure),
                _ => Err(Error::new(
                    ErrorKind::NotSupported,
                    "Invalid type for `arithmetic`",
                )),
            }
        }
    }};
}

macro_rules! castnum {
    ($x:expr, $y:ident, $z:tt, $w:expr) => {{
        match &mut $x {
            Types::TinyInt(dst) => *dst = $z($y)?,
            Types::SmallInt(dst) => *dst = $z($y)?,
            Types::Integer(dst) => *dst = $z($y)?,
            Types::BigInt(dst) => *dst = $z($y)?,
            Types::Decimal(dst) => *dst = $z($y)?,
            Types::Varchar(dst) => *dst = Varlen::Owned(Str::Val($y.to_string())),
            _ => Err(Error::new(
                ErrorKind::CannotCast,
                &*format!("Cannot cast {} to given type", $w),
            ))?,
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
        Ok(Value::new($x.content.clone().null_val()?))
    }};
}

macro_rules! string {
    ($x:ident, $y:expr) => {{
        if $x.is_null() {
            $y.to_string()
        } else {
            $x.to_string()
        }
    }};
}

macro_rules! unsupported {
    ($x:expr) => {
        Error::new(ErrorKind::NotSupported, $x)
    };
}

macro_rules! primitive_from_impl {
    ($x:ty, $y:ty) => {
        impl PrimitiveFrom<$x> for $y {
            fn from(val: &$x) -> $y {
                *val as $y
            }
        }
    };
}

macro_rules! parse_into_impl {
    ($x:ty) => {
        impl ParseInto<$x> for &str {
            fn into(self) -> Result<$x, Error> {
                self.parse::<$x>()
                    .map_err(|_| Error::new(ErrorKind::CannotParse, "Parse failure"))
            }
        }
    };
}

macro_rules! limits_impl {
    ($x:ty, $min:expr, $max:expr) => {
        impl HasLimits for $x {
            fn min() -> Self {
                $min
            }
            fn max() -> Self {
                $max
            }
        }
    };
}

macro_rules! arithmetic_impl {
    ($x:ty) => {
        impl Arithmetic for $x {
            fn modulo(&self, other: &Self) -> Self {
                *self % *other
            }
            fn zero() -> Self {
                0 as $x
            }
        }
    };
}

macro_rules! genmatch {
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
    fn genmatch_macro() {
        let value = Types::Integer(42);
        assert_eq!(None, genmatch!(value, None, {[TinyInt, SmallInt], Some(3)}));
        assert_eq!(
            Some(3),
            genmatch!(value, None, {[TinyInt, SmallInt, Integer, BigInt], Some(3)})
        );
        assert_eq!(
            Some(3),
            genmatch!(
                value, None,
                {[TinyInt, Integer], Some(3)}, {[SmallInt, BigInt], Some(5)})
        );
        assert_eq!(
            Some(5),
            genmatch!(
                value, None,
                {[TinyInt, SmallInt, BigInt, Decimal], Some(3)}, {[Integer], Some(5)})
        );
        assert_eq!(
            Some(5),
            genmatch!(
                value, None,
                {[TinyInt, SmallInt, BigInt], Some(3)},
                {[Integer, Decimal, Timestamp], Some(5)})
        );
        assert_eq!(
            Some(7),
            genmatch!(
                value, None,
                {[TinyInt], Some(3)}, {[SmallInt], Some(5)},
                {[Integer], Some(7)}, {[BigInt], Some(9)})
        );
    }
}
