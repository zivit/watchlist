use chrono::Datelike;

use crate::datetime;

#[test]
fn check_new_episodes_available() {
    let release_time = chrono::Local::now() - chrono::Duration::weeks(12);
    let current_episode = 11;
    let mut schedule = [0, 0, 0, 0, 0, 0, 0];
    schedule[release_time.weekday() as usize] = 1;
    let time = release_time.format("%Y-%m-%d %H:%M").to_string();

    assert!(datetime::check_new_episodes_available(&time, current_episode, schedule).unwrap());
}
