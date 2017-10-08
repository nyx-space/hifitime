use std::fmt;

pub trait TimeSystem {
    fn From(self::Instant); // Constructor, e.g. ModifiedJulian::From(instant);
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result; // Must be printable
}
