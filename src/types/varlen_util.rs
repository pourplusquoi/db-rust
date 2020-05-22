use crate::types::types::Str;
use crate::types::types::Varlen;

pub fn varlen_cmp(lhs: &Varlen, rhs: &Varlen) -> i8 {
    match lhs {
        Varlen::Owned(Str::Val(lhsval)) => str_varlen_cmp(&lhsval, rhs),
        Varlen::Owned(Str::MaxVal) => maxstr_varlen_cmp(rhs),
        Varlen::Borrowed(Str::Val(lhsval)) => str_varlen_cmp(&lhsval, rhs),
        Varlen::Borrowed(Str::MaxVal) => maxstr_varlen_cmp(rhs),
    }
}

fn maxstr_varlen_cmp(other: &Varlen) -> i8 {
    match other {
        Varlen::Owned(Str::Val(_)) => 1,
        Varlen::Owned(Str::MaxVal) => 0,
        Varlen::Borrowed(Str::Val(_)) => 1,
        Varlen::Borrowed(Str::MaxVal) => 0,
    }
}

fn str_varlen_cmp(lhs: &str, rhs: &Varlen) -> i8 {
    match rhs {
        Varlen::Owned(Str::Val(rhsval)) => str_cmp(lhs, &rhsval),
        Varlen::Owned(Str::MaxVal) => -1,
        Varlen::Borrowed(Str::Val(rhsval)) => str_cmp(lhs, rhsval),
        Varlen::Borrowed(Str::MaxVal) => -1,
    }
}

fn str_cmp(lhs: &str, rhs: &str) -> i8 {
    for (i, j) in lhs.chars().zip(rhs.chars()) {
        if i > j {
            return 1;
        } else if i < j {
            return -1;
        }
    }
    if lhs.len() > rhs.len() {
        1
    } else if lhs.len() < rhs.len() {
        -1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_cmp_test() {
        assert_eq!(0, str_cmp("", ""));
        assert_eq!(0, str_cmp("hello", "hello"));
        assert_eq!(-1, str_cmp("he", "hello"));
        assert_eq!(-1, str_cmp("", "hello"));
        assert_eq!(1, str_cmp("hello", "he"));
        assert_eq!(1, str_cmp("hello", ""));
        assert_eq!(-1, str_cmp("hello", "world"));
        assert_eq!(1, str_cmp("world", "hello"));
    }

    #[test]
    fn varlen_cmp_test() {
        assert_eq!(
            0,
            varlen_cmp(
                &Varlen::Borrowed(Str::Val("####")),
                &Varlen::Borrowed(Str::Val("####"))
            )
        );
        assert_eq!(
            0,
            varlen_cmp(
                &Varlen::Borrowed(Str::Val("####")),
                &Varlen::Owned(Str::Val("####".to_string()))
            )
        );
        assert_eq!(
            0,
            varlen_cmp(
                &Varlen::Owned(Str::Val("####".to_string())),
                &Varlen::Borrowed(Str::Val("####"))
            )
        );
        assert_eq!(
            0,
            varlen_cmp(
                &Varlen::Owned(Str::Val("####".to_string())),
                &Varlen::Owned(Str::Val("####".to_string()))
            )
        );
        assert_eq!(
            0,
            varlen_cmp(
                &Varlen::Borrowed(Str::MaxVal),
                &Varlen::Borrowed(Str::MaxVal)
            )
        );
        assert_eq!(
            0,
            varlen_cmp(&Varlen::Borrowed(Str::MaxVal), &Varlen::Owned(Str::MaxVal))
        );
        assert_eq!(
            0,
            varlen_cmp(&Varlen::Owned(Str::MaxVal), &Varlen::Borrowed(Str::MaxVal))
        );
        assert_eq!(
            0,
            varlen_cmp(&Varlen::Owned(Str::MaxVal), &Varlen::Owned(Str::MaxVal))
        );
        assert_eq!(
            -1,
            varlen_cmp(
                &Varlen::Borrowed(Str::Val("abcdefg")),
                &Varlen::Borrowed(Str::MaxVal)
            )
        );
        assert_eq!(
            -1,
            varlen_cmp(
                &Varlen::Borrowed(Str::Val("abcdefg")),
                &Varlen::Owned(Str::MaxVal)
            )
        );
        assert_eq!(
            -1,
            varlen_cmp(
                &Varlen::Owned(Str::Val("abcdefg".to_string())),
                &Varlen::Borrowed(Str::MaxVal)
            )
        );
        assert_eq!(
            -1,
            varlen_cmp(
                &Varlen::Owned(Str::Val("abcdefg".to_string())),
                &Varlen::Owned(Str::MaxVal)
            )
        );
        assert_eq!(
            1,
            varlen_cmp(
                &Varlen::Borrowed(Str::MaxVal),
                &Varlen::Borrowed(Str::Val("abcdefg"))
            )
        );
        assert_eq!(
            1,
            varlen_cmp(
                &Varlen::Borrowed(Str::MaxVal),
                &Varlen::Owned(Str::Val("abcdefg".to_string()))
            )
        );
        assert_eq!(
            1,
            varlen_cmp(
                &Varlen::Owned(Str::MaxVal),
                &Varlen::Borrowed(Str::Val("abcdefg"))
            )
        );
        assert_eq!(
            1,
            varlen_cmp(
                &Varlen::Owned(Str::MaxVal),
                &Varlen::Owned(Str::Val("abcdefg".to_string()))
            )
        );
    }
}
