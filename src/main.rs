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
                     image BLOB
                 );
    ";
    connection
        .execute(query)
        .context("Failed to create table")?;
    Ok(())
}

fn display_all_watchlist(ui: &AppWindow) -> Result<()> {
    let connection = sqlite::open(DATABASE_NAME).context("Failed to open database")?;
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
            ..Default::default()
        };
        model.push(show);
    }

    let shows: Rc<VecModel<Show>> = Rc::new(VecModel::from(model));
    ui.set_shows(ModelRc::from(shows));
    Ok(())
}

fn add_show(s: Show) -> Result<()> {
    let status = match s.status {
        Status::WatchLater => 0,
        Status::Watching => 1,
        Status::Completed => 2,
        Status::Dropped => 3,
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
                status = \"{}\"
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
            s.id,
        );
        connection.execute(query).context("Failed to add show")?;
    } else {
        let query = format!(
            "REPLACE INTO list(title, alternative_title, release_date,
                             about, link_to_show, score, favorite, status) VALUES
                             (\"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\");",
            s.title.to_string(),
            s.alternative_title.to_string(),
            s.release_date.to_string(),
            s.about.to_string(),
            s.link_to_show.to_string(),
            s.score.to_string(),
            s.favorite.to_string(),
            status,
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
    println!("favorite: {}", show.favorite);
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
