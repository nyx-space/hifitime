use std::fmt;

use super::instant::Instant;

/// Offset is an alias of Instant. It contains the same kind of information, but is used in a
/// very different context
pub type Offset = Instant;

#[derive(Debug)]
pub enum Errors {
    /// Carry is returned when a provided function does not support time carry. For example,
    /// if a Timezone `new` receives 60 seconds and there are only 59 seconds in the provided date
    /// time then a Carry is returned as the Result.
    Carry,
}


impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Errors::Carry => write!(f, "a carry error (e.g. 61 seconds)"),
        }
    }
}
