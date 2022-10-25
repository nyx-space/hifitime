/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::Unit;

use super::{Duration, Epoch, TimeScale};

use der::{Decode, Encode, Reader, Writer};

/// A duration is encoded with the centuries first followed by the nanoseconds.
impl Encode for Duration {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let (centuries, nanoseconds) = self.to_parts();
        centuries.encoded_len()? + nanoseconds.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let (centuries, nanoseconds) = self.to_parts();
        centuries.encode(encoder)?;
        nanoseconds.encode(encoder)
    }
}

impl<'a> Decode<'a> for Duration {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let centuries = decoder.decode()?;
        let nanoseconds = decoder.decode()?;
        Ok(Duration::from_parts(centuries, nanoseconds))
    }
}

/// An Epoch is encoded with time system first followed by the duration structure.
impl Encode for Epoch {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let ts: u8 = self.time_scale.into();
        ts.encoded_len()? + self.to_duration().encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let ts: u8 = self.time_scale.into();

        ts.encode(encoder)?;
        self.to_duration().encode(encoder)
    }
}

impl<'a> Decode<'a> for Epoch {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let ts: u8 = decoder.decode()?;
        let duration = decoder.decode()?;
        let time_scale: TimeScale = TimeScale::from(ts);
        Ok(match time_scale {
            TimeScale::TAI => Self::from_tai_duration(duration),
            TimeScale::TT => Self::from_tt_duration(duration),
            TimeScale::ET => Self::from_et_duration(duration),
            TimeScale::TDB => Self::from_tdb_duration(duration),
            TimeScale::UTC => Self::from_utc_duration(duration),
            TimeScale::GPST => Self::from_gpst_duration(duration),
            TimeScale::GST => Self::from_gst_duration(duration),
            TimeScale::BDT => Self::from_bdt_duration(duration),
        })
    }
}

/// An Epoch is encoded with time system first followed by the duration structure.
impl Encode for Unit {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let converted: u8 = self.into();
        converted.encoded_len()
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let converted: u8 = self.into();
        converted.encode(encoder)
    }
}

impl<'a> Decode<'a> for Unit {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let converted: u8 = decoder.decode()?;
        Ok(Self::from(converted))
    }
}

// Testing the encoding and decoding of an Epoch inherently also tests the encoding and decoding of a Duration
#[test]
fn test_encdec() {
    for ts_u8 in 0..=7 {
        let ts: TimeScale = ts_u8.into();

        let epoch = if ts == TimeScale::UTC {
            Epoch::from_gregorian_utc_hms(2022, 9, 6, 23, 24, 29)
        } else {
            Epoch::from_gregorian_hms(2022, 9, 6, 23, 24, 29, ts)
        };

        let duration = match ts {
            TimeScale::TAI => epoch.to_tai_duration(),
            TimeScale::ET => epoch.to_et_duration(),
            TimeScale::TT => epoch.to_tt_duration(),
            TimeScale::TDB => epoch.to_tdb_duration(),
            TimeScale::UTC => epoch.to_utc_duration(),
            TimeScale::GPST => epoch.to_gpst_duration(),
            TimeScale::GST => epoch.to_gst_duration(),
            TimeScale::BDT => epoch.to_bdt_duration(),
        };

        let e_dur = epoch.to_duration();

        assert_eq!(e_dur, duration, "{ts:?}");

        // Create a buffer
        let mut buf = [0_u8; 16];
        // Encode
        epoch.encode_to_slice(&mut buf).unwrap();
        // Decode
        let encdec_epoch = Epoch::from_der(&buf).unwrap();
        // Check that the duration in J1900 TAI is the same
        assert_eq!(
            encdec_epoch.duration_since_j1900_tai, epoch.duration_since_j1900_tai,
            "Decoded epoch incorrect ({ts:?}):\ngot: {encdec_epoch}\nexp: {epoch}",
        );
        // Check that the time scale used is preserved
        assert_eq!(
            encdec_epoch.time_scale, ts,
            "Decoded time system incorrect {ts:?}"
        );
    }

    for unit_u8 in 0..=7 {
        let unit: Unit = unit_u8.into();

        // Create a buffer
        let mut buf = [0_u8; 3];
        // Encode
        unit.encode_to_slice(&mut buf).unwrap();
        // Decode
        let encdec_unit = Unit::from_der(&buf).unwrap();

        assert_eq!(
            encdec_unit, unit,
            "Decoded epoch incorrect ({unit:?}):\ngot: {encdec_unit:?}\nexp: {unit:?}",
        );
    }
}
