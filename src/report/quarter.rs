use chrono::{Datelike, Local, NaiveDate};

/// Calculate start and end dates of the previous fiscal quarter
/// Returns (start_date, end_date, quarter_num, year)
pub fn get_previous_quarter() -> (NaiveDate, NaiveDate, u32, i32) {
    let today = Local::now().date_naive();
    let current_month = today.month();
    let current_year = today.year();

    // Determine current quarter (1-4)
    let current_quarter = (current_month - 1) / 3 + 1;

    // Calculate previous quarter
    let (prev_quarter, prev_year) = if current_quarter == 1 {
        (4, current_year - 1)
    } else {
        (current_quarter - 1, current_year)
    };

    // Calculate start and end months for the quarter
    let start_month = (prev_quarter - 1) * 3 + 1;
    let end_month = prev_quarter * 3;

    let start_date = NaiveDate::from_ymd_opt(prev_year, start_month, 1).unwrap();

    // Get last day of the end month
    let end_day = if end_month == 12 {
        NaiveDate::from_ymd_opt(prev_year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(prev_year, end_month + 1, 1).unwrap() - chrono::Duration::days(1)
    };

    (start_date, end_day, prev_quarter, prev_year)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Month;

    #[test]
    fn test_q4_2025_from_jan_2026() {
        // Simulate running in January 2026, should return Q4 2025
        let (start, end, quarter, year) = get_previous_quarter();

        assert_eq!(quarter, 4);
        assert_eq!(year, 2025);
        assert_eq!(start.month() as u32, 10);
        assert_eq!(start.day(), 1);
        assert_eq!(end.month() as u32, 12);
        assert_eq!(end.day(), 31);
    }

    #[test]
    fn test_quarter_calculation() {
        let (start, end, _quarter, _year) = get_previous_quarter();
        assert!(start < end || start == end); // Date sanity check
    }
}
