mod database;
mod datetime;
mod http;
mod parsers;
mod sites;
#[cfg(test)]
mod tests;

use anyhow::Result;
use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Timelike};
use database::*;
use datetime::*;
use http::*;
use sites::*;
use slint::{Model, ModelRc, VecModel};
use std::rc::Rc;

slint::include_modules!();

fn main() -> Result<()> {
    database::create()?;
    let ui = AppWindow::new()?;
    database::load_watchlist(&ui)?;

    ui.on_add_show(|shows, show| match add_show(&show) {
        Ok(_) => {
            let model = shows.as_any().downcast_ref::<VecModel<Show>>();
            if let None = model {
                eprintln!("Failed to downcast watchlist");
                return;
            }
            let model = model.unwrap();

            if show.id == 0 {
                let next_id = if shows.row_count() > 0 {
                    shows.row_data(0).unwrap().id + 1
                } else {
                    1
                };
                let mut show = show.clone();
                show.id = next_id;
                let status = show.status;
                let index = model
                    .iter()
                    .position(|v| v.status.eq(&status))
                    .unwrap_or_default();
                model.insert(index, show);
            } else {
                for i in 0..model.row_count() {
                    let s = model.row_data(i).unwrap();
                    if s.id == show.id {
                        model.set_row_data(i, show.clone());
                        break;
                    }
                }
            }

            let count = model.row_count();
            for i in 0..count {
                let mut s = model.row_data(i).unwrap();
                s.index = i as i32;
                model.set_row_data(i, s);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    });

    ui.on_load_image(|name| -> ImageDetails {
        let name_string = name.to_string();
        let path_to_image = std::path::Path::new(&name_string);
        if name.starts_with("http") {
            let p = download_image_by_http(path_to_image)
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

    ui.on_remove_show(|shows, show| match remove_show(&show) {
        Ok(_) => {
            let model = shows.as_any().downcast_ref::<VecModel<Show>>();
            if let None = model {
                eprintln!("Failed to downcast watchlist");
                return;
            }
            let model = model.unwrap();
            model.remove(show.index as usize);
            for i in 0..model.row_count() {
                let mut s = model.row_data(i).unwrap();
                s.index = i as i32;
                model.set_row_data(i, s);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    });

    ui.on_score_changed(|show| {
        _ = score_changed(&show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_status_changed(|show| {
        _ = status_changed(&show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_favorite_changed(|show| {
        _ = favorite_changed(&show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_season_changed(|show| {
        _ = season_changed(&show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_episode_changed(|show| {
        _ = episode_changed(&show).map_err(|e| eprintln!("Error: {}", e));
    });

    ui.on_check_new_episode_available(|show| -> bool {
        check_new_episodes_available(
            show.release_time.as_str(),
            show.episode as u32,
            [
                show.schedule_monday as u32,
                show.schedule_tuesday as u32,
                show.schedule_wednesday as u32,
                show.schedule_thursday as u32,
                show.schedule_friday as u32,
                show.schedule_saturday as u32,
                show.schedule_sunday as u32,
            ],
        )
        .unwrap_or_default()
    });

    ui.on_open_link(|link| {
        _ = open::that(link.as_str()).map_err(|e| eprintln!("Error: Failed to open URL: {}", e));
    });

    ui.on_search(move |shows, text| -> ModelRc<Show> {
        let filter_model = shows.filter({
            let text = text.clone();
            move |s| {
                if text.is_empty() {
                    true
                } else {
                    s.title
                        .to_lowercase()
                        .contains(text.to_lowercase().as_str())
                        || s.alternative_title
                            .to_lowercase()
                            .contains(text.to_lowercase().as_str())
                }
            }
        });

        let filtered = filter_model.iter().collect::<Vec<Show>>();
        let m = Rc::new(VecModel::from(filtered));
        ModelRc::from(m)
    });

    ui.on_show_filter(move |shows, filter| -> ModelRc<Show> {
        let filter_model = shows.filter({
            let filter = filter.clone();
            move |s| {
                (match filter.status {
                    FilterStatus::All => true,
                    FilterStatus::Watching => s.status == Status::Watching,
                    FilterStatus::Planned => s.status == Status::WatchLater,
                    FilterStatus::Completed => s.status == Status::Completed,
                    FilterStatus::Liked => s.favorite,
                    FilterStatus::Dropped => s.status == Status::Dropped,
                } && match filter.show_type {
                    FilterShowType::All => true,
                    FilterShowType::Serial => s.show_type == ShowType::Serial,
                    FilterShowType::Film => s.show_type == ShowType::Film,
                    FilterShowType::Cartoon => s.show_type == ShowType::Cartoon,
                    FilterShowType::Anime => s.show_type == ShowType::Anime,
                }) // &&
                   // TODO: ongoing and year
                   //
                   // match filter.ongoing {
                   //     FilterOngoing::All => s.
                   //     FilterOngoing::Ongoing =>
                   //     FilterOngoing::Completed =>
                   // }
            }
        });

        let filtered = filter_model.iter().collect::<Vec<Show>>();
        let m = Rc::new(VecModel::from(filtered));
        ModelRc::from(m)
    });

    ui.on_can_import_show_by_link(|link| check_link_is_importable(&link));
    ui.on_import_clicked(|link| import_clicked(&link));

    ui.on_get_weekday_now(|| Local::now().weekday() as i32);

    ui.on_get_weekday(|datetime| {
        let parsed_date =
            NaiveDateTime::parse_from_str(datetime.as_str(), "%Y-%m-%d %H:%M").unwrap_or_default();
        let release_time = match Local.from_local_datetime(&parsed_date) {
            chrono::offset::LocalResult::Single(t) => t,
            chrono::offset::LocalResult::Ambiguous(_, _) => Default::default(),
            chrono::offset::LocalResult::None => Default::default(),
        };
        release_time.weekday() as i32
    });

    ui.on_parse_datetime(|datetime| {
        let parsed_date =
            NaiveDateTime::parse_from_str(datetime.as_str(), "%Y-%m-%d %H:%M").unwrap_or_default();
        let release_time = match Local.from_local_datetime(&parsed_date) {
            chrono::offset::LocalResult::Single(t) => t,
            chrono::offset::LocalResult::Ambiguous(_, _) => Default::default(),
            chrono::offset::LocalResult::None => Default::default(),
        };
        ModelRc::from(Rc::new(VecModel::from(vec![
            release_time.year() as i32,
            release_time.month() as i32,
            release_time.day() as i32,
            release_time.hour() as i32,
            release_time.minute() as i32,
        ])))
    });

    ui.on_get_local_image_path(|| {
        if let Some(image_path) = rfd::FileDialog::new()
            .add_filter("Image files", &["jpg", "jpeg", "png"])
            .pick_file()
        {
            slint::SharedString::from(image_path.to_str().unwrap_or_default())
        } else {
            slint::SharedString::default()
        }
    });

    ui.run()?;
    Ok(())
}
