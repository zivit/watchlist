use anyhow::{bail, Context, Result};
use chrono::prelude::*;

pub fn check_new_episodes_available(time: &str, episode: i32, schedule: [i32; 7]) -> Result<bool> {
    let parsed_date = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M")
        .with_context(|| format!("Failed to parse release time: {}", time))?;
    let release_time = match Local.from_local_datetime(&parsed_date) {
        chrono::offset::LocalResult::Single(t) => t,
        chrono::offset::LocalResult::Ambiguous(_, _) => {
            bail!("Failed to convert naive time to local time")
        }
        chrono::offset::LocalResult::None => bail!("Failed to convert naive time to local time"),
    };
    let time_elapsed = Local::now().signed_duration_since(release_time);
    let weeks_count = time_elapsed.num_weeks();
    let episodes_elapsed = schedule.iter().sum::<i32>() * weeks_count as i32 + 1;
    let weeks_elapsed =
        release_time + chrono::Duration::weeks(weeks_count) + chrono::Duration::days(1);
    let day_of_last_elapsed_episode = weeks_elapsed.weekday();
    let time_elapsed = Local::now().signed_duration_since(weeks_elapsed);
    let episodes_elapsed_second_part = schedule
        .iter()
        .cycle()
        .take(time_elapsed.num_days() as usize)
        .skip(day_of_last_elapsed_episode as usize)
        .sum::<i32>();
    Ok(episodes_elapsed + episodes_elapsed_second_part > episode)
}

