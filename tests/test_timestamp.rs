use chrono::{Datelike, TimeZone, Timelike, Utc};
use scheduler::utils::get_start_timestamp_from_string;

#[test]
fn test_get_start_timestamp_from_string_0() {
    assert_eq!(
        get_start_timestamp_from_string("2024-12-01T00:01:12+0000"),
        Some(Utc.with_ymd_and_hms(2024, 12, 1, 0, 1, 12).unwrap())
    )
}

#[test]
fn test_get_start_timestamp_from_string_1() {
    assert_eq!(
        get_start_timestamp_from_string("20*4-12-01T00:01:12+0000"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_2() {
    assert_eq!(
        get_start_timestamp_from_string("2014-12-01Y00:01:12+0000"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_3() {
    assert_eq!(
        get_start_timestamp_from_string("2014/12/01 00-01-12+0000"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_4() {
    assert_eq!(
        get_start_timestamp_from_string("2024-12-01T01:00:12+0100"),
        Some(Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 12).unwrap())
    )
}

#[test]
fn test_get_start_timestamp_from_string_5() {
    assert_eq!(
        get_start_timestamp_from_string("2024-12-01T00:01:12Z"),
        Some(Utc.with_ymd_and_hms(2024, 12, 1, 0, 1, 12).unwrap())
    )
}

#[test]
fn test_get_start_timestamp_from_string_6() {
    assert_eq!(
        get_start_timestamp_from_string("2024-12-01T01:00:12-0100"),
        Some(Utc.with_ymd_and_hms(2024, 12, 1, 2, 0, 12).unwrap())
    )
}

#[test]
fn test_get_start_timestamp_from_string_7() {
    assert_eq!(
        get_start_timestamp_from_string("20*4-12-01T01:00:12-0100"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_8() {
    assert_eq!(
        get_start_timestamp_from_string("2024-*2-01T01:00:12-0100"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_9() {
    assert_eq!(
        get_start_timestamp_from_string("2014-12-*1T01:00:12-0100"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_10() {
    assert_eq!(
        get_start_timestamp_from_string("2024-12-01T0*:00:12-0100"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_11() {
    assert_eq!(
        get_start_timestamp_from_string("2024-12-01T01:0*:12-0100"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_12() {
    assert_eq!(
        get_start_timestamp_from_string("2024-12-01T01:00:*2-0100"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_13() {
    assert_eq!(
        get_start_timestamp_from_string("2014-12-01T01:00:12-0*00"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_14() {
    assert_eq!(
        get_start_timestamp_from_string("202d-12-01T01:00:12-01*0"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_15() {
    assert_eq!(
        get_start_timestamp_from_string("2014-12-01T01:00:12+0*:00"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_16() {
    assert_eq!(
        get_start_timestamp_from_string("2024-12-01T01:00:12+01:*0"),
        None
    )
}

#[test]
fn test_get_start_timestamp_from_string_17() {
    assert_eq!(
        get_start_timestamp_from_string("2024-12-01T02:00:12+02:00"),
        Some(Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 12).unwrap())
    )
}

#[test]
fn test_get_start_timestamp_from_string_18() {
    assert_eq!(
        get_start_timestamp_from_string("2024-12-01T02:00:12-02:00"),
        Some(Utc.with_ymd_and_hms(2024, 12, 1, 4, 0, 12).unwrap())
    )
}

#[test]
fn test_get_start_timestamp_from_string_19() {
    let now = chrono::Utc::now();
    assert_eq!(
        get_start_timestamp_from_string("****-12-01T02:00:12Z"),
        Some(Utc.with_ymd_and_hms(now.year(), 12, 1, 2, 0, 12).unwrap())
    )
}

#[test]
fn test_get_start_timestamp_from_string_20() {
    let now = chrono::Utc::now();
    assert_eq!(
        get_start_timestamp_from_string("2024-**-01T02:00:12Z"),
        Some(Utc.with_ymd_and_hms(2024, now.month(), 1, 2, 0, 12).unwrap())
    )
}

#[test]
fn test_get_start_timestamp_from_string_21() {
    let now = chrono::Utc::now();
    assert_eq!(
        get_start_timestamp_from_string("****-**-**T**:**:**Z"),
        // I have to do this to get rid of sub seconds
        Some(Utc.with_ymd_and_hms(
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        ).unwrap())
    )
}

#[test]
fn test_get_start_timestamp_from_string_22() {
    let now = chrono::Utc::now();
    assert_eq!(
        get_start_timestamp_from_string("****-12-01T02:00:12-02:00"),
        Some(Utc.with_ymd_and_hms(now.year(), 12, 1, 4, 0, 12).unwrap())
    )
}

#[test]
fn test_get_start_timestamp_from_string_23() {
    let now = chrono::Utc::now();
    assert_eq!(
        get_start_timestamp_from_string("****-12-01T00:00:12-12:34"),
        Some(Utc.with_ymd_and_hms(now.year(), 12, 1, 12, 34, 12).unwrap())
    )
}

#[test]
fn test_get_start_timestamp_from_string_24() {
    let now = chrono::Utc::now();
    assert_eq!(
        get_start_timestamp_from_string("****-12-01T00:00:12-1234"),
        Some(Utc.with_ymd_and_hms(now.year(), 12, 1, 12, 34, 12).unwrap())
    )
}
