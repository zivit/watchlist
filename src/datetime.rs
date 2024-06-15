use anyhow::{bail, Ok, Result};
use chrono::prelude::*;

pub fn check_new_episodes_available(
    time: &str,
    current_episode: u32,
    schedule: [u32; 7],
) -> Result<bool> {
    let start_time = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M")?;
    let now = Local::now().naive_local();

    let duration_since_start = now.signed_duration_since(start_time);
    if duration_since_start.num_days() < 0 {
        bail!("Release time is in the future");
    }

    let days_since_start = duration_since_start.num_days() as usize;
    let start_day_of_week = start_time.weekday().num_days_from_monday() as usize;

    let mut total_episodes = 0;
    for i in 0..=days_since_start {
        let day_of_week = (start_day_of_week + i) % 7;
        total_episodes += schedule[day_of_week];
    }

    Ok(total_episodes > current_episode)
}
