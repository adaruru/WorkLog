use std::collections::{HashMap, HashSet};

use chrono::{Datelike, Duration, NaiveDate, Weekday};

#[derive(Debug, Clone, Copy)]
pub struct SprintConfig {
    pub anchor_number: i64,
    pub anchor_start: NaiveDate,
    pub length_days: i64,
}

impl SprintConfig {
    pub fn sprint_of(&self, date: NaiveDate) -> i64 {
        let elapsed = (date - self.anchor_start).num_days();
        self.anchor_number + elapsed.div_euclid(self.length_days)
    }

    pub fn range_of(&self, number: i64) -> (NaiveDate, NaiveDate) {
        let offset = (number - self.anchor_number) * self.length_days;
        let start = self.anchor_start + Duration::days(offset);
        let end = start + Duration::days(self.length_days - 1);
        (start, end)
    }
}

pub fn week_range(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let offset = date.weekday().num_days_from_monday() as i64;
    let start = date - Duration::days(offset);
    (start, start + Duration::days(6))
}

pub fn month_range(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let start = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date);
    let end = if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
    }
    .map(|next| next - Duration::days(1))
    .unwrap_or(date);
    (start, end)
}

#[derive(Debug, Clone, Default)]
pub struct Calendar {
    pub holidays: HashSet<NaiveDate>,
    pub extra_workdays: HashSet<NaiveDate>,
}

impl Calendar {
    pub fn is_workday(&self, date: NaiveDate) -> bool {
        if self.extra_workdays.contains(&date) {
            return true;
        }
        if self.holidays.contains(&date) {
            return false;
        }
        !matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
    }

    pub fn required_hours(
        &self,
        date: NaiveDate,
        daily_required: f64,
        leaves: &HashMap<NaiveDate, f64>,
    ) -> f64 {
        if !self.is_workday(date) {
            return 0.0;
        }
        let leave = leaves.get(&date).copied().unwrap_or(0.0);
        (daily_required - leave).max(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoursSummary {
    pub required: f64,
    pub filled: f64,
    pub diff: f64,
}

pub fn summarize(
    from: NaiveDate,
    to: NaiveDate,
    daily_required: f64,
    calendar: &Calendar,
    leaves: &HashMap<NaiveDate, f64>,
    filled: &HashMap<NaiveDate, f64>,
) -> HoursSummary {
    let mut required_total = 0.0;
    let mut filled_total = 0.0;
    let mut cursor = from;
    while cursor <= to {
        required_total += calendar.required_hours(cursor, daily_required, leaves);
        filled_total += filled.get(&cursor).copied().unwrap_or(0.0);
        cursor += Duration::days(1);
    }
    HoursSummary {
        required: required_total,
        filled: filled_total,
        diff: filled_total - required_total,
    }
}

pub fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(value: &str) -> NaiveDate {
        parse_date(value).unwrap()
    }

    fn config() -> SprintConfig {
        SprintConfig {
            anchor_number: 12,
            anchor_start: date("2026-09-01"),
            length_days: 14,
        }
    }

    #[test]
    fn sprint_of_anchor_day_is_anchor_number() {
        assert_eq!(config().sprint_of(date("2026-09-01")), 12);
    }

    #[test]
    fn sprint_of_last_day_stays_in_same_sprint() {
        assert_eq!(config().sprint_of(date("2026-09-14")), 12);
    }

    #[test]
    fn sprint_of_next_day_rolls_over() {
        assert_eq!(config().sprint_of(date("2026-09-15")), 13);
    }

    #[test]
    fn sprint_of_date_before_anchor_goes_backwards() {
        assert_eq!(config().sprint_of(date("2026-08-31")), 11);
        assert_eq!(config().sprint_of(date("2026-08-19")), 11);
        assert_eq!(config().sprint_of(date("2026-08-18")), 11);
        assert_eq!(config().sprint_of(date("2026-08-17")), 10);
        assert_eq!(config().sprint_of(date("2026-08-04")), 10);
    }

    #[test]
    fn range_of_returns_inclusive_bounds() {
        let (start, end) = config().range_of(13);
        assert_eq!(start, date("2026-09-15"));
        assert_eq!(end, date("2026-09-28"));
    }

    #[test]
    fn sprint_of_and_range_of_are_consistent() {
        let cfg = config();
        for number in [9, 10, 12, 15, 30] {
            let (start, end) = cfg.range_of(number);
            assert_eq!(cfg.sprint_of(start), number);
            assert_eq!(cfg.sprint_of(end), number);
        }
    }

    #[test]
    fn week_range_runs_monday_to_sunday() {
        let (start, end) = week_range(date("2026-09-18"));
        assert_eq!(start, date("2026-09-14"));
        assert_eq!(end, date("2026-09-20"));
    }

    #[test]
    fn week_range_on_sunday_keeps_that_week() {
        let (start, end) = week_range(date("2026-09-20"));
        assert_eq!(start, date("2026-09-14"));
        assert_eq!(end, date("2026-09-20"));
    }

    #[test]
    fn month_range_covers_whole_month() {
        let (start, end) = month_range(date("2026-02-10"));
        assert_eq!(start, date("2026-02-01"));
        assert_eq!(end, date("2026-02-28"));
    }

    #[test]
    fn month_range_handles_december() {
        let (start, end) = month_range(date("2026-12-05"));
        assert_eq!(start, date("2026-12-01"));
        assert_eq!(end, date("2026-12-31"));
    }

    fn cal(holidays: &[&str], workdays: &[&str]) -> Calendar {
        Calendar {
            holidays: holidays.iter().map(|value| date(value)).collect(),
            extra_workdays: workdays.iter().map(|value| date(value)).collect(),
        }
    }

    #[test]
    fn weekend_is_not_a_workday() {
        let calendar = Calendar::default();
        assert!(!calendar.is_workday(date("2026-09-19")));
        assert!(!calendar.is_workday(date("2026-09-20")));
        assert!(calendar.is_workday(date("2026-09-18")));
    }

    #[test]
    fn holiday_is_not_a_workday() {
        let calendar = cal(&["2026-09-18"], &[]);
        assert!(!calendar.is_workday(date("2026-09-18")));
    }

    #[test]
    fn make_up_day_turns_a_weekend_into_a_workday() {
        let calendar = cal(&[], &["2025-02-08"]);
        assert!(calendar.is_workday(date("2025-02-08")));
    }

    #[test]
    fn make_up_day_wins_over_holiday_on_the_same_date() {
        let calendar = cal(&["2025-02-08"], &["2025-02-08"]);
        assert!(calendar.is_workday(date("2025-02-08")));
    }

    #[test]
    fn required_hours_is_zero_on_weekend() {
        let required = Calendar::default().required_hours(date("2026-09-19"), 8.0, &HashMap::new());
        assert_eq!(required, 0.0);
    }

    #[test]
    fn make_up_day_requires_a_full_day() {
        let calendar = cal(&[], &["2025-02-08"]);
        let required = calendar.required_hours(date("2025-02-08"), 8.0, &HashMap::new());
        assert_eq!(required, 8.0);
    }

    #[test]
    fn leave_reduces_required_hours() {
        let leaves = HashMap::from([(date("2026-09-18"), 4.0)]);
        let required = Calendar::default().required_hours(date("2026-09-18"), 8.0, &leaves);
        assert_eq!(required, 4.0);
    }

    #[test]
    fn leave_longer_than_required_clamps_to_zero() {
        let leaves = HashMap::from([(date("2026-09-18"), 10.0)]);
        let required = Calendar::default().required_hours(date("2026-09-18"), 8.0, &leaves);
        assert_eq!(required, 0.0);
    }

    #[test]
    fn leave_on_a_make_up_day_is_deducted() {
        let calendar = cal(&[], &["2025-02-08"]);
        let leaves = HashMap::from([(date("2025-02-08"), 4.0)]);
        let required = calendar.required_hours(date("2025-02-08"), 8.0, &leaves);
        assert_eq!(required, 4.0);
    }

    #[test]
    fn summarize_reports_shortfall() {
        let filled = HashMap::from([(date("2026-09-14"), 8.0), (date("2026-09-15"), 3.5)]);
        let summary = summarize(
            date("2026-09-14"),
            date("2026-09-18"),
            8.0,
            &Calendar::default(),
            &HashMap::new(),
            &filled,
        );
        assert_eq!(summary.required, 40.0);
        assert_eq!(summary.filled, 11.5);
        assert_eq!(summary.diff, -28.5);
    }

    #[test]
    fn summarize_reports_exact_match() {
        let filled = HashMap::from([(date("2026-09-14"), 8.0), (date("2026-09-15"), 8.0)]);
        let summary = summarize(
            date("2026-09-14"),
            date("2026-09-15"),
            8.0,
            &Calendar::default(),
            &HashMap::new(),
            &filled,
        );
        assert_eq!(summary.diff, 0.0);
    }

    #[test]
    fn summarize_reports_overtime() {
        let filled = HashMap::from([(date("2026-09-14"), 11.0)]);
        let summary = summarize(
            date("2026-09-14"),
            date("2026-09-14"),
            8.0,
            &Calendar::default(),
            &HashMap::new(),
            &filled,
        );
        assert_eq!(summary.diff, 3.0);
    }

    #[test]
    fn summarize_skips_weekend_and_holiday() {
        let summary = summarize(
            date("2026-09-14"),
            date("2026-09-20"),
            8.0,
            &cal(&["2026-09-18"], &[]),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(summary.required, 32.0);
    }

    #[test]
    fn summarize_counts_hours_logged_on_a_holiday() {
        let filled = HashMap::from([(date("2026-09-18"), 5.0)]);
        let summary = summarize(
            date("2026-09-18"),
            date("2026-09-18"),
            8.0,
            &cal(&["2026-09-18"], &[]),
            &HashMap::new(),
            &filled,
        );
        assert_eq!(summary.required, 0.0);
        assert_eq!(summary.filled, 5.0);
        assert_eq!(summary.diff, 5.0);
    }
}
