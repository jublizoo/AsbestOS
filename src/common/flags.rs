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


// TODO: Macro for bitflags
