use std::fmt;

pub trait TimeSystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result; // Must be printable
}
