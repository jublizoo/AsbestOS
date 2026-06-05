use core::ops::{Add, Sub, Div, Rem};

trait Numeric: 
    Copy + Add<Output = Self> + Sub<Output = Self> + Rem<Output = Self>
    + Div<Output=Self> + PartialEq<Self>
{ }

impl Numeric for u8 {}
impl Numeric for u16 {}
impl Numeric for u32 {}
impl Numeric for usize {}
impl Numeric for i8 {}
impl Numeric for i16 {}
impl Numeric for i32 {}
impl Numeric for isize {}

trait HasZero: Numeric {
    const ZERO: Self;
}

impl HasZero for u8 { const ZERO: Self = 0; }
impl HasZero for u16 { const ZERO: Self = 0; }
impl HasZero for u32 { const ZERO: Self = 0; }
impl HasZero for usize { const ZERO: Self = 0; }
impl HasZero for i8 { const ZERO: Self = 0; }
impl HasZero for i16 { const ZERO: Self = 0; }
impl HasZero for i32 { const ZERO: Self = 0; }
impl HasZero for isize { const ZERO: Self = 0; }

trait Unsigned: Numeric { }

impl Unsigned for u8 {}
impl Unsigned for u16 {}
impl Unsigned for u32 {}
impl Unsigned for usize {}

pub trait Alignable {
    fn align_up(self, multiple: Self) -> Self;
    fn align_down(self, multiple: Self) -> Self;
}

impl<T: Unsigned + HasZero> Alignable for T {
    /// Round up, but will not modify exact multiples
    #[inline(always)]
    fn align_up(self, multiple: Self) -> Self {
        let rem = self % multiple;
        if rem == T::ZERO {
            self
        } else {
            self + (multiple - rem)
        }
    }

    /// Round down, but will not modify exact multiples
    #[inline(always)]
    fn align_down(self, multiple: Self) -> Self {
        self - (self % multiple)
    }
}

impl<T> Alignable for *const T {
    fn align_up(self, multiple: Self) -> Self {
        (self as usize).align_up(multiple as usize) as Self
    }

    fn align_down(self, multiple: Self) -> Self {
        (self as usize).align_up(multiple as usize) as Self
    }
}

impl<T> Alignable for *mut T {
    fn align_up(self, multiple: Self) -> Self {
        (self as usize).align_up(multiple as usize) as Self
    }

    fn align_down(self, multiple: Self) -> Self {
        (self as usize).align_up(multiple as usize) as Self
    }
}
