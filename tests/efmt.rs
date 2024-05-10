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
        Err(EpochError::Parse {
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
