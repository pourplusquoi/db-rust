use crate::disk::bitmap::Bitmap;
use crate::disk::bitmap::BITS_PER_WORD;
use crate::disk::bitmap::FULL_WORD;
use std::collections::BTreeSet;

pub struct Selector {
    bitmap: Bitmap,
    free: BTreeSet<usize>,
}

impl Selector {
    pub fn new(path: &str) -> std::io::Result<Self> {
        Ok(Selector {
            bitmap: Bitmap::new(path)?,
            free: BTreeSet::new(),
        })
        .and_then(|mut selector| {
            selector.init();
            Ok(selector)
        })
    }

    pub fn vacant(&self) -> usize {
        match self.free.iter().nth(0) {
            Some(&word_idx) => {
                let word = self.bitmap.get_word(word_idx);
                // It is safe to unwrap here, because |word| < |FULL_WORD| always
                // holds.
                let bit_idx = (0..BITS_PER_WORD)
                    .rev()
                    .skip_while(|x| word & (1 << x) > 0)
                    .nth(0)
                    .map(|x| BITS_PER_WORD - 1 - x)
                    .unwrap();
                word_idx * BITS_PER_WORD + bit_idx
            }
            None => self.bitmap.len() * BITS_PER_WORD,
        }
    }

    pub fn set_used(&mut self, idx: usize) {
        let prev = self.bitmap.len();
        let word_idx = idx / BITS_PER_WORD;
        self.bitmap.set_bit(idx, true);
        for i in prev..=word_idx {
            self.free.insert(i);
        }
        if self.bitmap.get_word(word_idx) == FULL_WORD {
            self.free.remove(&word_idx);
        }
    }

    pub fn set_free(&mut self, idx: usize) {
        let prev = self.bitmap.len();
        let word_idx = idx / BITS_PER_WORD;
        self.bitmap.set_bit(idx, false);
        for i in prev..=word_idx {
            self.free.insert(i);
        }
        if self.bitmap.get_word(word_idx) < FULL_WORD {
            self.free.insert(word_idx);
        }
    }

    pub fn is_used(&self, idx: usize) -> bool {
        self.bitmap.get_bit(idx)
    }

    pub fn compact(&mut self) {
        self.bitmap.compact();
        while let Some(&word_idx) = self.free.iter().last() {
            if word_idx >= self.bitmap.len() {
                self.free.remove(&word_idx);
            }
        }
    }

    fn init(&mut self) {
        for word_idx in 0..self.bitmap.len() {
            if self.bitmap.get_word(word_idx) < FULL_WORD {
                self.free.insert(word_idx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::file_deleter::FileDeleter;

    #[test]
    fn set_and_vacant() {
        let path = "/tmp/testfile.selector.1.db";

        // Test file deleter with RAII.
        let mut file_deleter = FileDeleter::new();
        file_deleter.push(&path);

        let result = Selector::new(&path);
        assert!(result.is_ok(), "Failed to create Selector");

        let mut selector = result.unwrap();
        assert_eq!(0, selector.vacant());
        assert_eq!(false, selector.is_used(0));

        for i in 0..128 {
            selector.set_used(i);
            assert_eq!(true, selector.is_used(i));
            assert_eq!(i + 1, selector.vacant());
        }

        selector.set_free(80);
        assert_eq!(false, selector.is_used(80));
        assert_eq!(80, selector.vacant());
        selector.set_used(80);
        assert_eq!(true, selector.is_used(80));
        assert_eq!(128, selector.vacant());

        for i in 64..128 {
            selector.set_free(i);
            assert_eq!(false, selector.is_used(i));
            assert_eq!(64, selector.vacant());
        }

        selector.compact();
        assert_eq!(0, selector.free.len());
        assert_eq!(8, selector.bitmap.len());
    }

    #[test]
    fn drop_new() {
        let path = "/tmp/testfile.selector.2.db";

        // Test file deleter with RAII.
        let mut file_deleter = FileDeleter::new();
        file_deleter.push(&path);

        {
            let result = Selector::new(&path);
            assert!(result.is_ok(), "Failed to create Selector");

            let mut selector = result.unwrap();
            selector.set_used(64);
            selector.set_used(128);

            for i in 0..64 {
                assert_eq!(i, selector.vacant());
                selector.set_used(i);
            }

            for i in 65..128 {
                assert_eq!(i, selector.vacant());
                selector.set_used(i);
            }

            selector.set_used(1024);
            selector.set_free(1024);
            assert_eq!(113, selector.free.len());
            assert_eq!(129, selector.bitmap.len());
        } // Drops selector.

        {
            let result = Selector::new(&path);
            assert!(result.is_ok(), "Failed to create Selector");

            let selector = result.unwrap();
            assert_eq!(1, selector.free.len());
            assert_eq!(17, selector.bitmap.len());

            assert_eq!(true, selector.is_used(64));
            assert_eq!(true, selector.is_used(128));
            assert_eq!(false, selector.is_used(1024));
            assert_eq!(129, selector.vacant());
        } // Drops selector.
    }
}
