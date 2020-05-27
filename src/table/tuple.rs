#![allow(dead_code)]

use crate::catalog::schema::Schema;
use crate::common::reinterpret;
use crate::types::types::Operation;
use crate::types::value::Value;
use std::clone::Clone;
use std::mem;

#[derive(Clone)]
struct Tuple {
    // rid: RID,
    data: Vec<u8>,
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
        for (s, d) in self.data.iter().zip(dst.iter_mut()) {
            *d = *s;
        }
    }

    // The caller needs to make sure that |src| is valid.
    pub fn deserialize_from(&mut self, src: &[u8]) {
        let size = reinterpret::read_u64(src) as usize;
        self.data = vec![0; size];
        for (d, s) in self.data.iter_mut().zip(src.iter()) {
            *d = *s;
        }
    }

    // The caller needs to ensure that |idx| won't be out of range.
    pub fn value<'a>(&self, schema: &'a Schema, idx: usize) -> Value<'a> {
        let mut value = Value::new(schema.nth_types(idx).unwrap().clone());
        value.deserialize_from(self.data_ptr(schema, idx));
        value
    }

    // The caller needs to ensure that |idx| won't be out of range.
    pub fn is_null(&self, schema: &Schema, idx: usize) -> bool {
        self.value(schema, idx).is_null()
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
            if self.is_null(schema, idx) {
                s.push_str("<NULL>");
            } else {
                s.push_str(&self.value(schema, idx).to_string());
            }
        }
        s.push_str(") ");
        s.push_str(&format!("Tuple size is {}", self.data.len()));
        s
    }

    fn data_ptr(&self, schema: &Schema, idx: usize) -> &[u8] {
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
