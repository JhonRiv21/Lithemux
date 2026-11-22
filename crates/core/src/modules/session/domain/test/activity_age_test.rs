use crate::modules::session::domain::activity_age::ActivityAge;

const NOW: i64 = 1_800_000_000;

fn age(seconds_ago: i64) -> ActivityAge {
    ActivityAge::between(NOW, NOW - seconds_ago)
}

// PLAN §0.5 example ("hace 5 min") + §0.6 status: the unit grows with the age
#[test]
fn the_age_is_told_in_the_largest_unit_that_fits() {
    assert_eq!(age(0), ActivityAge::JustNow);
    assert_eq!(age(59), ActivityAge::JustNow);
    assert_eq!(age(60), ActivityAge::Minutes(1));
    assert_eq!(age(59 * 60 + 59), ActivityAge::Minutes(59));
    assert_eq!(age(3600), ActivityAge::Hours(1));
    assert_eq!(age(86_399), ActivityAge::Hours(23));
    assert_eq!(age(86_400), ActivityAge::Days(1));
    assert_eq!(age(3 * 86_400 + 5), ActivityAge::Days(3));
}

// A clock that moved back must not show a negative age
#[test]
fn activity_in_the_future_counts_as_just_now() {
    assert_eq!(age(-120), ActivityAge::JustNow);
}

// PLAN §0.6 status: the wording people read
#[test]
fn the_english_wording_has_a_singular_day() {
    assert_eq!(age(10).to_string(), "just now");
    assert_eq!(age(300).to_string(), "5 min ago");
    assert_eq!(age(7200).to_string(), "2 h ago");
    assert_eq!(age(86_400).to_string(), "1 day ago");
    assert_eq!(age(2 * 86_400).to_string(), "2 days ago");
}
