/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

extern crate der;

use super::{Duration, Epoch, TimeSystem};

use self::der::{Decode, Encode, Reader, Writer};

use core::convert::From;

impl Epoch {
    /// Enables the encoding of an epoch into ASN1 provided the time system to use for serialization.
    /// Epoch will always be serialized in a high fidelity duration.
    ///
    /// ## Layout
    /// Once encoded, the data is stored as follows, where the upper column represents the length in octets.
    /// ```text
    ///
    /// |  1  |  1  |     1     |  1  |  1  |  11   |
    /// | TAG | LEN | CENTURIES | TAG | LEN | NANOS |
    ///
    /// ```
    ///
    /// ## Size
    /// Although the data itself fits in 11 octets (1 for the time system and 10 for the duration),
    /// ASN1 is a variable size encoding with tags. For an epoch that's +/-7 centuries around its
    /// reference epoch, the ASN1 DER encoding will take up 16 octets. For durations further away
    /// from the reference epoch, expect 17 octets on the wire.
    pub fn to_asn1(&self, time_system: TimeSystem) -> Asn1Epoch {
        let duration = match time_system {
            TimeSystem::ET => self.as_et_duration(),
            TimeSystem::TAI => self.0,
            TimeSystem::TT => self.as_tt_duration(),
            TimeSystem::TDB => self.as_tdb_duration(),
            TimeSystem::UTC => self.as_utc_duration(),
        };

        Asn1Epoch {
            duration,
            time_system,
        }
    }
}

// TODO: Make two serializable epochs. One for high precision and one for float.
// The structure should preferrably not be public so that the function becomes Epoch.serialize::<{Duration, f64}>() and Epoch::deserialize::<'a>(&'a [u8]).
// The Duration should be serializable itself both in high precision and float of seconds. Should there be float of days?!

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Asn1Epoch {
    pub(crate) duration: Duration,
    pub(crate) time_system: TimeSystem,
}

impl From<Asn1Epoch> for Epoch {
    fn from(epoch: Asn1Epoch) -> Self {
        match epoch.time_system {
            TimeSystem::ET => Self::from_et_duration(epoch.duration),
            TimeSystem::TAI => Self(epoch.duration),
            TimeSystem::TT => Self::from_tt_duration(epoch.duration),
            TimeSystem::TDB => Self::from_tdb_duration(epoch.duration),
            TimeSystem::UTC => Self::from_utc_seconds(epoch.duration.in_seconds()),
        }
    }
}

impl Default for Asn1Epoch {
    fn default() -> Self {
        Self {
            duration: Duration::default(),
            time_system: TimeSystem::TAI,
        }
    }
}

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
impl Encode for Asn1Epoch {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let ts: u8 = self.time_system.into();
        ts.encoded_len()? + self.duration.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let ts: u8 = self.time_system.into();

        ts.encode(encoder)?;
        self.duration.encode(encoder)
    }
}

impl<'a> Decode<'a> for Asn1Epoch {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let ts: u8 = decoder.decode()?;
        let duration = decoder.decode()?;
        let time_system: TimeSystem = TimeSystem::from(ts);
        Ok(Self {
            duration,
            time_system,
        })
    }
}

// Testing the encoding and decoding of an Epoch inherently also tests the encoding and decoding of a Duration
#[test]
fn test_encdec() {
    let epoch = Epoch::from_gregorian_utc_hms(2022, 9, 6, 23, 24, 29);

    for ts_u8 in 0..5 {
        let ts: TimeSystem = ts_u8.into();
        let duration = match ts {
            TimeSystem::ET => epoch.as_et_duration(),
            TimeSystem::TAI => epoch.as_tai_duration(),
            TimeSystem::TT => epoch.as_tt_duration(),
            TimeSystem::TDB => epoch.as_tdb_duration(),
            TimeSystem::UTC => epoch.as_utc_duration(),
        };

        // Create a buffer
        let mut buf = [0_u8; 16];
        // Encode
        epoch.to_asn1(ts).encode_to_slice(&mut buf).unwrap();
        // Decode
        let asn1_epoch = Asn1Epoch::from_der(&buf).unwrap();
        assert_eq!(asn1_epoch.duration, duration, "Decoded duration incorrect");
        assert_eq!(asn1_epoch.time_system, ts, "Decoded time system incorrect");
        // Convert into Epoch
        let epoch_out: Epoch = asn1_epoch.into();
        assert_eq!(epoch_out, epoch);
    }
}
