pub mod instant;

#[cfg(test)]
mod tests {
    use std::time;
    #[test]
    fn it_works() {
        use instant;
        let example = instant::Instant::new(159, 0, instant::Era::Present);
        let delta = time::Duration::new(5, 0);
        println!("{:?}", example + delta);
    }
}
