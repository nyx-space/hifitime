extern crate hifitime;

#[test]
fn utc_carry_error() {
    use hifitime::utc::Utc;
    use hifitime::traits::TimeZone;

    let now = Utc::new(2018, 10, 08, 22, 08, 47, 0);
    println!("{:?}", now);
}
