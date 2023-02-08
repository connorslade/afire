use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    middleware::{MiddleResult, Middleware},
    Header, HeaderType, Request, Response,
};

const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Middleware to add the HTTP Date header (as defined in [RFC 9110, Section 5.6.7](https://www.rfc-editor.org/rfc/rfc9110.html#section-5.6.7)).
/// This is technically required for all servers that have a clock, so I may move it to the core library at some point.
///
/// # Example
/// ```rust
/// # use afire::{extension::Date, Middleware};
/// # fn add(mut server: afire::Server) {
/// Date.attach(&mut server);
/// # }
pub struct Date;

impl Middleware for Date {
    fn post(&self, _req: &Request, res: &mut Response) -> MiddleResult {
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards. Make sure your date is set correctly.")
            .as_secs();

        res.headers
            .push(Header::new(HeaderType::Date, imp_date(epoch)));
        MiddleResult::Continue
    }
}

/// Returns the number of days in a month.
/// Month is 1-indexed.
fn days_in_month(month: u8, year: u16) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 => 29,
        2 => 28,
        _ => unreachable!("Invalid month: {}", month),
    }
}

/// Returns the current date in the IMF-fixdate format.
/// Example: `Sun, 06 Nov 1994 08:49:37 GMT`
fn imp_date(epoch: u64) -> String {
    let seconds = epoch % 60;
    let minutes = (epoch / 60) % 60;
    let hours = (epoch / 3600) % 24;
    let mut days = (epoch / 86400) as u16;
    let weekday = (days + 4) % 7;

    let mut year = 1970;
    let mut month = 1;
    while days >= days_in_month(month, year) as u16 {
        days -= days_in_month(month, year) as u16;
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
    }

    format!(
        "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
        DAYS[weekday as usize],
        days + 1,
        MONTHS[month as usize - 1],
        year,
        hours,
        minutes,
        seconds
    )
}

#[cfg(test)]
mod test {
    use super::imp_date;

    #[test]
    fn test_epoch() {
        assert_eq!(imp_date(0), "Thu, 01 Jan 1970 00:00:00 GMT");
        assert_eq!(imp_date(123456), "Fri, 02 Jan 1970 10:17:36 GMT");
        assert_eq!(imp_date(1675899597), "Wed, 08 Feb 2023 23:39:57 GMT");
    }
}
