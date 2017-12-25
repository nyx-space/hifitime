// Part of hifitime.
// TODO: Add LICENSE.txt and README.md

//! hifitime 0.0.1
//!
//! Precise date and time handling in Rust. Epoch is 01 Jan 1900 at midnight.
//!
//! Features:
//!
//! * Leap seconds (as announced by the IETF on a yearly basis)
//! * Julian dates and Modified Julian dates
//! * UTC representation with ISO8601 formatting
//! * Time varying `TimeZone`s to represent very high speed reference frames
//!
//! Most (all?) examples are validated with external references, as detailed on a test-by-test
//! basis.
//!
//! Does not includes:
//! * Dates only, or times only (i.e. handles only the combination of both)
//! * Custom formatting of date time objects (for now)
//!
//! Examples:
//! ```
//! use hifitime::utc::{Utc, TimeZone, TimeSystem};
//! use hifitime::instant::Duration;
//! use hifitime::julian::ModifiedJulian;
//!
//! let santa = Utc::new(2017, 12, 25, 01, 02, 14, 0).expect("Xmas failed");
//!
//! assert_eq!(
//!     santa.as_instant() + Duration::new(3600, 0),
//!     Utc::new(2017, 12, 25, 02, 02, 14, 0)
//!         .expect("Xmas failed")
//!         .as_instant(),
//!     "Could not add one hour to Christmas"
//! );
//! assert_eq!(format!("{}", santa), "2017-12-25T01:02:14+00:00");
//! assert_eq!(
//!     ModifiedJulian::from_instant(santa.as_instant()).days,
//!     58112.043217592596
//! );
//! assert_eq!(
//!     ModifiedJulian::from_instant(santa.as_instant()).julian_days(),
//!     2458112.5432175924
//! );
//! ```
//!

pub mod utils;
pub mod traits;
pub mod instant;
pub mod julian;
pub mod utc;
