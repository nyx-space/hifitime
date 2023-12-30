#[cfg(feature = "ut1")]
#[test]
fn test_ut1_from_file() {
    use core::str::FromStr;
    use hifitime::ut1::Ut1Provider;
    use hifitime::Epoch;

    let provider = Ut1Provider::from_eop_file("data/eop-2021-10-12--2023-01-04.short").unwrap();

    println!("{}", provider);

    // Grabbed from AstroPy:
    // >>> Time("2022-01-03 03:05:06.789101")
    // <Time object: scale='utc' format='iso' value=2022-01-03 03:05:06.789>
    // >>> Time("2022-01-03 03:05:06.789101").ut1
    // <Time object: scale='ut1' format='iso' value=2022-01-03 03:05:06.679>
    // >>>
    //
    let epoch = Epoch::from_str("2022-01-03 03:05:06.7891").unwrap();
    assert_eq!(
        format!("{:x}", epoch.to_ut1(provider)),
        "2022-01-03T03:05:06.679020600 TAI"
    );
}

#[cfg(feature = "ut1")]
#[test]
fn test_ut1_from_jpl() {
    use core::str::FromStr;
    use hifitime::ut1::Ut1Provider;
    use hifitime::Epoch;

    // Download a specific version of the UT1 file
    let provider = Ut1Provider::download_from_jpl("221222_190002-marge_eop2.short").unwrap();

    println!("{}", provider);

    // Grabbed from AstroPy:
    // >>> Time("2022-01-03 03:05:06.789101")
    // <Time object: scale='utc' format='iso' value=2022-01-03 03:05:06.789>
    // >>> Time("2022-01-03 03:05:06.789101").ut1
    // <Time object: scale='ut1' format='iso' value=2022-01-03 03:05:06.679>
    // >>>
    //
    let epoch = Epoch::from_str("2022-01-03 03:05:06.7891 UTC").unwrap();
    let ut1_epoch = epoch.to_ut1(provider);
    assert_eq!(
        format!("{:x}", ut1_epoch),
        "2022-01-03T03:05:43.789100000 TAI",
    );
}
