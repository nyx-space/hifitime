use std::f32::consts::E;

use hifitime::efmt::consts::*;
use hifitime::prelude::*;

#[test]
fn epoch_parse_with_format() {
    use core::str::FromStr;
    let e = Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33);

    assert_eq!(
        ISO8601_DATE.parse("2015-02-07").unwrap(),
        Epoch::from_gregorian_utc_at_midnight(2015, 2, 7)
    );

    assert_eq!(ISO8601.parse("2015-02-07T11:22:33.0 UTC").unwrap(), e);
    assert_eq!(ISO8601_FLEX.parse("2015-02-07T11:22:33.0 UTC").unwrap(), e);
    assert_eq!(ISO8601_FLEX.parse("2015-02-07T11:22:33").unwrap(), e);

    assert_eq!(ISO8601_STD.parse("2015-02-07T11:22:33.0").unwrap(), e);

    #[cfg(feature = "std")]
    {
        // Test an epoch that's much more precise than usual time keepers
        let e_prec = Epoch::from_gregorian_utc(2015, 2, 7, 11, 22, 33, 123456789);
        assert_eq!(e_prec.to_isoformat(), "2015-02-07T11:22:33.123456");
        assert_ne!(
            e_prec.to_isoformat(),
            Formatter::new(e_prec, ISO8601).to_string()
        );
    }

    assert_eq!(RFC3339.parse("2015-02-07T11:22:33.0 UTC").unwrap(), e);

    assert!(RFC3339.parse("2018-02-13T23:08:32Z").is_ok());

    assert!(RFC3339.parse("2018-02-13T23:08:32.123Z").is_ok());

    assert!(RFC3339.parse("2018-02-13T23:08:32.123456983Z").is_ok());

    assert_eq!(
        ISO8601_ORDINAL
            .parse(&format!("{}", Formatter::new(e, ISO8601_ORDINAL)))
            .unwrap(),
        Epoch::from_gregorian_utc_at_midnight(2015, 2, 7) // Ordinal removes the knowledge below days
    );

    // Confirmation that https://github.com/nyx-space/hifitime/issues/202 is a documentation problem and not a functionality problem.
    let fmtd = Formatter::new(e, Format::from_str("%H:%M").unwrap());
    assert_eq!(format!("{fmtd}"), format!("11:22"));
}

#[test]
fn epoch_format_rfc2822() {
    let epoch = Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33);

    assert_eq!(
        format!("{}", Formatter::new(epoch, RFC2822_LONG)),
        "Saturday, 07 February 2015 11:22:33"
    );

    assert_eq!(
        RFC2822
            .parse("Saturday, 07 February 2015 11:22:33")
            .unwrap(),
        epoch
    );

    // Note that the space is optional when parsing, that's because of how the parsing is done.
    assert_eq!(
        RFC2822.parse("Saturday,07 February 2015 11:22:33").unwrap(),
        epoch
    );

    assert_eq!(
        format!("{}", Formatter::new(epoch, RFC2822)),
        "Sat, 07 Feb 2015 11:22:33"
    );

    assert_eq!(RFC2822.parse("Sat,07 Feb 2015 11:22:33").unwrap(), epoch);
    assert_eq!(
        RFC2822_LONG.parse("Sat,07 Feb 2015 11:22:33").unwrap(),
        epoch
    );

    // Ensure that we check the weekday is valid.
    assert_eq!(
        RFC2822.parse("Fri, 07 Feb 2015 11:22:33"),
        Err(HifitimeError::Parse {
            source: hifitime::ParsingError::WeekdayMismatch {
                found: Weekday::Friday,
                expected: Weekday::Saturday
            },
            details: "weekday and day number do not match"
        })
    );

    // In RFC2822, only the seconds are displayed, so adding microseconds here won't change the output
    assert_eq!(
        format!("{}", Formatter::new(epoch + 2 * Unit::Microsecond, RFC2822)),
        "Sat, 07 Feb 2015 11:22:33"
    );

    // But removing microseconds will cause a rounding the other way.
    assert_eq!(
        format!("{}", Formatter::new(epoch - 2 * Unit::Microsecond, RFC2822)),
        "Sat, 07 Feb 2015 11:22:32"
    );

    assert_eq!(
        Epoch::from_format_str("Sat, 07 Feb 2015 11:22:33", "%a, %d %b %Y %H:%M:%S").unwrap(),
        epoch
    );

    assert_eq!(
        Epoch::from_str_with_format("Sat, 07 Feb 2015 11:22:33", RFC2822).unwrap(),
        epoch
    );

    assert_eq!(
        Epoch::from_format_str("Sat, 07 Feb 15 11:22:33", "%a, %d %b %y %H:%M:%S").unwrap(),
        epoch
    );
}

#[test]
fn regression_test_gh_244() {
    assert_eq!(
        Epoch::from_format_str("Y\u{c}ڰ%d\t\u{16}(\u{e}\u{f}\u{f}#\0d\u{f}AAAA918199\u{f}\u{f}4\u{1d}11-011-\0\0\0 \0\0\t\u{16}\t\u{16}(\u{e}MMMMMMMMMMMMMMMMMMMMMMM\u{f}\u{e}\u{c}\u{10}\u{f}\0\u{f}\u{f}\u{f}\0\u{1}\0\0 \0MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM", "MMM%%%%Y\u{c}ڰd%z%Y\u{c}ڰd%z%%%Y\u{c}ڰd%z%Y\u{c}ڰ%d\t\u{16}(\u{e}\u{f}MMMMMMMMMMMMMMMMMMMMMMMM%%%%Y\u{c}ڰd%z%Y\u{c}ڰd%z%%%Y\u{c}ڰd%z%Y\u{c}ڰ%d\t\u{16}(\u{e}\u{f}\u{f}#\0d\u{f}AAAA918199\u{f}\u{f}4\u{1d}11-05j\t\u{16})\u{e}\u{f}\u{f}#\0d\u{f}AAAA9"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("HHHHHHHHHHH%A\n\nt%z%%AAHHHHHHHHHHHHd\0\0\0HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH/>HHHHHHHAڰ\nI\n%%A%", "%A\n\nt%z%%AA0000%z %AAڰ\nI\n%%A%%A\n\nt%z%%AA000000000%m%AAAAAA%z%A\n\nt%z%%AAHHHHHHHH\0\u{4}H.591)19u\u{f}\u{f}4\u{1d}11405j0%%%%%zڰd%z%%d\0\0"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABBBB", "BBBBBBBBBBBBBBBBBBBBBBBBBBBBm%AAAAAc%z%A\n\nt%z%%AAHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHAڰ\nI\n%%A%%A\n\nt%z%%AA000000000%m%AAAAAA%z%A\n\nt%z%%AAڰ\nI\n%%A%%A\n"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
}

#[test]
fn regression_test_gh_246() {
    assert_eq!(
        Epoch::from_format_str("J4JJJJJJ00002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000005000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000I000000000000000000000000JJJJJ002344086123440861000000000000000000000000000000000JJJJJJJJJJJJJJJJJJJJJJ000000000000000000000000000000000000000000000JJJJJJJJJJJJJJJJJJJJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ0000000000JJJJJJJJJoJJJJJJJJJ0\u{5}\u{5}JJJJJJJJ0JJJJ0000000000000000000000000000000000000JJJJJJJJJJJJJJJJJJJJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ0000000000JJJJJJJJJoJJJJJJJJJ0\u{5}\u{5}JJJJJJJJ0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ\u{10}JJJJJJlJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJd\0\0\0JJJJJJJJJJoJJJJJJJJJ9JJJJJJJJJJJ", "JJJJJJJJJ)JJJJJJJJJJJJJJJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ%%%JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ\u{5}2@0618JJ0JJJJJJ%S%J%JJJJJJJJJJNJJJJJJJJJJ0000000000JJJJJJJ7JJJJJ\nJ(JJ\0\0\0\u{5}\u{5}\u{5}\u{5}J4JJJJJJ0000200000000000000O000000000084311JJJJJJJJ0\0\0\0dJJJJJJJJJJJJJJJJJJJJJJJJJJJJ\u{10}JJJJJJlJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJd\0\0\0JJJJJJJJJJoJJJJJJJJJ9JJJJJJJJJJJJJJJJJJJJ)JJJJJJJJJJJJJJJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ%%%JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ%JJJJJJJJJJJJJJ00000000000000000000000000000000000000000\u{15}00000000000000000000000000000000000000000JJJJJJJJ%JJJJJJJJ7JJJJJJJ(JJ\0\0\0\u{5}\u{5}\u{5}\u{5}J4JJJJJJ00002000000000000000000000000JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ%J0000000000000000000000000000000000000000000JJJJJJJJ%JJJJJJJJ7JJJJJJJ(JJ\0\0\0\u{5}\u{5}\u{5}\u{5}J4JJJJJJ000000000000000000000000JJJJJJJJ"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("\u{1a}5%JJJJJJCJ0\0\0\0dTƻ%TJ+\0\0\0eT190333919002344086184311J7JJJJJJ0\0\0\u{10}dJJ%z%JJJJJJ_JJJ\u{5}JJJJJJJJJJJJJJJJJJ000000000000000000000000000000000000000000000JJJJJJJJJJJJJJJJJJJJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ0\u{5}\u{5}JJJJJJJJ0JJJJ00000\u{5}\u{5}JJJJJJJJ0JJJJ0000000000000000000000000000000000000JJ00JJJJJJJJJJJJJJJJJJJJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ0\u{10}00000000JJJJJJJJJo0\u{10}00000000JJJJJJJJJoJJJJJJJJJ000000000000000000000000000000000JJJJJJJJJJJJJJJJJJ", "JJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJIJJJJJJJJJJJJ>JJJJJ000\r000000JJJJJJJJJoJJJJJJJJJ0\u{5}\u{5}JJJJJJJJ0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ\u{10}JJJJJJlJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJd\0\0\0JJJJJJJJJJoJJJJJJJJJ9JJJJJJJJJJJJJJJJJJJJ)JJJIJJJJJJJJJJJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ%%%JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ\u{5}2@0618JJ0JJJJJJ%S%J%JJJJJJJJJJNJJ9JJJJJJJ0000000000JJJJJJJ7JJJJJJJJJJJJJJJJJJJJJJJJJ\u{5}2@0618JJ0JJJJJJ%S%J%JJJJJJJJJJNJJJJJJJJJJ0000000000JJJJJJJ7ƻ%TJ"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("20091@JJ0JJJJJJ%B%\u{5}00000000000000000000000:00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000JJJJJJJJ%JJJJJJJJ0000002018JJ0J00000000000000000000000000000000000000000000000000000000000000000000000000000000000JJJJJJJJ%JJJJJJJJ0000002018JJ0JJJJJJ%B%BAJJJ\u{1}\0JJ=JJJJJJB| minute%BAJJJ\u{1}\0JEJJJ%B%BAJJI\u{1}\0JJ=JJJJJJJJJJJJJ0000000000000000000JJJ0000002018JJ0J", "JJJJJ%B%BAJJJ\u{1}\0JJ=JJJJJJB| minute%BAJJJ\u{1}\0JEJJJ%B%BAJJI\u{1}\0JJ=JJJJJJJJJJJJJ000000000000000\u{1}\0JJ=JJJJJJJJJJJJJ0000000000000000000JJJ00JJJJJ%B%BAJJJ\u{1}\0JJ=JJJJJJB| minute%BAJJJ\u{1}\0JEJJJ%B%BAJJI\u{1}\0JJ=JJJJJJJJJJJJJ0000000000000000000JJJ0000002018JJ0JJJJJJ%B%BAJJJ\u{1}\0JJ=JJJJJJB| minute%BAJJJ\u{1}\0JEJJJ%B%BAJJI\u{1}\0JJ=JJJJJJJJJJJJJ000000000000000\u{1}\0JJ=JJJJJJJJJJJJJ0000000000000000000JJJ0000002018JJ0JJJJJJ%B%BAJJJ\u{1}\0JJ=JJJJJJBn minute%BAJJJ\u{1}\0JEJJJ%B%BAJJI\u{1}"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("\u{1e}\u{5}\u{5}3\u{5}0t\u{7f}m\u{10}\0\0\0\0\u{17}\u{5}\u{5}\u{5}200618JJ0JJJJ\u{5}0t\u{7f}m%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%\u{b}%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%\0\0\0\0%%%%%%%jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjj~jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjOjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjJ0JJJJ\u{5}0t\u{7f}m%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%\u{b}%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%\0\0\0\0%%%%%%%jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjj~jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjOjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjj", "jjjjjjjjjjjjjjjjjjjHjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjj.0123JJ%A%jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjijjjjjjjjjjjjjjjjjjjjjjjjjkjjjjjjjjjjjjjjjjjbjjjjjjjjjjj.0123JJ%A%jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjijjjjjjjjjjjjjjjjjjjjjjjjjkjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjj.0123JJ%A%jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjkjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjj.0123JJ%A%jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjHjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjj.0123JJ%A%jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjijjjjjjjjjjjjjjjjjjjjjjjjjkjjjjjjjjjjjjjjjjjbjjjjjjjjjjj.0123JJ%A%jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjijjjjjjjjjjjjjjjjjjjjjjjjjkjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjj.0123JJ%A%jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjkjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjj.0123JJ%A%jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjk"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("Àt.036919002\u{13}44086123440861000000000000000000\0\u{10}0000000000000JJJJJJJJJJJJJJJJJJJJJJ000000000000000000000000000000000000000000000JJJJJJJJJJJJJJJJJJJJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ0000000000JJJJJJJJJoJJJJJJJJJ0\u{5}\u{5}JJJJJJJJ0JJJJ0000000000000000000000000000000000000JJJJJJJJJJJJJJJJJJJJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ0E\u{5}\0\0\0J0000000000JJJJJJJJJoJJJJJJJJJ0\u{5}\u{5}JJJJJJJJ0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ\u{10}JJJJJJlJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJd\0\0\0JJJJJJJJJJoJJJJJJJJJ9JJJJJJJJJJJJJJJJJJJJ)JJJJJJJJJJJJJJJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ%%%JJJJ", "JJJJJJJJJJJJJJJJJJJJJJJJJJJJJ\u{5}2@0618JJ0JJJJJJ%S%J%JJJJJJJJJJNJJJJJJJJJJ0000000000JJJJJJJJNJJJJJJJJJJ0000000000JJJJJJJ7JJJJJ\nJ(JJ\0\0\0\u{5}\u{5}\u{5}\u{5}J4JJJJJJ0000200000000000000O0000000minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJ7JJJJJ\nJ(JJ\0\0\0\u{5}\u{5}\u{5}\u{5}J4JJJJJJ0000200000000000000O0000000minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ%%JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJd\0\0\0JJJJJJJJJJoJJJJJJJJJ9JJJJJJJJJJJJJJJJJJJJ)JJJJJJJJJJJJJJJJJJ%%%JJJJ-0000 02)9minJM%S%J%JJJ\u{1}\0JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ%%%JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ\u{5}2@0618JJ0JJJJJJ%S%J%JJJJJJJJJJNJJJJJJJJJJ0000000000JJJJJJJ7JJJJJ\nJ(JJ\0\0\0\u{5}\u{5}\u{5}\u{5}J4JJJJJJ0000200000000000000O0000000000"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("\u{1e}\u{5}\u{5}3\u{5}0t\u{7f}m\u{10}\0\0\0\0\u{17}\u{5}\u{5}\u{5}200618JJ0JJJJ\u{5}0t\u{7f}m\u{10}\0bbbbbbbb%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%% %%<%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%m%m%J\"J\0\u{4}23Jm%m%J\"J\0%%%%%%%%%%%%%%%%%%%%%%%%%%%%", "%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%m%m%J\"J\0\u{4}23Jm%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%m%m%J\"J\0\u{4}23Jm%m%J\"J\0\u{4}bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb%m%m%J\"J\0\u{4}23Jm%m%J\"J\0\u{4}bbbbbbbbbbbbbbbbbbbbbbbbbbbb%m%m%J\"J\0\u{4}23Jm%m%bJ"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("\u{1e}\u{5}\u{5}3\u{5}0t\u{7f}m\u{10}\0\0\0\0\u{17}\u{5}\u{5}\u{5}200618JJ0JJJJ\u{5}0t\u{7f}m\u{10}\0bbbbbbbb%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%% %%<%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%\u{b}%%", "%%%%AJJJJJJeA%JA%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%m%m%J\"J\0\u{4}23Jm%m%J\"J\0\u{4}bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb%m%m%J\"J\0\u{4}23Jm%m%J\"J\0\u{4}bbbbbbbbbbbbbbbbbbbbbbbbbbbb%m%m%J\"J\0\u{4}23Jm%m%bJ"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("J4JJJJJJ000020000000000000000000000000\r000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000JJJJJJJJJJ16JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ00000000000000000000000000000000000\u{1c}0000000000000000000000000000000000000000000000000000000000000000000000000J0\0\00000000000\r000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000JJJJJJJJJJ16JJJJJJJJJJJJJJJJJJJJJJJJJJJJ", "JJJJJJJJJJJ00000000000000000000000000000000000\u{1c}0000000000000000000000000000000000000000000000000000000000000000000000000J0\0\0\0dMƻ%%MJ00000000000000000000000000000000000000000000000000000000000000000000000JJJJJJJJ%JJJJJJJJ7JJ00000000000000000JJJJJJJJJJJJJJJJJJJJJJJJJJJcJJJJJJJJKJJJJJJJJcJJJJKKJ\0\u{5}\u{5}\u{17}\0\u{1}\u{12}\u{5}\u{5}\u{1a}\u{5}+\u{5}N\u{5}\u{5}\u{5}j\0\0\0E\u{5}\u{5}\u{5}JJJJJ2J0\0\0\u{5}\u{5}\u{17}\0\u{1}\u{12}\u{5}\u{5}\u{1a}\u{5}\u{5}\u{5}\u{17}\0\u{1}\u{12}\u{5}\u{5}\u{1a}\u{5}+\u{5}N\u{5}\u{5}\u{5}j\0\0\0\u{5}\u{5}\u{5}\u{5}JJJJJCJ0\0\0\u{5}\u{5}\u{17}\0\u{1}\u{12}\u{5}\u{5}\u{1a}\u{5}+\u{5}N\u{5}\u{5}\u{5}j\0\u{1}\0\u{5}\u{5}\u{5}\u{5}\u{1a}5%JJJ\u{7f}dTƻ%TJ+\u{2}\0\0dTƻ%T\0\u{7f}dTƻ%TJ+\0d\0dTƻ%TJ+\u{5}N\u{5}\u{5}\u{5}j\0\u{1}\0\u{5}\u{5}\u{5}\u{5}\u{1a}5%JJJ\u{7f}dTƻ%TJ+\0\0\0dTƻ%T\0\u{7f}dTƻ%TJ+\0\0\0\u{4}\0ƻ%TJ0000000000000000000JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ%JJJJJJ000000000JJJJJJJJ%JJJJJJJJ000#000JJJJJJJJ%JJJJJeJJ0\r00F02018JJ\0dMƻ%%MJ00000000000000000000000000000000000000000000000000000000000000000000000JJJJJJJJ%JJJJJJJJ7JJ00000000000000000JJJJJJJJJJJJJJJJJJJJJJJJJJJcJJJJJJJJKJJJJJJJJcJJJJKKJ\0\u{5}\u{5}\u{17}\0\u{1}\u{12}\u{5}\u{5}\u{1a}\u{5}+\u{5}N\u{5}\u{5}\u{5}j\0\0\0E\u{5}\u{5}\u{5}JJJJJ2J0\0\0\u{5}\u{5}\u{17}\0\u{1}\u{12}\u{5}\u{5}\u{1a}\u{5}\u{5}\u{5}\u{17}\0\u{1}\u{12}\u{5}\u{5}\u{1a}\u{5}+\u{5}N\u{5}\u{5}\u{5}j\0\0\0\u{5}\u{5}\u{5}\u{5}JJJJJCJ0\0\0\u{5}\u{5}\u{17}\0\u{1}\u{12}\u{5}\u{5}\u{1a}\u{5}+\u{5}N\u{5}\u{5}\u{5}j\0\u{1}\0\u{5}\u{5}\u{5}\u{5}\u{1a}5%JJJ\u{7f}dTƻ%TJ+\u{2}\0\0dTƻ%T\0\u{7f}dTƻ%TJ+\0d\0dTƻ%TJ+\u{5}N\u{5}\u{5}\u{5}j\0\u{1}\0\u{5}\u{5}\u{5}\u{5}\u{1a}5%JJJ\u{7f}dTƻ%TJ+\0\0\0dTƻ%T\0\u{7f}dTƻ%TJ+\0\0\0\u{4}\0ƻ%TJ0000000000000000000JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ%JJJJJJ000000000JJJJJJJJ%JJJJJJJJ000#000JJJJJJJJ%JJJJJeJJ0\r00F02018JJ0JJJ9JJ%B%BAJJJ\u{1}\0JJ=JJJJJJB| minute%6AJJJ\u{1}\0JEJJJ%b%BEJJJ%B%BAK000F02018JJ0JJJJJJ%B%BAJJJ"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("\0\u{8} 11-&.\u{f}\u{f}\0\u{f}\u{f}\u{f}\0\u{1}\u{3}\u{8} 11-&5j0\u{f}\u{f}\0\u{4}\u{1}\0\u{f}\u{f}\0\u{f}\u{f}\u{f}\0\u{1}\0\u{8} 11-&5j0\u{f}\u{f}\u{f}\0\u{1}\0\u{8} 11-&5j0%%f밀%f\r%\u{f}\0\u{f}\u{f}\u{f}\0\u{1}\u{3}\u{8} 11-&5j0\u{f}\u{f}\0\u{4}\u{1}\0\u{f}\u{f}\0\u{f}\u{f}\u{f}\0\u{1}\0\u{8} 11-&5j0\u{f}\u{f}\u{f}\0\u{1}\0\u{8} 11-&5j0%%f밀%f\r%", "Y\0%%f밀\u{8} 11-l5j0c%f밀5j0\u{f}\u{f}11-&5j0%%f밀%f\r%Y\0%%f밀\u{8} 11-Y\0%%f밀\u{8} 11-l5j0c%f밀5j0\u{f}\u{f}11-&5j0%%f밀%f\r%Y\0%%f밀\u{8} 11-l5j0c%f밀%f\r%Y\0%%f밀%f\r%"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA11:05j0%%\u{10}#\0d\u{f}\u{e}\u{c}\u{10}\u{f}\0AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAaaaaaa@aaaaaaaaa%Jڰda@aaaaaaaaa%Jڰd%z%%d\0\0\u{f}\u{f}\u{f}#aaaaaaaaaaaaa@aaaaaaaaa%Jڰda@aaaaaaaaa%Jڰd%AAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", "AAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAaaaaaa@aaaaaaaaa%Jڰda@aaaaaaaaa%Jڰd%z%%d\0\0\u{f}\u{f}\u{f}#aaaaaaaaaaaaa@aaaaaaaaa%Jڰda@aaaaaaaaa%Jڰd%z%%d\0\0\u{f}\u{f}\u{f}#aaaaaaaaaaaa@aaaaaaaaa%Jڰda@aaaaaaaaa%Jڰd%z%%d\0\0\u{f}\u{f}\u{f}#aaaaaaaaaAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA0AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\u{f}\n\u{f}4\u{1d}11-0\u{1b}j0%%%%%d%%d%Ha\u{f0030}$z\u{1d}%Y\0\0"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("aa%JMMMMMMMMMMMMMM000000000004222001Zaaaa@aaaaaaaaaaaa@aaaaaaaaa%a\01 aaaaaaaa@aaaaaaaaa%JMMMMMMMMMMMMMM000000000004222001Zaaaa@aaaaaaaaaaaa@aaaaaaaaa%aa%Jڰd%aanaaaaaaaa@aa\0\u{10}aaaaa%aa%Jڰd%zZaaaa@aaaaaaaaaaaa@aaaaaaa", "aa%aa%Jڰd%aanaaaaaaaa@aa\0\u{10}aaaaa%aa%Jڰd%z%%d\n\n\n\n\n\n\n\n\n\n\n\0\0\n\n\n*\n\n\n\n\n\na%Jڰd%aanaaaaaaaa@aa\0\u{10}aaaaa%aa%Jڰd%zZaaaa@aaaaaaaaaaaa@aaaaaaaaa%aa%Jڰd%aanaaaaaaaa@aa\0\u{10}aaaaa%aa%Jڰd%z%%d\n\n\n\n\n\n\n\n\n\n\n\0\0\n\n\n*\n\n\n\n\n\n\n\n\u{8}\n\n\nA\n\n\nI\n\n%,"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{17}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{13}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}\u{b}BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB", "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBHHHHHHHHHdHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH\u{f}\u{f}\0\0\0\0 \0\00%%d밀%z\r%Y\0\0\0\t\u{4}>\u{f}1\u{1b}9\u{f}\u{f}4\u{1d}11-\0\0\0 \0\0\t\u{16}\t\u{16}(\u{e}\u{f}\u{f}#\0d\u{f}AAAA918199\u{f}\u{f}4\u{1d}11-05j0%%%%H%A05\u{7f}0%%%%H%AA-05j0%%%%H%A05j0%%HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH'HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\0d\u{f}\0\0H%A05\u{7f}0%%%%H%AA-05j0%%%%H%A05j0%%HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB'HHHHHHHHHHHHHHHHHHHHHHHHHHHHH\0\t\u{4}>\u{f}\u{f}MMMM\0\0\u{2}\0"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
    assert_eq!(
        Epoch::from_format_str("MYڰN%z%dd#+.0000042222922222222222223222222222220422229222222222222232222222222222222$4J%%%J%%J%MM2>MMMM02222222%%J%MM2204J>MMMM0222204M.MM0000022022MMM232222222222222222$4J%%%J%%J%MM2>MMMM02222222%%J%MM2204J>MMMM0222204M.MM0000022022MMMMM.Mm$%J%MMMMM/>MMMM02$4J%%%J%%J%MM2>MMMM022222222$4J%%%J%%J%MMjS%%%%%d%%d%wa\u{f0030}\u{f}z\u{1d}%d%wa\u{f0030}\u{f}z\u{1d}%a\u{f0030}\u{f}z\u{1d}%d%wa\u{f0030}MMMM0222204M.MM0000022022MMM", "MM.Mm$%J%MMMMM/>MMMM02$4J%%%J%%J%MM2>MMMM022222222$4J%%%J%%J%MMjS%%%%%d%%d%wa\u{f0030}\u{f}z\u{1d}%d%wa\u{f0030}\u{f}z\u{1d}%a\u{f0030}\u{f}z\u{1d}%d%wa\u{f0030}MM.Mm$%J%MMMMM/>MMMM02$4J%%%J%%J%MM2>MMMM022222222$4J%%%J%%J%MMjS%%%%%d%%d%wa\u{f0030}\u{f}z\u{1d}%d%wa\u{f0030}\u{f}z\u{1d}%a\u{f0030}\u{f}z\u{1d}%d%wa\u{f0030}MMMM0222204M.MM0000022022MMMMM.Mm$%J%MMMMM/>MMMM02$4J%%%J%%J%MM2>MMMM022222222$4J%%%J%%J%M\u{7f}\0\0\u{8}\u{7f}-%%d%%d%wa\u{f0030}\u{f}z\u{1d}%d%wa\u{f0030}\u{f};\u{1d}%a\u{f0030}\u{f}z\u{1d}%d%wa\u{f0030}\u{f}z\u{1d}%Y\0\u{16}\t\u{16}(\t"),
        Err(HifitimeError::Parse {
            source: ParsingError::UnknownFormat,
            details: "when using format string"
        })
    );
}
