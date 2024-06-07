use anyhow::{bail, Context, Ok, Result};
use chrono::prelude::*;

pub fn check_new_episodes_available(time: &str, episode: u32, schedule: [u32; 7]) -> Result<bool> {
    let parsed_date = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M")
        .with_context(|| format!("Failed to parse release time: {}", time))?;
    let release_time = match Local.from_local_datetime(&parsed_date) {
        chrono::offset::LocalResult::Single(t) => t,
        chrono::offset::LocalResult::Ambiguous(_, _) => {
            bail!("Failed to convert naive time to local time")
        }
        chrono::offset::LocalResult::None => bail!("Failed to convert naive time to local time"),
    };
    let time_now = Local::now();
    let time_elapsed = time_now.signed_duration_since(release_time);
    if time_elapsed.num_weeks() < 0 {
        bail!("Release time is in the future");
    }

    let weeks_count = time_elapsed.num_weeks();
    let episodes_per_week = schedule
        .iter()
        .sum::<u32>();
    let episodes_elapsed = episodes_per_week * weeks_count as u32;
    let episodes_elapsed_this_week: u32 = schedule
        .iter()
        .enumerate()
        .filter(|(i, d)| **d > 0 && time_now.weekday() as usize > *i)
        .map(|(_, d)| *d)
        .sum();

    Ok(episodes_elapsed + episodes_elapsed_this_week > episode)
}

