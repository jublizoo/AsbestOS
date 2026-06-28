use core::ops::{BitAnd, BitOr, Not, Shl};

/// - `should_set`: True if the bitmask bits should be set, false if they should be cleard.
pub fn write_byte_bitmask<T>(original: T, bitmask: T, should_set: bool) -> T
where 
    T: BitOr<Output = T> + BitAnd<Output = T> + Not<Output = T>
{
    if should_set {
        original | bitmask
    } else {
        original & !bitmask
    }
}

/// - `should_set`: True if the bitmask bits should be set, false if they should be cleard.
pub fn write_byte_flag<T>(original: T, flag_idx: u8, should_set: bool) -> T
where 
    T: From<u8> + BitOr<Output = T> + BitAnd<Output = T> + Not<Output = T>
{
    write_byte_bitmask(original, T::from(1 << flag_idx), should_set)
}

/// Returns a u64 with 1s only between bits `start` and `last`
/// 0-indexed, from LSB, `start` and `last` inclusive.
/// `start`/`last` order does not mattter.
pub const fn range_mask(start: u8, last: u8) -> u64 {
    assert!(start < 64 && last < 64);

    (((1_i64 << 63) >> (63 - last)) ^ ((1_i64 << 63) >> (63 - start))) as u64
}

// TODO: Macro for bitflags
