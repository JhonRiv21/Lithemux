//! A moment as `2026-11-22T08:00:00Z`, without a date library.

const SECONDS_PER_DAY: i64 = 86_400;

/// `unix` seconds since 1970, in UTC.
pub fn utc_timestamp(unix: i64) -> String {
    let (days, seconds) = (
        unix.div_euclid(SECONDS_PER_DAY),
        unix.rem_euclid(SECONDS_PER_DAY),
    );
    let (year, month, day) = civil_from_days(days);
    let (hour, minute, second) = (seconds / 3600, seconds % 3600 / 60, seconds % 60);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Days since 1970-01-01 to a date of the proleptic Gregorian calendar
/// (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}
