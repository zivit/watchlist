use crate::datetime;
use anyhow::Context;
use chrono::{Datelike, Local, NaiveDateTime, TimeZone};

fn prepare_data_in_anticipation_of_released_episodes(episodes_number: u32) -> (String, [u32; 7]) {
    let release_time = chrono::Local::now() - chrono::Duration::weeks(episodes_number as i64 - 1);
    let mut schedule = [0, 0, 0, 0, 0, 0, 0];
    schedule[release_time.weekday() as usize] = 1;
    (release_time.format("%Y-%m-%d %H:%M").to_string(), schedule)
}

fn prepare_data_with_the_expectation_that_two_episodes_per_week_will_be_released(
    weeks: u32,
) -> (String, [u32; 7]) {
    let release_time = chrono::Local::now() - chrono::Duration::weeks(weeks as i64 - 1);
    let mut schedule = [0, 0, 0, 0, 0, 0, 0];
    schedule[release_time.weekday() as usize] = 1;
    if release_time.weekday() as u32 == 6 {
        schedule[release_time.weekday() as usize] = 2;
    } else {
        schedule[release_time.weekday() as usize + 1] = 1;
    }
    (release_time.format("%Y-%m-%d %H:%M").to_string(), schedule)
}

#[test]
fn check_kaidju_8() {
    let current_episode = 8;
    let _episodes_number = 12;
    let parsed_date = NaiveDateTime::parse_from_str("2024-04-14 06:00", "%Y-%m-%d %H:%M")
        .with_context(|| format!("Failed to parse release time: {}", "2024-04-14 06:00")).unwrap();
    let release_time = match Local.from_local_datetime(&parsed_date) {
        chrono::offset::LocalResult::Single(t) => t,
        chrono::offset::LocalResult::Ambiguous(_, _) => {
            panic!("Failed to convert naive time to local time")
        }
        chrono::offset::LocalResult::None => panic!("Failed to convert naive time to local time"),
    };
    let mut schedule = [0, 0, 0, 0, 0, 0, 0];
    schedule[release_time.weekday() as usize] = 1;
    let data = (release_time.format("%Y-%m-%d %H:%M").to_string(), schedule);
    assert!(datetime::check_new_episodes_available(&data.0, current_episode, data.1).unwrap());
}

#[test]
fn check_new_episodes_available() {
    let current_episode = 11;
    let episodes_number = 12;
    let data = prepare_data_in_anticipation_of_released_episodes(episodes_number);
    assert!(datetime::check_new_episodes_available(&data.0, current_episode, data.1).unwrap());
}

#[test]
fn check_new_episodes_not_available() {
    let current_episode = 12;
    let episodes_number = 12;
    let data = prepare_data_in_anticipation_of_released_episodes(episodes_number);
    assert!(!datetime::check_new_episodes_available(&data.0, current_episode, data.1).unwrap());
}

#[test]
fn check_new_episodes_not_available_with_episode_greather_then_max() {
    let current_episode = 13;
    let episodes_number = 12;
    let data = prepare_data_in_anticipation_of_released_episodes(episodes_number);
    assert!(!datetime::check_new_episodes_available(&data.0, current_episode, data.1).unwrap());
}

#[test]
fn check_new_episodes_available_with_zero_watched() {
    let current_episode = 0;
    let episodes_number = 12;
    let data = prepare_data_in_anticipation_of_released_episodes(episodes_number);
    assert!(datetime::check_new_episodes_available(&data.0, current_episode, data.1).unwrap());
}

#[test]
fn check_new_episodes_available_with_many_episodes() {
    let current_episode = 394;
    let episodes_number = 1200;
    let data = prepare_data_in_anticipation_of_released_episodes(episodes_number);
    assert!(datetime::check_new_episodes_available(&data.0, current_episode, data.1).unwrap());
}

#[test]
fn check_new_episodes_available_2_per_week() {
    let current_episode = 23;
    let weeks_elapsed = 12;
    let data = prepare_data_with_the_expectation_that_two_episodes_per_week_will_be_released(
        weeks_elapsed,
    );
    assert!(datetime::check_new_episodes_available(&data.0, current_episode, data.1).unwrap());
}

#[test]
fn check_new_episodes_not_available_2_per_week() {
    let current_episode = 24;
    let weeks_elapsed = 12;
    let data = prepare_data_with_the_expectation_that_two_episodes_per_week_will_be_released(
        weeks_elapsed,
    );
    assert!(!datetime::check_new_episodes_available(&data.0, current_episode, data.1).unwrap());
}

#[test]
fn check_new_episodes_not_available_2_per_week_with_episode_greather_then_max() {
    let current_episode = 25;
    let weeks_elapsed = 12;
    let data = prepare_data_with_the_expectation_that_two_episodes_per_week_will_be_released(
        weeks_elapsed,
    );
    assert!(!datetime::check_new_episodes_available(&data.0, current_episode, data.1).unwrap());
}
