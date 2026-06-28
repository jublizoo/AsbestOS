/// - `should_set`: True if the bitmask bits should be set, false if they should be cleard.
pub fn write_byte_bitmask(original: u8, bitmask: u8, should_set: bool) -> u8 {
    if should_set {
        original | bitmask
    } else {
        original & !bitmask
    }
}

/// - `should_set`: True if the bitmask bits should be set, false if they should be cleard.
pub fn write_byte_flag(original: u8, flag_idx: u8, should_set: bool) -> u8 {
    write_byte_bitmask(original, 1 << flag_idx, should_set)
}

/// Returns a u64 with 1s only between bits `start` and `end`
/// 0-indexed, from LSB, start and end inclusive.
/// `start`/`end` order does not mattter.
pub const fn range_mask(start: u8, end: u8) -> u64 {
    assert!(start < 64 && end < 64);

    (((1_i64 << 63) >> (63 - end)) ^ ((1_i64 << 63) >> (63 - start))) as u64
}

// TODO: Macro for bitflags
