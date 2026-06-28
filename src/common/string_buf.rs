use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::TryReserveError;
use core::{fmt::{self, Error}, ops::Deref};
use crate::no_alloc;

#[derive(Clone)]
pub struct StringBuf {
    buf: Vec<u8>,
}

impl StringBuf {
    pub const fn new() -> Self {
        Self {
            buf: Vec::new(),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            buf: self.buf.clone()
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn extend(&mut self, s: &str) -> Result<(), ()> {
        if self.buf.try_reserve(s.len()).is_err() { return Err(()); }
        no_alloc!( self.buf.extend(s.as_bytes()) );
        Ok(())
    }

    pub fn try_push(&mut self, c: u8) -> Result<(), ()> {
        if self.buf.try_reserve(1).is_err() { return Err(()); }
        no_alloc!( self.buf.push(c) );
        Ok(())
    }
}

impl<'a> Deref for StringBuf {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        core::str::from_utf8(&self.buf).unwrap()
    }
}

// Fuck you fake clippy
impl Into<String> for StringBuf {
    fn into(self) -> String {
        String::from_utf8(self.buf).unwrap()
    }
}

impl fmt::Write for StringBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let res: Result<(), TryReserveError> = self.buf.try_reserve(bytes.len());
        if res.is_err() {
            return Err(Error {});
        }
        no_alloc!(self.buf.extend_from_slice(bytes));

        Ok(())
    }
}
