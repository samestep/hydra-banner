use bytesize::ByteSize;
use chrono::DateTime;
use std::time::Duration;

pub fn format_timestamp(ts: i64) -> String {
    DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| ts.to_string())
}

pub fn format_duration(secs: i64) -> String {
    humantime::format_duration(Duration::from_secs(secs.unsigned_abs())).to_string()
}

pub fn format_size(bytes: u64) -> String {
    ByteSize(bytes).to_string_as(false).replace(' ', "")
}
