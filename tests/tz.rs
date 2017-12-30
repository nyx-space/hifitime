extern crate hifitime;

use hifitime::utc::{Utc, TimeSystem, TimeZone, Offset};
use hifitime::instant::Era;
use hifitime::Errors;
use std::fmt;

struct KiloTZ {
    base_utc: Utc,
}

impl TimeZone for KiloTZ
where
    Self: Sized,
{
    /// Returns the offset between this TimeZone and UTC. In this case, the offset is 10 hours.
    /// cf. https://en.wikipedia.org/wiki/List_of_military_time_zones
    fn utc_offset() -> Offset {
        Offset::new(10 * 3600, 0, Era::Present)
    }

    fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanos: u32,
    ) -> Result<KiloTZ, Errors> {
        // Check this date can exist in UTC (otherw it's invalid regardless of the timezone)
        let init_utc = Utc::new(year, month, day, hour, minute, second, nanos)?;
        // Perform the time zone correction and store the UTC value.
        let base_utc = Utc::from_instant(init_utc.as_instant() - Self::utc_offset().duration());
        Ok(KiloTZ { base_utc: base_utc })
    }
}

impl fmt::Display for KiloTZ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Kinda hacky but it works. I might change this in future implementations.
        let tz_corrected =
            Utc::from_instant(self.base_utc.as_instant() + Self::utc_offset().duration());
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}+10:00",
            tz_corrected.year,
            tz_corrected.month,
            tz_corrected.day,
            tz_corrected.hour,
            tz_corrected.minute,
            tz_corrected.second
        )
    }
}

#[test]
fn kilotz() {
    let santa_ktz = KiloTZ::new(2017, 12, 25, 00, 00, 00, 00).expect("Santa failed");
    assert_eq!(format!("{}", santa_ktz), "2017-12-25T00:00:00+10:00");
}
