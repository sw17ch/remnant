use std::fmt;

const DISPLAY_BYTES: usize = 5;

pub fn debug_bytes(f: &mut fmt::Formatter, bytes: &[u8]) -> fmt::Result {
    write!(f, "(")?;
    for b in bytes {
        write!(f, "{:02x}", b)?;
    }
    write!(f, ")")
}

pub fn display_bytes(f: &mut fmt::Formatter, bytes: &[u8]) -> fmt::Result {
    for b in bytes.iter().take(DISPLAY_BYTES) {
        write!(f, "{:02x}", b)?;
    }
    Ok(())
}
