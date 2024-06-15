use std::{fs::File, io::Read, rc::Rc, sync::{Arc, Mutex}, thread};

use crate::{datetime::*, AppWindow, Show, ShowType, Status};
use anyhow::{Context, Result};
use image::EncodableLayout;
use slint::{ComponentHandle, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};
use sqlite::State;

pub const DATABASE_NAME: &str = "watchlist.db";

pub fn create() -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = "CREATE TABLE IF NOT EXISTS list (
                     id INTEGER PRIMARY KEY AUTOINCREMENT,
                     title TEXT NOT NULL UNIQUE,
                     alternative_title TEXT,
                     release_date TEXT,
                     about TEXT,
                     link_to_show TEXT,
                     score INTEGER,
                     favorite BOOL,
                     status INTEGER,
                     image BLOB,
                     show_type INTEGER,
                     season INTEGER,
                     episodes_count INTEGER,
                     episode INTEGER,
                     release_time TEXT,
                     schedule_monday INTEGER,
                     schedule_tuesday INTEGER,
                     schedule_wednesday INTEGER,
                     schedule_thursday INTEGER,
                     schedule_friday INTEGER,
                     schedule_saturday INTEGER,
                     schedule_sunday INTEGER
                 );";
    connection
        .execute(query)
        .context("Failed to create table")?;

    Ok(())
}

fn rows_count() -> Result<u32> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let mut statement = connection.prepare("SELECT COUNT(*) FROM list;")?;
    statement.next()?;
    let count: i64 = statement.read::<i64, _>(0)?;
    Ok(count as u32)
}

fn execute_query(query: &str) -> Result<ModelRc<Show>> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let mut statement = connection.prepare(query)?;
    let mut model = Vec::new();
    let mut index = 0;

    while let Ok(State::Row) = statement.next() {
        let st = statement.read::<i64, _>("status")?;
        let status = match st {
            1 => Status::Watching,
            2 => Status::Completed,
            3 => Status::Dropped,
            _ => Status::WatchLater,
        };
        let sht = statement.read::<i64, _>("show_type")?;
        let show_type = match sht {
            1 => ShowType::Film,
            2 => ShowType::Cartoon,
            3 => ShowType::Anime,
            _ => ShowType::Serial,
        };

        let episode = statement.read::<i64, _>("episode")? as i32;
        let episodes_count = statement.read::<i64, _>("episodes_count")? as i32;
        let schedule_monday = statement.read::<i64, _>("schedule_monday")? as i32;
        let schedule_tuesday = statement.read::<i64, _>("schedule_tuesday")? as i32;
        let schedule_wednesday = statement.read::<i64, _>("schedule_wednesday")? as i32;
        let schedule_thursday = statement.read::<i64, _>("schedule_thursday")? as i32;
        let schedule_friday = statement.read::<i64, _>("schedule_friday")? as i32;
        let schedule_saturday = statement.read::<i64, _>("schedule_saturday")? as i32;
        let schedule_sunday = statement.read::<i64, _>("schedule_sunday")? as i32;
        let release_time = statement.read::<String, _>("release_time")?;
        let new_episodes_available = check_new_episodes_available(
            &release_time,
            episode as u32,
            [
                schedule_monday as u32,
                schedule_tuesday as u32,
                schedule_wednesday as u32,
                schedule_thursday as u32,
                schedule_friday as u32,
                schedule_saturday as u32,
                schedule_sunday as u32,
            ],
        )
        .unwrap_or_default();

        let show = Show {
            id: statement.read::<i64, _>("id")? as i32,
            index,
            title: statement.read::<String, _>("title")?.into(),
            alternative_title: statement.read::<String, _>("alternative_title")?.into(),
            release_date: statement.read::<String, _>("release_date")?.into(),
            about: statement.read::<String, _>("about")?.into(),
            link_to_show: statement.read::<String, _>("link_to_show")?.into(),
            score: statement.read::<i64, _>("score")? as i32,
            favorite: statement.read::<String, _>("favorite")?.eq("true"),
            status,
            show_type,
            season: statement.read::<i64, _>("season")? as i32,
            episodes_count,
            episode,
            release_time: release_time.into(),
            schedule_monday,
            schedule_tuesday,
            schedule_wednesday,
            schedule_thursday,
            schedule_friday,
            schedule_saturday,
            schedule_sunday,
            new_episodes_available,
            ..Default::default()
        };
        model.push(show);
        index += 1;
    }

    let shows: Rc<VecModel<Show>> = Rc::new(VecModel::from(model));
    Ok(ModelRc::from(shows))
}

fn load_images(ui: slint::Weak<AppWindow>) -> Result<()> {
    let query = "SELECT image FROM list
        ORDER BY
            CASE status
                WHEN 1 THEN 0
                WHEN 0 THEN 1
                WHEN 2 THEN 2
                WHEN 3 THEN 3
                ELSE 4
            END,
            id DESC;
    ";
    let rows_number = rows_count()?;

    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let mut statement = connection.prepare(query)?;
    let model = Arc::new(Mutex::new(Vec::new()));
    let mut index = 0;

    while let Ok(State::Row) = statement.next() {
        let picture_blob = statement.read::<Vec<u8>, _>("image");
        if let Ok(content) = picture_blob {
            if content.is_empty() {
                model.lock().unwrap().push(None);
            } else {
                let picture_image = image::load_from_memory(content.as_bytes())
                    .expect("Failed to load picture from memory")
                    .into_rgba8();
                let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                    picture_image.as_raw(),
                    picture_image.width(),
                    picture_image.height(),
                );
                model.lock().unwrap().push(Some(buffer));
            }
        } else {
            model.lock().unwrap().push(None);
        }

        index += 1;
        let loading_progress = index as f32 / rows_number as f32 * 100.0;
        let ui_clone = ui.clone();
        _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = ui_clone.upgrade() {
                app.set_loading_progress(loading_progress as i32);
            }
        });
    }

    let ui_clone = ui.clone();
    _ = slint::invoke_from_event_loop(move || {
        let model = model.lock().unwrap();
        if let Some(app) = ui_clone.upgrade() {
            let shows = app.get_shows();
            for i in 0..shows.row_count() {
                if let Some(buffer) = model[i].clone() {
                    let picture = slint::Image::from_rgba8(buffer);
                    let mut s = shows.row_data(i).unwrap();
                    s.picture = picture;
                    shows.set_row_data(i, s);
                }
            }
            app.set_shows(shows);
        }
    });

    Ok(())
}

pub fn load_watchlist(ui: &AppWindow) -> Result<()> {
    let query = "SELECT * FROM list
        ORDER BY
            CASE status
                WHEN 1 THEN 0
                WHEN 0 THEN 1
                WHEN 2 THEN 2
                WHEN 3 THEN 3
                ELSE 4
            END,
            id DESC;
    ";
    let shows = execute_query(query)?;
    ui.invoke_set_shows(shows);

    let ui_weak = ui.as_weak();
    thread::spawn(move || {
        _ = load_images(ui_weak).map_err(|e| eprintln!("Failed to load images: {e}"));
    });
    Ok(())
}

pub fn add_show(s: &Show) -> Result<()> {
    let status = match s.status {
        Status::WatchLater => 0,
        Status::Watching => 1,
        Status::Completed => 2,
        Status::Dropped => 3,
    };

    let show_type = match s.show_type {
        ShowType::Serial => 0,
        ShowType::Film => 1,
        ShowType::Cartoon => 2,
        ShowType::Anime => 3,
    };

    let title = s.title.to_string().replace('"', "“");
    let alternative_title = s.alternative_title.to_string().replace('"', "“");
    let about = s.about.to_string().replace('"', "“");

    let connection = sqlite::open(DATABASE_NAME).expect("Failed to connect to database");

    if s.id != 0 {
        let query = format!(
            "UPDATE list SET
                title = \"{}\",
                alternative_title = \"{}\",
                release_date = \"{}\",
                about = \"{}\",
                link_to_show = \"{}\",
                score = \"{}\",
                favorite = \"{}\",
                status = \"{}\",
                season = \"{}\",
                episodes_count = \"{}\",
                episode = \"{}\",
                release_time = \"{}\",
                schedule_monday = \"{}\",
                schedule_tuesday = \"{}\",
                schedule_wednesday = \"{}\",
                schedule_thursday = \"{}\",
                schedule_friday = \"{}\",
                schedule_saturday = \"{}\",
                schedule_sunday = \"{}\",
                show_type = \"{}\"
            WHERE id = {};
            ",
            title,
            alternative_title,
            s.release_date,
            about,
            s.link_to_show,
            s.score,
            s.favorite,
            status,
            s.season,
            s.episodes_count,
            s.episode,
            s.release_time,
            s.schedule_monday,
            s.schedule_tuesday,
            s.schedule_wednesday,
            s.schedule_thursday,
            s.schedule_friday,
            s.schedule_saturday,
            s.schedule_sunday,
            show_type,
            s.id,
        );
        connection.execute(query).context("Failed to update show")?;
    } else {
        let query = format!(
            "REPLACE INTO list(title, alternative_title, release_date, about, link_to_show,
                            score, favorite, status, season, episodes_count, episode, release_time,
                            schedule_monday, schedule_tuesday, schedule_wednesday,
                            schedule_thursday, schedule_friday, schedule_saturday,
                            schedule_sunday, show_type) VALUES
                            (\"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\",
                            \"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\",
                            \"{}\", \"{}\", \"{}\", \"{}\");",
            title,
            alternative_title,
            s.release_date,
            about,
            s.link_to_show,
            s.score,
            s.favorite,
            status,
            s.season,
            s.episodes_count,
            s.episode,
            s.release_time,
            s.schedule_monday,
            s.schedule_tuesday,
            s.schedule_wednesday,
            s.schedule_thursday,
            s.schedule_friday,
            s.schedule_saturday,
            s.schedule_sunday,
            show_type,
        );
        connection.execute(query).context("Failed to add show")?;
    }

    if !s.link_to_picture.is_empty() {
        let query = format!("UPDATE list SET image = ? WHERE title = \"{}\";", s.title);

        let mut content = Vec::new();
        let st = if !s.link_to_picture.is_empty() {
            let mut file = File::open(s.link_to_picture.to_string()).expect("Failed to open file");
            file.read_to_end(&mut content)
                .expect("Failed to read_to_end");
            unsafe { std::str::from_utf8_unchecked(&content) }
        } else {
            ""
        };

        let mut statement = connection.prepare(query).unwrap();
        statement.bind((1, st)).unwrap();
        if let Ok(sqlite::State::Row) = statement.next() {}
    }

    Ok(())
}

pub fn remove_show(show: &Show) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!("DELETE FROM list WHERE title = \"{}\";", show.title);
    connection
        .execute(query)
        .with_context(|| format!("Failed to delete show with title \"{}\"", show.title))?;
    Ok(())
}

pub fn score_changed(show: &Show) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!(
        "UPDATE list SET score = \"{}\" WHERE id = \"{}\";",
        show.score, show.id,
    );
    connection
        .execute(query)
        .with_context(|| format!("Failed to change score to {}", show.score))?;
    Ok(())
}

pub fn status_changed(show: &Show) -> Result<()> {
    let status = match show.status {
        Status::WatchLater => 0,
        Status::Watching => 1,
        Status::Completed => 2,
        Status::Dropped => 3,
    };

    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!(
        "UPDATE list SET status = \"{}\" WHERE id = \"{}\";",
        status, show.id,
    );
    connection
        .execute(query)
        .with_context(|| format!("Failed to change status to {}", show.score))?;
    Ok(())
}

pub fn favorite_changed(show: &Show) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!(
        "UPDATE list SET favorite = \"{}\" WHERE id = \"{}\";",
        show.favorite, show.id,
    );
    connection
        .execute(query)
        .with_context(|| format!("Failed to change favorite to {}", show.score))?;
    Ok(())
}

pub fn season_changed(show: &Show) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!(
        "UPDATE list SET season = \"{}\" WHERE id = \"{}\";",
        show.season, show.id,
    );
    connection
        .execute(query)
        .with_context(|| format!("Failed to change season to {}", show.score))?;
    Ok(())
}

pub fn episode_changed(show: &Show) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!(
        "UPDATE list SET episode = \"{}\" WHERE id = \"{}\";",
        show.episode, show.id,
    );
    connection
        .execute(query)
        .with_context(|| format!("Failed to change episode to {}", show.score))?;
    Ok(())
}
