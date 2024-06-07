use crate::{parsers, Show};

const SITE_IMDB: &str = "https://www.imdb.com";

#[derive(PartialEq)]
pub enum Sites {
    Imdb,
    Unknown,
}

pub fn check_link_is_allowed_site(link: &str) -> Sites {
    if link.starts_with(SITE_IMDB) {
        Sites::Imdb
    } else {
        Sites::Unknown
    }
}

pub fn check_link_is_importable(link: &str) -> bool {
    if link.is_empty() {
        return false;
    }

    let site = check_link_is_allowed_site(link);
    site != Sites::Unknown
}

pub fn import_clicked(link: &str) -> Show {
    let site = check_link_is_allowed_site(link);
    if site == Sites::Imdb {
        return parsers::imdb(link);
    }
    Show::default()
}
