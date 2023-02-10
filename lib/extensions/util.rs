//! Misc utilities that may be helpful when making an website.
//! These things are all here because I ended up copy-pasting them between projects, and decided to put them all together in a crate.

const TIME_UNITS: &[(&str, u16)] = &[
    ("second", 60),
    ("minute", 60),
    ("hour", 24),
    ("day", 30),
    ("month", 12),
    ("year", 0),
];

/// Turn relative number of seconds into a more readable relative time.
/// If the time is 0, now will be returned.
/// Works with the following units:
/// - Seconds
/// - Minutes
/// - Hours
/// - Days
/// - Months
/// - Years
///
/// Ex 1 minute ago or 3 years ago
pub fn fmt_relative_time(secs: u64) -> String {
    if secs == 0 {
        return "now".into();
    }

    let mut secs = secs as f64;
    for i in TIME_UNITS {
        if i.1 == 0 || secs < i.1 as f64 {
            secs = secs.round();
            return format!("{} {}{} ago", secs, i.0, if secs > 1.0 { "s" } else { "" });
        }

        secs /= i.1 as f64;
    }

    format!("{} years ago", secs.round())
}

#[cfg(test)]
mod test {
    use super::fmt_relative_time;

    #[test]
    fn test_fmt_relative_time() {
        assert_eq!(fmt_relative_time(0), "now");
        assert_eq!(fmt_relative_time(315_569_520), "10 years ago");
        assert_eq!(fmt_relative_time(20), "20 seconds ago");
        assert_eq!(fmt_relative_time(60), "1 minute ago");
        assert_eq!(fmt_relative_time(120), "2 minutes ago");
    }
}
