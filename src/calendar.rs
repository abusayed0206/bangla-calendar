// calendar.rs - Bangla calendar calculation

use crate::constants::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// Convert English number to Bangla numerals
pub fn to_bangla_number(num: i32) -> String {
    num.to_string()
        .chars()
        .map(|c| {
            if let Some(digit) = c.to_digit(10) {
                BANGLA_DIGITS[digit as usize]
            } else {
                c
            }
        })
        .collect()
}

/// Bangla date structure
#[derive(Debug, Clone)]
pub struct BanglaDate {
    pub day: i32,
    pub month: i32,
    pub year: i32,
    pub weekday: i32,
}

impl BanglaDate {
    pub fn get_ordinal(&self) -> &'static str {
        if self.day >= 1 && self.day <= 31 {
            BANGLA_ORDINALS[self.day as usize]
        } else {
            "?"
        }
    }

    pub fn get_month_name(&self) -> &'static str {
        if self.month >= 0 && self.month < 12 {
            BANGLA_MONTHS[self.month as usize]
        } else {
            "?"
        }
    }

    pub fn get_season(&self) -> &'static str {
        if self.month >= 0 && self.month < 12 {
            BANGLA_SEASONS[self.month as usize]
        } else {
            "?"
        }
    }

    pub fn get_weekday_name(&self) -> &'static str {
        if self.weekday >= 0 && self.weekday < 7 {
            BANGLA_WEEKDAYS[self.weekday as usize]
        } else {
            "?"
        }
    }

    pub fn get_year_bangla(&self) -> String {
        to_bangla_number(self.year)
    }

    /// Line 1: ০৬ই পৌষ,
    pub fn format_line1(&self) -> String {
        format!("{} {},", self.get_ordinal(), self.get_month_name())
    }

    /// Line 2: ১৪৩২ বঙ্গাব্দ
    pub fn format_line2(&self) -> String {
        format!("{} বঙ্গাব্দ", self.get_year_bangla())
    }

    /// Line 3: শনিবার, হেমন্তকাল
    pub fn format_line3(&self) -> String {
        format!("{}, {}কাল", self.get_weekday_name(), self.get_season())
    }
}

const BD_MONTH_DAYS: &[i32] = &[31, 31, 31, 31, 31, 30, 30, 30, 30, 30, 30, 30];

fn is_gregorian_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn calculate_bangla_date_bd(
    gregorian_year: i32,
    gregorian_month: i32,
    gregorian_day: i32,
) -> BanglaDate {
    let greg_month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let is_leap = is_gregorian_leap_year(gregorian_year);
    let prev_leap = is_gregorian_leap_year(gregorian_year - 1);

    let mut day_of_year = gregorian_day;
    for m in 0..(gregorian_month - 1) {
        day_of_year += greg_month_days[m as usize];
        if m == 1 && is_leap {
            day_of_year += 1;
        }
    }

    let boishakh_start = if prev_leap { 105 } else { 104 };

    let bangla_year;
    let day_in_bangla_year;

    if day_of_year >= boishakh_start {
        bangla_year = gregorian_year - 593;
        day_in_bangla_year = day_of_year - boishakh_start + 1;
    } else {
        bangla_year = gregorian_year - 594;
        let prev_year_days = if prev_leap { 366 } else { 365 };
        day_in_bangla_year = (prev_year_days - boishakh_start + 1) + day_of_year;
    }

    let mut remaining_days = day_in_bangla_year;
    let mut bangla_month = 0;

    let falgun_days = if is_gregorian_leap_year(bangla_year + 594) {
        31
    } else {
        30
    };

    for m in 0..12 {
        let month_days = if m == 10 {
            falgun_days
        } else {
            BD_MONTH_DAYS[m]
        };
        if remaining_days <= month_days {
            bangla_month = m as i32;
            break;
        }
        remaining_days -= month_days;
    }

    let bangla_day = remaining_days;
    let weekday = calculate_weekday(gregorian_year, gregorian_month, gregorian_day);

    BanglaDate {
        day: bangla_day,
        month: bangla_month,
        year: bangla_year,
        weekday,
    }
}

fn calculate_weekday(year: i32, month: i32, day: i32) -> i32 {
    let mut y = year;
    let mut m = month;

    if m < 3 {
        m += 12;
        y -= 1;
    }

    let k = y % 100;
    let j = y / 100;

    let h = (day + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
    (h + 6) % 7
}

pub fn get_current_bangla_date() -> BanglaDate {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let bd_timestamp = now + (6 * 3600);
    let adjusted_timestamp = bd_timestamp - (5 * 3600);
    let (year, month, day) = timestamp_to_gregorian(adjusted_timestamp);

    calculate_bangla_date_bd(year, month, day)
}

fn timestamp_to_gregorian(timestamp: i64) -> (i32, i32, i32) {
    let mut days = (timestamp / 86400) as i32;
    let mut year = 1970;

    loop {
        let days_in_year = if is_gregorian_leap_year(year) {
            366
        } else {
            365
        };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let month_days = if is_gregorian_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for (i, &md) in month_days.iter().enumerate() {
        if days < md {
            month = (i + 1) as i32;
            break;
        }
        days -= md;
    }

    let day = days + 1;
    (year, month, day)
}
