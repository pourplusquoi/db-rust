use std::io::Error;
use std::io::ErrorKind;

pub fn already_exists(message: &str) -> Error {
  Error::new(ErrorKind::AlreadyExists, message)
}

pub fn invalid_data(message: &str) -> Error {
  Error::new(ErrorKind::InvalidData, message)
}

pub fn invalid_input(message: &str) -> Error {
  Error::new(ErrorKind::InvalidInput, message)
}

pub fn not_found(message: &str) -> Error {
  Error::new(ErrorKind::NotFound, message)
}