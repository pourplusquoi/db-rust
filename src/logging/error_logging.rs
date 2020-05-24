use log::error;
use std::fmt::Debug;
use std::result::Result;

pub trait ErrorLogging<T>: Sized {
    fn log(&self);
    fn log_and(self) -> Self;
}

impl<T, E> ErrorLogging<T> for Result<T, E>
where
    E: Debug,
{
    fn log(&self) {
        match &self {
            Ok(_) => (),
            Err(e) => {
                error!("{:?}", e);
            }
        };
    }

    fn log_and(self) -> Self {
        self.log();
        self
    }
}
