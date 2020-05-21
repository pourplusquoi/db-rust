use log::error;
use std::fmt::Debug;
use std::result::Result;

pub trait ErrorLogging<T>: Sized {
    fn log(&self);
    fn log_and_fn<F, R>(&self, f: F) -> R
    where
        F: Fn(&Self) -> R;
    fn log_and_fn_once<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R;
    fn log_and_ok(self) -> Option<T>;
    fn log_and_is_ok(&self) -> bool;
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

    fn log_and_fn<F, R>(&self, f: F) -> R
    where
        F: Fn(&Self) -> R,
    {
        self.log();
        f(self)
    }

    fn log_and_fn_once<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        self.log();
        f(self)
    }

    fn log_and_ok(self) -> Option<T> {
        self.log_and_fn_once(Self::ok)
    }

    fn log_and_is_ok(&self) -> bool {
        self.log_and_fn(Self::is_ok)
    }
}
