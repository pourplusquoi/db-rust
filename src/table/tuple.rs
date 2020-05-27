
#![allow(dead_code)]

use crate::catalog::schema::Schema;
use crate::types::types::Types;
use crate::types::value::Value;

struct Tuple {
    allocated: bool,
    // rid: RID,
    size: usize,
    data: [u8],
}

impl Tuple {
    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_allocated(&self) -> bool {
        self.allocated
    }

    pub fn serialize_to(&self, dst: &mut [u8]) {}

    pub fn deserialize_from(&mut self, src: &[u8]) {}

    // TODO
    pub fn value(&self, schema: &Schema, idx: usize) -> Value {
        Value::new(Types::Integer(0))
    }

    pub fn is_null(&self, schema: &Schema, idx: usize) -> bool {
        self.value(schema, idx).is_null()
    }

    // TODO
    pub fn to_string(&self) -> String {
        "".to_string()
    }
}