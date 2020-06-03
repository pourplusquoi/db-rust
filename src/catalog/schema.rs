use crate::catalog::column::Column;
use crate::types::types::Types;
use std::cmp::Eq;
use std::cmp::PartialEq;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Schema<'a> {
    len: usize,
    columns: Vec<Column<'a>>,
    // Indices of uninlined columns starting from 0.
    uninlined: Vec<usize>,
}

impl<'a> Schema<'a> {
    pub fn new(columns: Vec<Column<'a>>) -> Self {
        Schema {
            len: 0,
            columns: columns,
            uninlined: Vec::new(),
        }
        .init()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn columns(&self) -> &Vec<Column> {
        &self.columns
    }

    pub fn uninlined(&self) -> &Vec<usize> {
        &self.uninlined
    }

    pub fn is_inlined(&self) -> bool {
        self.uninlined.is_empty()
    }

    pub fn nth_is_inlined(&self, idx: usize) -> Option<bool> {
        self.nth_column(idx).map(|x| x.is_inlined())
    }

    pub fn nth_offset(&self, idx: usize) -> Option<usize> {
        self.nth_column(idx).map(|x| x.offset())
    }

    pub fn nth_types(&self, idx: usize) -> Option<&Types> {
        self.nth_column(idx).map(|x| x.types())
    }

    pub fn nth_len(&self, idx: usize) -> Option<usize> {
        self.nth_column(idx).map(|x| x.len())
    }

    pub fn nth_fixed_len(&self, idx: usize) -> Option<usize> {
        self.nth_column(idx).map(|x| x.fixed_len())
    }

    pub fn nth_variable_len(&self, idx: usize) -> Option<usize> {
        self.nth_column(idx).map(|x| x.variable_len())
    }

    pub fn nth_column(&self, idx: usize) -> Option<&Column> {
        self.columns.iter().nth(idx)
    }

    pub fn column_idx(&self, name: &str) -> Option<usize> {
        for (idx, column) in self.columns.iter().enumerate() {
            if column.name() == name {
                return Some(idx);
            }
        }
        None
    }

    pub fn to_string(&self) -> String {
        format!(
            "Schema[NumColumns:{}, IsInlined:{}, Length:{}]",
            self.columns.len(),
            self.is_inlined(),
            self.len
        )
    }

    fn init(mut self) -> Self {
        let mut offset = 0;
        for (idx, column) in self.columns.iter_mut().enumerate() {
            if !column.is_inlined() {
                self.uninlined.push(idx);
            }
            column.set_offset(offset);
            offset += column.fixed_len();
        }
        self.len = offset;
        self
    }
}

impl<'a> PartialEq for Schema<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.columns.len() != self.columns.len() || self.is_inlined() != other.is_inlined() {
            return false;
        }
        for (lhs, rhs) in self.columns.iter().zip(other.columns.iter()) {
            if lhs != rhs {
                return false;
            }
        }
        true
    }
}

impl<'a> Eq for Schema<'a> {}
