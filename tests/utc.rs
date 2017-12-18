extern crate hifitime;

#[test]
fn utc_valid_dates() {
    use hifitime::utc::{Utc, USUAL_DAYS_PER_MONTH};
    use hifitime::julian::{SECONDS_PER_DAY, DAYS_PER_YEAR};
    use hifitime::instant::{Era, Instant};
    use hifitime::traits::{TimeSystem, TimeZone};

    let epoch = Utc::new(1900, 01, 01, 0, 0, 0, 0).expect("epoch failed");
    assert_eq!(
        epoch.as_instant(),
        Instant::new(0, 0, Era::Present),
        "Incorrect Epoch computed"
    );

    for dyear in -2..2 {
        let era: Era;
        if dyear >= 0 {
            era = Era::Present;
        } else {
            era = Era::Past;
        }
        for dhour in 0..7 {
            for dminute in (0..60).rev() {
                for dsecond in 0..60 {
                    let utc = Utc::new(
                        1900 + dyear,
                        1,
                        1,
                        dhour as u8,
                        dminute as u8,
                        dsecond,
                        1590,
                    ).expect("epoch plus a day failed");
                    let inst = Instant::new(
                        3600 * dhour + 60 * dminute + dsecond as u64 +
                            (SECONDS_PER_DAY as u64) * 365 * (dyear.abs() as u64),
                        1590,
                        era,
                    );
                    assert_eq!(
                        utc.as_instant(),
                        inst,
                        "Incorrect Epoch+{} year(s) + {} hour(s) + {} minute(s) + {} second(s)
                     + some computed (utc.as_instant)",
                        dyear,
                        dhour,
                        dminute,
                        dsecond
                    );
                    assert_eq!(
                        Utc::from_instant(inst).as_instant(),
                        inst,
                        "Incorrect Epoch+{} year(s) + {} hour(s) + {} minute(s) + {} second(s)
                     + some computed (utc.from_instant)",
                        dyear,
                        dhour,
                        dminute,
                        dsecond
                    );
                }
            }
        }
    }

    assert_eq!(
        Utc::new(1905, 01, 01, 0, 0, 0, 1590).expect("epoch 1905 failed").as_instant(),
        Instant::new(3600 * 24 + (SECONDS_PER_DAY as u64) * 365 * 5, 1590, Era::Present),
        "Incorrect Epoch 1905 + some computed",
    );

    Utc::new(2018, 10, 08, 22, 08, 47, 0).expect("standard date failed");

    assert_eq!(
        Utc::new(1971, 12, 31, 23, 59, 59, 0)
            .expect("January 1972 leap second failed")
            .as_instant(),
        Instant::new(2272060799, 0, Era::Present),
        "Incorrect January 1972 pre-leap second number computed"
    );
    assert_eq!(
        Utc::new(1971, 12, 31, 23, 59, 59, 0)
            .expect("January 1972 1 second before leap second failed")
            .as_instant(),
        Utc::new(1971, 12, 31, 23, 59, 60, 0)
            .expect("January 1972 1 second before leap second failed")
            .as_instant(),
        "Incorrect January 1972 leap second number computed"
    );
    assert_eq!(
        Utc::new(1972, 01, 01, 00, 00, 00, 0)
            .expect("January 1972 leap second failed")
            .as_instant(),
        Instant::new(2272060800, 0, Era::Present),
        "Incorrect January 1972 post-leap second number computed"
    );
    assert_eq!(
        Utc::new(1972, 01, 01, 00, 00, 01, 0)
            .expect("January 1972 leap second failed")
            .as_instant(),
        Instant::new(2272060801, 0, Era::Present),
        "Incorrect January 1972 post-post-leap second number computed"
    );
    assert_eq!(
        Utc::new(1972, 06, 30, 23, 59, 59, 0)
            .expect("July leap second failed")
            .as_instant(),
        Instant::new(2287785599, 0, Era::Present),
        "Incorrect July 1972 pre-leap second number computed"
    );
    assert_eq!(
        Utc::new(1972, 06, 30, 23, 59, 59, 0)
            .expect("July leap second failed")
            .as_instant(),
        Utc::new(1972, 06, 30, 23, 59, 60, 0)
            .expect("July leap second failed")
            .as_instant(),
        "Incorrect July 1972 leap second number computed"
    );
    assert_eq!(
        Utc::new(1972, 07, 01, 00, 00, 00, 0)
            .expect("July leap second failed")
            .as_instant(),
        Instant::new(2287785600, 0, Era::Present),
        "Incorrect July 1972 post-leap second number computed"
    );
    assert_eq!(
        Utc::new(1972, 07, 01, 00, 00, 01, 0)
            .expect("July leap second failed")
            .as_instant(),
        Instant::new(2287785601, 0, Era::Present),
        "Incorrect July 1972 post-post-leap second number computed"
    );
    assert_eq!(
        Utc::new(1993, 06, 30, 23, 59, 59, 0)
            .expect("July leap pre-second failed")
            .as_instant(),
        Instant::new(2950473599, 0, Era::Present),
        "Incorrect July 1993 pre-leap second number computed"
    );
    assert_eq!(
        Utc::new(1993, 06, 30, 23, 59, 59, 0)
            .expect("July leap second failed")
            .as_instant(),
        Utc::new(1993, 06, 30, 23, 59, 60, 0)
            .expect("July leap second failed")
            .as_instant(),
        "Incorrect July 1993 leap second number computed"
    );
    assert_eq!(
        Utc::new(1993, 07, 01, 00, 00, 00, 0)
            .expect("July leap second failed")
            .as_instant(),
        Instant::new(2950473600, 0, Era::Present),
        "Incorrect July 1993 post-leap second number computed"
    );
    assert_eq!(
        Utc::new(1993, 07, 01, 00, 00, 01, 0)
            .expect("July leap second failed")
            .as_instant(),
        Instant::new(2950473601, 0, Era::Present),
        "Incorrect July 1993 post-post-leap second number computed"
    );
    assert_eq!(
        Utc::new(2016, 12, 31, 23, 59, 60, 0)
            .expect("January 2017 leap second failed")
            .as_instant(),
        Instant::new(3692217599, 0, Era::Present),
        "Incorrect January 2017 pre-leap second number computed"
    );
    assert_eq!(
        Utc::new(2016, 12, 31, 23, 59, 59, 0)
            .expect("January 2017 leap second failed")
            .as_instant(),
        Utc::new(2016, 12, 31, 23, 59, 60, 0)
            .expect("January 2017 leap second failed")
            .as_instant(),
        "Incorrect January 2017 leap second number computed"
    );
    assert_eq!(
        Utc::new(2017, 1, 1, 00, 00, 00, 0)
            .expect("January 2017 leap second plus one failed")
            .as_instant(),
        Instant::new(3692217600, 0, Era::Present),
        "Incorrect January 2017 post-leap second plus one number computed"
    );
    assert_eq!(
        Utc::new(2017, 1, 1, 00, 00, 01, 0)
            .expect("January 2017 post-leap second plus one failed")
            .as_instant(),
        Instant::new(3692217601, 0, Era::Present),
        "Incorrect January 2017 post-post-leap second plus one number computed"
    );
    assert_eq!(
        Utc::new(2015, 06, 30, 23, 59, 59, 0)
            .expect("July leap pre-second failed")
            .as_instant(),
        Instant::new(3644697599, 0, Era::Present),
        "Incorrect July 2015 pre-leap second number computed"
    );
    assert_eq!(
        Utc::new(2015, 06, 30, 23, 59, 59, 0)
            .expect("July leap second failed")
            .as_instant(),
        Utc::new(2015, 06, 30, 23, 59, 60, 0)
            .expect("July leap second failed")
            .as_instant(),
        "Incorrect July 2015 leap second number computed"
    );
    assert_eq!(
        Utc::new(2015, 07, 01, 00, 00, 00, 0)
            .expect("July leap second failed")
            .as_instant(),
        Instant::new(3644697600, 0, Era::Present),
        "Incorrect July 2015 post-leap second number computed"
    );
    assert_eq!(
        Utc::new(2015, 07, 01, 00, 00, 01, 0)
            .expect("July leap second failed")
            .as_instant(),
        Instant::new(3644697601, 0, Era::Present),
        "Incorrect July 2015 post-post-leap second number computed"
    );

    // List of leap years from https://kalender-365.de/leap-years.php .
    let leap_years: [i32; 146] = [
        1804,
        1808,
        1812,
        1816,
        1820,
        1824,
        1828,
        1832,
        1836,
        1840,
        1844,
        1848,
        1852,
        1856,
        1860,
        1864,
        1868,
        1872,
        1876,
        1880,
        1884,
        1888,
        1892,
        1896,
        1904,
        1908,
        1912,
        1916,
        1920,
        1924,
        1928,
        1932,
        1936,
        1940,
        1944,
        1948,
        1952,
        1956,
        1960,
        1964,
        1968,
        1972,
        1976,
        1980,
        1984,
        1988,
        1992,
        1996,
        2000,
        2004,
        2008,
        2012,
        2016,
        2020,
        2024,
        2028,
        2032,
        2036,
        2040,
        2044,
        2048,
        2052,
        2056,
        2060,
        2064,
        2068,
        2072,
        2076,
        2080,
        2084,
        2088,
        2092,
        2096,
        2104,
        2108,
        2112,
        2116,
        2120,
        2124,
        2128,
        2132,
        2136,
        2140,
        2144,
        2148,
        2152,
        2156,
        2160,
        2164,
        2168,
        2172,
        2176,
        2180,
        2184,
        2188,
        2192,
        2196,
        2204,
        2208,
        2212,
        2216,
        2220,
        2224,
        2228,
        2232,
        2236,
        2240,
        2244,
        2248,
        2252,
        2256,
        2260,
        2264,
        2268,
        2272,
        2276,
        2280,
        2284,
        2288,
        2292,
        2296,
        2304,
        2308,
        2312,
        2316,
        2320,
        2324,
        2328,
        2332,
        2336,
        2340,
        2344,
        2348,
        2352,
        2356,
        2360,
        2364,
        2368,
        2372,
        2376,
        2380,
        2384,
        2388,
        2392,
        2396,
        2400,
    ];
    for year in leap_years.iter() {
        Utc::new(*year, 02, 29, 22, 08, 47, 0).expect(
            format!(
                "{} leap year failed",
                year
            ).as_str(),
        );
    }
}

#[test]
fn utc_invalid_dates() {
    use hifitime::utc::Utc;
    use hifitime::traits::TimeZone;

    Utc::new(2001, 02, 29, 22, 08, 47, 0).expect_err("29 Feb 2001 did not fail");
    Utc::new(2016, 12, 31, 23, 59, 61, 0).expect_err("January leap second did not fail");
    Utc::new(2015, 06, 30, 23, 59, 61, 0).expect_err("July leap second did not fail");
}
