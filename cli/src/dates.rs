use chrono::{DateTime, Datelike, FixedOffset, Local};
use parse_datetime::{parse_datetime, ParseDateTimeError};

/// Wrapper for parse_datetime that also allows parses months.
pub fn parse_datetime_extended(input: &str) -> Result<DateTime<FixedOffset>, ParseDateTimeError> {
    let now = Local::now();
    let extended_result = match input.to_lowercase().as_str() {
        "jan" | "january" => format!("{}-01-01", now.year()),
        "feb" | "february" => format!("{}-02-01", now.year()),
        "mar" | "march" => format!("{}-03-01", now.year()),
        "apr" | "april" => format!("{}-04-01", now.year()),
        "may" => format!("{}-05-01", now.year()),
        "jun" | "june" => format!("{}-06-01", now.year()),
        "jul" | "july" => format!("{}-07-01", now.year()),
        "aug" | "august" => format!("{}-08-01", now.year()),
        "sep" | "september" => format!("{}-09-01", now.year()),
        "oct" | "october" => format!("{}-10-01", now.year()),
        "nov" | "november" => format!("{}-11-01", now.year()),
        "dec" | "december" => format!("{}-12-01", now.year()),
        _ => input.to_string(),
    };

    parse_datetime(&extended_result)
}
