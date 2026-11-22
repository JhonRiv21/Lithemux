use crate::platform::log::timestamp::utc_timestamp;

// PLAN §3.6: each line of the log has its time.
#[test]
fn the_time_is_written_in_utc_to_the_second() {
    assert_eq!(utc_timestamp(0), "1970-01-01T00:00:00Z");
    assert_eq!(utc_timestamp(1_800_000_000), "2027-01-15T08:00:00Z");
    assert_eq!(utc_timestamp(951_782_400), "2000-02-29T00:00:00Z");
    assert_eq!(utc_timestamp(1_795_910_399), "2026-11-28T23:59:59Z");
}
