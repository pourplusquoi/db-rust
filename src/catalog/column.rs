#![allow(dead_code)]

use crate::types::types::Types;

pub struct Column<'a> {
    // The name of the column.
    name: String,
    // The value type of column.
    types: Types<'a>,
    // Whether the column is inlined.
    inlined: bool,
    // The offset of column in tuple.
    offset: Option<usize>,
    // If the column is not inlined, this is set to pointer size; else, it is
    // set to length of the fixed length.
    fixed_len: usize,
    // If the column is inlined, this is set to 0; else, it is set to length of
    // the variable length.
    variable_len: usize,
}

impl<'a> Column<'a> {
    pub fn new(name: String, types: Types<'a>, length: usize) -> Self {
        Column {
            name: name,
            types: types,
            inlined: false,
            offset: None,
            fixed_len: 0,
            variable_len: 0,
        }
        .init(length)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn types(&self) -> &Types {
        &self.types
    }

    pub fn offset(&self) -> usize {
        self.offset.expect("Offset unset")
    }

    pub fn len(&self) -> usize {
        if self.inlined {
            self.fixed_len
        } else {
            self.variable_len
        }
    }

    pub fn is_inlined(&self) -> bool {
        self.inlined
    }

    pub fn fixed_len(&self) -> usize {
        self.fixed_len
    }

    pub fn variable_len(&self) -> usize {
        self.variable_len
    }

    pub fn eq(&self, other: &Self) -> bool {
        self.types.id() == other.types.id() && self.inlined == other.inlined
    }

    pub fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = Some(offset);
    }

    pub fn to_string(&self) -> String {
        let length = if self.inlined {
            format!("FixedLength:{}", self.fixed_len)
        } else {
            format!("VariableLength:{}", self.variable_len)
        };
        format!(
            "Column[{}, {}, Offset:{}, {}]",
            self.name,
            self.types.name(),
            self.offset(),
            length
        )
    }

    fn init(mut self, length: usize) -> Self {
        self.set_inlined();
        self.set_len(length);
        self
    }

    fn set_inlined(&mut self) {
        self.inlined = self.types.is_inlined();
    }

    fn set_len(&mut self, length: usize) {
        if self.inlined {
            self.fixed_len = length;
            self.variable_len = 0;
        } else {
            self.fixed_len = 4;
            self.variable_len = length;
        }
    }
}
