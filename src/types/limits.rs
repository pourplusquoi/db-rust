#![allow(dead_code)]

pub const DBL_MIN: f64 = std::f64::MIN;
pub const DBL_MAX: f64 = std::f64::MAX;
pub const FLT_MIN: f32 = std::f32::MIN;
pub const FLT_MAX: f32 = std::f32::MAX;

pub const PELOTON_INT8_MIN: i8 = std::i8::MIN + 1;
pub const PELOTON_INT16_MIN: i16 = std::i16::MIN + 1;
pub const PELOTON_INT32_MIN: i32 = std::i32::MIN + 1;
pub const PELOTON_INT64_MIN: i64 = std::i64::MIN + 1;
pub const PELOTON_DECIMAL_MIN: f64 = FLT_MIN as f64;
pub const PELOTON_TIMESTAMP_MIN: u64 = 0;
pub const PELOTON_DATE_MIN: u32 = 0;
pub const PELOTON_BOOLEAN_MIN: i8 = 0;

pub const PELOTON_INT8_MAX: i8 = std::i8::MAX;
pub const PELOTON_INT16_MAX: i16 = std::i16::MAX;
pub const PELOTON_INT32_MAX: i32 = std::i32::MAX;
pub const PELOTON_INT64_MAX: i64 = std::i64::MAX;
pub const PELOTON_UINT64_MAX: u64 = std::u64::MAX - 1;
pub const PELOTON_DECIMAL_MAX: f64 = DBL_MAX;
pub const PELOTON_TIMESTAMP_MAX: u64 = 11231999986399999999;
pub const PELOTON_DATE_MAX: u64 = std::i32::MAX as u64;
pub const PELOTON_BOOLEAN_MAX: i8 = 1;

pub const PELOTON_VALUE_NULL: u32 = std::u32::MAX;
pub const PELOTON_INT8_NULL: i8 = std::i8::MIN;
pub const PELOTON_INT16_NULL: i16 = std::i16::MIN;
pub const PELOTON_INT32_NULL: i32 = std::i32::MIN;
pub const PELOTON_INT64_NULL: i64 = std::i64::MIN;
pub const PELOTON_DECIMAL_NULL: f64 = DBL_MIN;
pub const PELOTON_TIMESTAMP_NULL: u64 = std::u64::MAX;
pub const PELOTON_DATE_NULL: u64 = 0;
pub const PELOTON_BOOLEAN_NULL: i8 = i8::MIN;

pub const PELOTON_VARCHAR_MAX_LEN: u32 = u32::MAX;

// Use to make TEXT type as the alias of VARCHAR(TEXT_MAX_LENGTH)
const PELOTON_TEXT_MAX_LEN: u32 = 1000000000;

// // Objects (i.e., VARCHAR) with length prefix of -1 are NULL
// #define OBJECTLENGTH_NULL -1
