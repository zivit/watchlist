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

fn replace_html_entities(text: &str) -> String {
    let mut result = text.to_owned();
    result = result.replace("&#x27;", "'");
    result = result.replace("&#039;", "'");
    result = result.replace("&quot;", "\"");
    result = result.replace("&amp;", "&");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&ndash;", "-");
    result
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
            // parsed.title = text.as_str().into();
            parsed.title = replace_html_entities(text.as_str());
        }
    }

    let re_alternative = Regex::new(alternative)?;
    if let Some(captures) = re_alternative.captures(&doc) {
        if let Some(text) = captures.get(1) {
            parsed.alternative_title = text.as_str().into();
            parsed.alternative_title = parsed.alternative_title.replace("&#x27;", "'");
            parsed.alternative_title = parsed.alternative_title.replace("&#039;", "'");
            parsed.alternative_title = parsed.alternative_title.replace("&quot;", "\"");
            parsed.alternative_title = parsed.alternative_title.replace("&amp;", "&");
            parsed.alternative_title = parsed.alternative_title.replace("&lt;", "<");
            parsed.alternative_title = parsed.alternative_title.replace("&gt;", ">");
            parsed.alternative_title = parsed.alternative_title.replace("&ndash;", "-");
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
            parsed.about = parsed.about.replace("&#x27;", "'");
            parsed.about = parsed.about.replace("&#039;", "'");
            parsed.about = parsed.about.replace("&quot;", "\"");
            parsed.about = parsed.about.replace("&amp;", "&");
            parsed.about = parsed.about.replace("&lt;", "<");
            parsed.about = parsed.about.replace("&gt;", ">");
            parsed.about = parsed.about.replace("&ndash;", "-");
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
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            Show::default()
        }
    }
}

pub fn imdb(link: &str) -> Show {
    let parsed = scrab(
        link,
        r#"hero__primary-text">([^<]+)"#,
        r#"Original title: ([^<]+)"#,
        r#"releaseinfo\?ref_=tt_ov_rdat">([^<]+)"#,
        r#"bruFve">([^<]+)"#,
        r#"class="ipc-image" loading="eager" src="([^"]+)"#,
    );

    get_show(parsed)
}
