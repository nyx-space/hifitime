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
        format!("{:x}", epoch.to_ut1(&provider)),
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
    let provider = Ut1Provider::download_from_jpl("2022/221222_190002-marge_eop2.short").unwrap();

    println!("{}", provider);

    // Grabbed from AstroPy:
    // >>> Time("2022-01-03 03:05:06.789101")
    // <Time object: scale='utc' format='iso' value=2022-01-03 03:05:06.789>
    // >>> Time("2022-01-03 03:05:06.789101").ut1
    // <Time object: scale='ut1' format='iso' value=2022-01-03 03:05:06.679>
    // >>>
    //
    let epoch = Epoch::from_str("2022-01-03 03:05:06.7891 UTC").unwrap();
    let ut1_epoch = epoch.to_ut1(&provider);
    assert_eq!(
        format!("{:x}", ut1_epoch),
        "2022-01-03T03:05:06.679020600 TAI",
    );
}

#[cfg(feature = "ut1")]
mod cache {
    use hifitime::ut1::{Ut1DataSource, Ut1Provider, DEFAULT_UT1_CACHE_MAX_AGE};
    use hifitime::Duration;
    use std::path::PathBuf;

    /// Unique scratch directory for a test, so that tests running in parallel do not share a cache.
    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "hifitime-ut1-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_ut1_cache_reuse_does_not_download() {
        let dir = scratch_dir("reuse");
        let cache_path = dir.join("latest_eop2.short");
        std::fs::copy("data/eop-2021-10-12--2023-01-04.short", &cache_path).unwrap();

        // A fresh cache must be used as is: this would fail on a machine without network access if
        // the implementation tried to download the file.
        let provider = Ut1Provider::download_from_jpl_cached_at(
            "latest_eop2.short",
            &cache_path,
            DEFAULT_UT1_CACHE_MAX_AGE,
        )
        .unwrap();

        let reference =
            Ut1Provider::from_eop_file("data/eop-2021-10-12--2023-01-04.short").unwrap();
        assert_eq!(provider.as_slice().len(), reference.as_slice().len());
        for (got, expected) in provider.as_slice().iter().zip(reference.as_slice()) {
            assert_eq!(got.epoch, expected.epoch);
            assert_eq!(got.delta_tai_minus_ut1, expected.delta_tai_minus_ut1);
        }

        match provider.source() {
            Ut1DataSource::Cache { age } => {
                assert!(
                    age <= DEFAULT_UT1_CACHE_MAX_AGE,
                    "cache age {age:?} is too old"
                )
            }
            other => panic!("expected the data to come from the cache, got {other:?}"),
        }

        // A provider built from a user provided file is not flagged as cached or downloaded.
        assert_eq!(reference.source(), Ut1DataSource::Provided);

        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_ut1_cache_stale_falls_back_to_cache_when_download_fails() {
        let dir = scratch_dir("stale");
        let cache_path = dir.join("latest_eop2.short");
        std::fs::copy("data/eop-2021-10-12--2023-01-04.short", &cache_path).unwrap();

        // A maximum cache age of zero forces a download attempt, and this version does not exist on
        // the JPL server, so the download fails whether or not the machine has network access. The
        // cached file must then be used, with a warning.
        let provider = Ut1Provider::download_from_jpl_cached_at(
            "no-such-version.short",
            &cache_path,
            Duration::ZERO,
        )
        .unwrap();

        assert!(matches!(provider.source(), Ut1DataSource::Cache { .. }));
        assert!(!provider.as_slice().is_empty());

        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_ut1_cache_missing_and_download_failing_is_an_error() {
        let dir = scratch_dir("missing");
        let cache_path = dir.join("latest_eop2.short");

        // No cached file and no way to download one: the download error is returned.
        assert!(
            Ut1Provider::download_from_jpl_cached_at(
                "no-such-version.short",
                &cache_path,
                DEFAULT_UT1_CACHE_MAX_AGE
            )
            .is_err(),
            "a missing cache and a failed download must be an error"
        );

        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_ut1_cache_path_is_sanitized() {
        let path = Ut1Provider::cache_path("2022/221222_190002-marge_eop2.short");

        if let Some(path) = path {
            assert_eq!(
                path.file_name().unwrap(),
                "2022_221222_190002-marge_eop2.short"
            );
        }
    }
}
