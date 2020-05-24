#![allow(dead_code)]

use std::boxed::Box;
use std::error;
use std::fmt;
use std::marker::Send;
use std::marker::Sync;

pub struct Error {
    kind: ErrorKind,
    error: Box<dyn error::Error + Send + Sync>,
}

pub enum ErrorKind {
    NotSupported,
    CannotCast,
    DivideByZero,
    SqrtOnNegative,
    Overflow,
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Error {
            kind: kind,
            error: error.into(),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}
