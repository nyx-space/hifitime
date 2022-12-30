use hifitime::efmt::consts::RFC3339;
use hifitime::{Epoch, Unit};
use iai::black_box;

fn epoch_from_gregorian_utc() {
    black_box(Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33));
}

fn epoch_from_gregorian_tai() {
    black_box(Epoch::from_gregorian_tai_hms(2015, 2, 7, 11, 22, 33));
}

fn epoch_from_tdb_seconds() {
    black_box(Epoch::from_tdb_seconds(725603206.3990843));
}

fn epoch_jde_tdb_seconds() {
    black_box(Epoch::from_jde_tdb(2459943.1861358252));
}

fn epoch_from_et_seconds() {
    black_box(Epoch::from_et_seconds(725603337.1330023));
}

fn epoch_jde_et_seconds() {
    black_box(Epoch::from_jde_tdb(2459943.186081989));
}

fn epoch_add() {
    let e: Epoch = Epoch::from_gregorian_tai_hms(2015, 2, 7, 11, 22, 33);
    black_box(e + 50 * Unit::Second);
}

fn epoch_sub() {
    let e: Epoch = Epoch::from_gregorian_tai_hms(2015, 2, 7, 11, 22, 33);
    black_box(e - 50 * Unit::Second);
}

fn parse_rfc3339_with_seconds() {
    black_box(Epoch::from_gregorian_str("2018-02-13T23:08:32Z").unwrap());
}

fn parse_rfc3339_with_milliseconds() {
    black_box(Epoch::from_gregorian_str("2018-02-13T23:08:32.123Z").unwrap());
}

fn parse_rfc3339_with_nanoseconds() {
    black_box(Epoch::from_gregorian_str("2018-02-13T23:08:32.123456983Z").unwrap());
}

fn fmt_parse_rfc3339_with_seconds() {
    black_box(RFC3339.parse("2018-02-13T23:08:32Z").unwrap());
}

fn fmt_parse_rfc3339_with_milliseconds() {
    black_box(RFC3339.parse("2018-02-13T23:08:32.123Z").unwrap());
}

fn fmt_parse_rfc3339_with_nanoseconds() {
    black_box(RFC3339.parse("2018-02-13T23:08:32.123456983Z").unwrap());
}

iai::main!(
    epoch_from_gregorian_utc,
    epoch_from_gregorian_tai,
    epoch_from_tdb_seconds,
    epoch_jde_tdb_seconds,
    epoch_from_et_seconds,
    epoch_jde_et_seconds,
    epoch_add,
    epoch_sub,
    parse_rfc3339_with_seconds,
    parse_rfc3339_with_milliseconds,
    parse_rfc3339_with_nanoseconds,
    fmt_parse_rfc3339_with_seconds,
    fmt_parse_rfc3339_with_milliseconds,
    fmt_parse_rfc3339_with_nanoseconds
);
