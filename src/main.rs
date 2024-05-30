use chrono::prelude::*;

use anyhow::{Context, Result};
use downloader::{Download, Downloader};
use image::EncodableLayout;
use slint::{ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};
use sqlite::State;
use std::{fs::File, io::Read, rc::Rc};

slint::include_modules!();

mod parsers;

const DATABASE_NAME: &str = "watchlist.db";
const SITE_IMDB: &str = "https://www.imdb.com";

#[derive(PartialEq)]
enum Sites {
    Imdb,
    Unknown,
}

fn create_database() -> Result<()> {
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

fn check_new_episodes_available(time: &str, episode: i32, schedule: [i32; 7]) -> Result<bool> {
    let parsed_date = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M")
        .with_context(|| format!("Failed to parse release time: {}", time))?;
    let release_time: DateTime<Local> = Local.from_local_datetime(&parsed_date).unwrap();
    let release_weekday = release_time.weekday() as i32;
    let time_duration = Local::now().signed_duration_since(release_time);
    let weeks_count = time_duration.num_weeks();
    let episodes_elapsed = schedule.iter().sum::<i32>() * weeks_count as i32;
    let weeks_elapsed = release_time + chrono::Duration::weeks(weeks_count);
    let time_duration = Local::now().signed_duration_since(weeks_elapsed);
    let episodes_elapsed_second_part = schedule.iter()
        .cycle()
        .skip(release_weekday as usize)
        .take(time_duration.num_days() as usize + 1)
        .sum::<i32>();
    println!("Elapsed: {}, current: {}", episodes_elapsed + episodes_elapsed_second_part, episode);
    Ok(episodes_elapsed + episodes_elapsed_second_part > episode)
}

fn execute_query(ui: &AppWindow, query: &str) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let mut statement = connection.prepare(query)?;
    let mut model = Vec::new();

    while let Ok(State::Row) = statement.next() {
        let picture_blob = statement.read::<Vec<u8>, _>("image");
        let picture;
        if let Ok(content) = picture_blob {
            if content.is_empty() {
                picture = slint::Image::default();
            } else {
                let picture_image = image::load_from_memory(content.as_bytes())
                    .context("Failed to load picture from memory")?
                    .into_rgba8();
                let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                    picture_image.as_raw(),
                    picture_image.width(),
                    picture_image.height(),
                );
                picture = slint::Image::from_rgba8(buffer);
            }
        } else {
            picture = slint::Image::default();
        }

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
        let new_episodes_available = check_new_episodes_available(&release_time, episode,
            [schedule_monday, schedule_tuesday, schedule_wednesday, schedule_thursday,
            schedule_friday, schedule_saturday, schedule_sunday])
            .unwrap_or_default();
            // .map_or_else(|e| { eprintln!("Error: {}", e); false }, |v| v);

        let show = Show {
            id: statement.read::<i64, _>("id")? as i32,
            title: statement.read::<String, _>("title")?.into(),
            alternative_title: statement.read::<String, _>("alternative_title")?.into(),
            release_date: statement.read::<String, _>("release_date")?.into(),
            about: statement.read::<String, _>("about")?.into(),
            link_to_show: statement.read::<String, _>("link_to_show")?.into(),
            score: statement.read::<i64, _>("score")? as i32,
            favorite: statement.read::<String, _>("favorite")?.eq("true"),
            status,
            picture,
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
    }

    let shows: Rc<VecModel<Show>> = Rc::new(VecModel::from(model));
    ui.set_shows(ModelRc::from(shows));
    Ok(())
}

fn display_all_watchlist(ui: &AppWindow) -> Result<()> {
    let query = "SELECT * FROM list
        ORDER BY
            CASE status
                WHEN 1 THEN 0
                WHEN 0 THEN 1
                WHEN 2 THEN 2
                WHEN 3 THEN 3
                ELSE 4
            END;
    ";
    execute_query(ui, query)
}

fn display_list_of_shows_found(ui: &AppWindow, text: &str) -> Result<()> {
    let query = format!(
        "SELECT * FROM list
            WHERE
            LOWER(title) LIKE \"%{0}%\" OR
            LOWER(alternative_title) LIKE \"%{0}%\";",
        text);
    execute_query(ui, &query)
}

fn add_show(s: Show) -> Result<()> {
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
            s.title.to_string(),
            s.alternative_title.to_string(),
            s.release_date.to_string(),
            s.about.to_string(),
            s.link_to_show.to_string(),
            s.score.to_string(),
            s.favorite.to_string(),
            status,
            s.season,
            s.episodes_count,
            s.episode,
            s.release_time.to_string(),
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
            s.title.to_string(),
            s.alternative_title.to_string(),
            s.release_date.to_string(),
            s.about.to_string(),
            s.link_to_show.to_string(),
            s.score.to_string(),
            s.favorite.to_string(),
            status,
            s.season,
            s.episodes_count,
            s.episode,
            s.release_time.to_string(),
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
        let query = format!(
            "UPDATE list SET image = ? WHERE title = \"{}\";",
            s.title.to_string()
        );

        let mut content = Vec::new();
        let st = if !s.link_to_picture.is_empty() {
            let mut file = File::open(&s.link_to_picture.to_string()).expect("Failed to open file");
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

fn download_image_by_http(url: &std::path::Path) -> Result<std::path::PathBuf> {
    let mut p = std::env::temp_dir();
    p.push("watchlist");
    std::fs::create_dir(&p).unwrap_or_default();

    let mut d = Downloader::builder()
        .download_folder(&p)
        .parallel_requests(1)
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()?;
    let dl = Download::new(url.to_str().unwrap_or_default());
    let result = d.download(&[dl]).unwrap_or_default();

    for r in result {
        match r {
            Err(e) => print!("Error: {}", e.to_string()),
            Ok(s) => print!("Success: {}", &s),
        }
    }

    Ok(std::path::PathBuf::from(
        &p.join(url.file_name().unwrap_or_default()),
    ))
}

fn check_link_is_allowed_site(link: &str) -> Sites {
    if link.starts_with(SITE_IMDB) {
        return Sites::Imdb;
    } else {
        return Sites::Unknown;
    }
}

fn check_link_is_importable(link: &str) -> bool {
    if link.is_empty() {
        return false;
    }

    let site = check_link_is_allowed_site(link);
    if site == Sites::Unknown {
        return false;
    } else {
        return true;
    }
}

fn import_clicked(link: &str) -> Show {
    let site = check_link_is_allowed_site(link);
    if site == Sites::Imdb {
        return parsers::imdb(link);
    }
    Show::default()
}

fn remove_show(show: Show) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!("DELETE FROM list WHERE title = \"{}\";", show.title);
    connection
        .execute(query)
        .with_context(|| format!("Failed to delete show with title \"{}\"", show.title))?;
    Ok(())
}

fn score_changed(show: Show) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!(
        "UPDATE list SET score = \"{}\" WHERE id = \"{}\";",
        show.score,
        show.id,
    );
    connection
        .execute(query)
        .with_context(|| format!("Failed to change score to {}", show.score))?;
    Ok(())
}

fn status_changed(show: Show) -> Result<()> {
    let status = match show.status {
        Status::WatchLater => 0,
        Status::Watching => 1,
        Status::Completed => 2,
        Status::Dropped => 3,
    };

    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!(
        "UPDATE list SET status = \"{}\" WHERE id = \"{}\";",
        status,
        show.id,
    );
    connection
        .execute(query)
        .with_context(|| format!("Failed to change status to {}", show.score))?;
    Ok(())
}

fn favorite_changed(show: Show) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!(
        "UPDATE list SET favorite = \"{}\" WHERE id = \"{}\";",
        show.favorite.to_string(),
        show.id,
    );
    connection
        .execute(query)
        .with_context(|| format!("Failed to change favorite to {}", show.score))?;
    Ok(())
}

fn season_changed(show: Show) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!(
        "UPDATE list SET season = \"{}\" WHERE id = \"{}\";",
        show.season.to_string(),
        show.id,
    );
    connection
        .execute(query)
        .with_context(|| format!("Failed to change season to {}", show.score))?;
    Ok(())
}

fn episode_changed(show: Show) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
    let query = format!(
        "UPDATE list SET episode = \"{}\" WHERE id = \"{}\";",
        show.episode.to_string(),
        show.id,
    );
    connection
        .execute(query)
        .with_context(|| format!("Failed to change episode to {}", show.score))?;
    Ok(())
}

fn main() -> Result<()> {
    create_database()?;
    let ui = AppWindow::new()?;
    display_all_watchlist(&ui)?;

    ui.on_add_show(|show| {
        let _ = add_show(show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_load_image(|name| -> ImageDetails {
        let name_string = name.to_string();
        let path_to_image = std::path::Path::new(&name_string);
        if name.starts_with("http") {
            let p = download_image_by_http(&path_to_image)
                .map_err(|e| eprintln!("Error: {}", e))
                .unwrap_or_default();
            ImageDetails {
                source: slint::Image::load_from_path(&p).unwrap_or_default(),
                path: slint::SharedString::from(p.to_str().unwrap_or_default()),
            }
        } else {
            ImageDetails {
                source: slint::Image::load_from_path(path_to_image).unwrap_or_default(),
                path: slint::SharedString::from(name_string),
            }
        }
    });

    ui.on_remove_show(|show| {
        let _ = remove_show(show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_score_changed(|show| {
        let _ = score_changed(show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_status_changed(|show| {
        let _ = status_changed(show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_favorite_changed(|show| {
        let _ = favorite_changed(show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_score_changed(|show| {
        let _ = season_changed(show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_episode_changed(|show| {
        let _ = episode_changed(show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_open_link(|link| {
        let _ = open::that(link.as_str()).map_err(|e| eprintln!("Error: Failed to open URL: {}", e));
    });

    let ui_weak = ui.as_weak();
    ui.on_search(move |text| {
        let ui = ui_weak.unwrap();
        let _ = display_list_of_shows_found(&ui, &text).map_err(|e| eprintln!("Error: {}", e));
    });

    let ui_weak = ui.as_weak();
    ui.on_update_watchlist(move || {
        let ui = ui_weak.unwrap();
        display_all_watchlist(&ui).unwrap();
    });

    ui.on_can_import_show_by_link(|link| check_link_is_importable(&link));
    ui.on_import_clicked(|link| import_clicked(&link));
    ui.run()?;
    Ok(())
}
