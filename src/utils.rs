#[derive(Debug)]
pub enum Errors {
    /// Carry is returned when a provided function does not support time carry. For example,
    /// if a Timezone `new` receives 60 seconds and there are only 59 seconds in the provided date
    /// time then a Carry is returned as the Result.
    Carry,
}
