use crate::catalog::schema::Schema;
use crate::common::reinterpret;
use crate::types::types::Operation;
use crate::types::value::Value;
use std::clone::Clone;
use std::cmp::PartialEq;
use std::default::Default;
use std::fmt::Debug;
use std::mem;

#[derive(Clone, Debug, PartialEq)]
pub struct Tuple {
    // rid: RID,
    data: Vec<u8>,
}

impl Default for Tuple {
    fn default() -> Self {
        Tuple { data: Vec::new() }
    }
}

impl Tuple {
    // The caller needs to ensure that |values| and |schema.columns| have the same size.
    pub fn new(values: &Vec<Value>, schema: &Schema) -> Self {
        // Step1: Calculate size of the tuple.
        let mut size = schema.len();
        for &idx in schema.uninlined().iter() {
            size += values[idx].len() + mem::size_of::<u64>();
        }
        let mut tuple = Tuple {
            data: vec![0; size],
        };
        let ptr = tuple.data.as_mut_slice();

        // Step2: Serialize each column (attribute) based on input value.
        let mut str_offset = schema.len();
        for idx in 0..schema.columns().len() {
            let nth_offset = schema.nth_offset(idx).unwrap();
            if !schema.nth_is_inlined(idx).unwrap() {
                reinterpret::write_u64(&mut ptr[nth_offset..], str_offset as u64);
                values[idx].serialize_to(&mut ptr[str_offset..]);
                str_offset += values[idx].len() + mem::size_of::<u64>();
            } else {
                values[idx].serialize_to(&mut ptr[nth_offset..]);
            }
        }
        tuple
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    // The caller needs to make sure that |dst| has enough space.
    pub fn serialize_to(&self, dst: &mut [u8]) {
        let size = self.data.len() as u64;
        reinterpret::write_u64(dst, size);
        for (s, d) in self
            .data
            .iter()
            .zip(dst.iter_mut().skip(mem::size_of::<u64>()))
        {
            *d = *s;
        }
    }

    // The caller needs to make sure that |src| is valid.
    pub fn deserialize_from(&mut self, src: &[u8]) {
        let size = reinterpret::read_u64(src) as usize;
        self.data = vec![0; size];
        for (d, s) in self
            .data
            .iter_mut()
            .zip(src.iter().skip(mem::size_of::<u64>()))
        {
            *d = *s;
        }
    }

    // The caller needs to ensure that |idx| won't be out of range.
    pub fn nth_value<'a>(&self, schema: &'a Schema, idx: usize) -> Value<'a> {
        let mut value = Value::new(schema.nth_types(idx).unwrap().clone());
        value.deserialize_from(self.nth_data_ptr(schema, idx));
        value
    }

    // The caller needs to ensure that |idx| won't be out of range.
    pub fn nth_is_null(&self, schema: &Schema, idx: usize) -> bool {
        self.nth_value(schema, idx).is_null()
    }

    pub fn to_string(&self, schema: &Schema) -> String {
        let mut s = String::from("(");
        let mut first = true;
        for idx in 0..schema.columns().len() {
            if first {
                first = false;
            } else {
                s.push_str(", ");
            }
            if self.nth_is_null(schema, idx) {
                s.push_str("<NULL>");
            } else {
                s.push_str(&self.nth_value(schema, idx).to_string());
            }
        }
        s.push_str(") ");
        s.push_str(&format!("Tuple size is {}", self.data.len()));
        s
    }

    fn nth_data_ptr(&self, schema: &Schema, idx: usize) -> &[u8] {
        let nth_offset = schema.nth_offset(idx).unwrap();
        let ptr = &self.data.as_slice()[nth_offset..];
        if schema.nth_is_inlined(idx).unwrap() {
            ptr
        } else {
            let str_offset = reinterpret::read_u64(ptr) as usize;
            &ptr[str_offset..]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::column::Column;
    use crate::types::types::Str;
    use crate::types::types::Types;
    use crate::types::types::Varlen;

    fn create_tuple() -> (Schema<'static>, Tuple) {
        let values = vec![
            Value::new(Types::Varchar(Varlen::Owned(Str::Val(
                "Instagram".to_string(),
            )))),
            Value::new(Types::Integer(123456789)),
        ];
        let schema = Schema::new(vec![
            Column::new("Name".to_string(), Types::owned(), 10),
            Column::new("Count".to_string(), Types::integer(), 4),
        ]);
        let tuple = Tuple::new(&values, &schema);
        (schema, tuple)
    }

    #[test]
    fn new_and_nth() {
        let (schema, tuple) = create_tuple();
        assert_eq!(false, tuple.nth_is_null(&schema, 0));
        assert_eq!(false, tuple.nth_is_null(&schema, 1));

        let value1 = Value::new(Types::Varchar(Varlen::Owned(Str::Val(
            "Instagram".to_string(),
        ))));
        let value2 = Value::new(Types::Integer(123456789));
        assert_eq!(Some(true), value1.eq(&tuple.nth_value(&schema, 0)));
        assert_eq!(Some(true), value2.eq(&tuple.nth_value(&schema, 1)));
    }

    #[test]
    fn serialize_and_deserialize() {
        let (_, tuple) = create_tuple();
        let mut buffer: Vec<u8> = vec![0; 100];
        tuple.serialize_to(buffer.as_mut_slice());

        let mut tuple2 = Tuple::default();
        tuple2.deserialize_from(buffer.as_slice());
        assert_eq!(tuple, tuple2);
    }
}
