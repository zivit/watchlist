use crate::Show;
use anyhow::{Context, Result};
use regex::Regex;
use webpage::{Webpage, WebpageOptions};

#[derive(Default)]
struct ParsedShow {
    title: String,
    alternative_title: String,
    release_date: String,
    about: String,
    link_to_picture: String,
}

fn scrab(
    link: &str,
    title: &str,
    alternative: &str,
    release: &str,
    about: &str,
    image: &str,
) -> Result<Show> {
    let site = Webpage::from_url(link, WebpageOptions::default())
        .with_context(|| format!("Could not read from URL: {}", link))?;
    let doc = site.http.body;

    let mut parsed = ParsedShow::default();

    let re_title = Regex::new(title)?;
    if let Some(captures) = re_title.captures(&doc) {
        if let Some(text) = captures.get(1) {
            parsed.title = text.as_str().into();
        }
    }

    let re_alternative = Regex::new(alternative)?;
    if let Some(captures) = re_alternative.captures(&doc) {
        if let Some(text) = captures.get(1) {
            parsed.alternative_title = text.as_str().into();
        }
    }

    let re_release = Regex::new(release)?;
    if let Some(captures) = re_release.captures(&doc) {
        if let Some(text) = captures.get(1) {
            parsed.release_date = text.as_str().into();
        }
    }

    let re_about = Regex::new(about)?;
    if let Some(captures) = re_about.captures(&doc) {
        if let Some(text) = captures.get(1) {
            parsed.about = text.as_str().into();
        }
    }

    let re_image = Regex::new(image)?;
    if let Some(captures) = re_image.captures(&doc) {
        if let Some(text) = captures.get(1) {
            parsed.link_to_picture = text.as_str().into();
        }
    }

    if parsed.alternative_title.is_empty() {
        parsed.alternative_title = parsed.title.clone();
    }

    Ok(Show {
        title: parsed.title.into(),
        alternative_title: parsed.alternative_title.into(),
        release_date: parsed.release_date.into(),
        about: parsed.about.into(),
        link_to_picture: parsed.link_to_picture.into(),
        link_to_show: link.into(),
        ..Default::default()
    })
}

fn get_show(parsed: Result<Show>) -> Show {
    match parsed {
        Ok(p) => return p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Show::default();
        }
    }
}

pub fn imdb(link: &str) -> Show {
    let parsed = scrab(
        link,
        r#"hero__primary-text">([^<]+)"#,
        r#"Original title: ([^<]+)"#,
        r#"releaseinfo\?ref_=tt_ov_rdat">([^<]+)"#,
        r#"class="sc-7193fc79-2 kpMXpM">([^<]+)"#,
        r#"class="ipc-image" loading="eager" src="([^"]+)"#,
    );

    get_show(parsed)
}
