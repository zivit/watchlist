use crate::{parsers, sites::{self, Sites}};

fn check_site(link: &str, title: &str, alternative: &str, release_date: &str, about: &str) {
    let site = sites::check_link_is_allowed_site(link);
    let show = match site {
        Sites::Imdb => parsers::imdb(link),
        Sites::Unknown => panic!("Wrong site"),
    };
    assert_eq!(show.title.as_str(), title);
    assert_eq!(show.alternative_title.as_str(), alternative);
    assert_eq!(show.release_date.as_str(), release_date);
    assert_eq!(show.about.as_str(), about);
}

#[test]
fn imdb_the_orville() {
    check_site(
        "https://www.imdb.com/title/tt5691552/?ref_=nv_sr_srsg_0_tt_2_nm_0_q_%25D0%25BE%25D1%2580%25D0%25B2%25D1%2596%25D0%25BB",
        "Орвіл",
        "The Orville",
        "2017–2022",
        "Set 400 years in the future, the crew of the U.S.S. Orville continue their mission of exploration, navigating both the mysteries of the universe, and the complexities of their own interpersonal relationships."
    );
}

#[test]
fn imdb_mushoku_tensei() {
    check_site(
        "https://www.imdb.com/title/tt13293588/?ref_=nv_sr_srsg_0_tt_3_nm_0_q_%25D1%2580%25D0%25B5%25D1%2596%25D0%25BD%25D0%25BA",
        "Реінкарнація безробітного: В інший світ на повному серйозі",
        "Mushoku Tensei: Isekai Ittara Honki Dasu",
        "2021– ",
        "A 34-year-old underachiever gets run over by a bus, but his story isn't over. Reincarnated as an infant, he'll embark on an epic adventure."
    );
}

#[test]
fn imdb_your_name() {
    check_site(
        "https://www.imdb.com/title/tt5311514/?ref_=nv_sr_srsg_0_tt_8_nm_0_q_%25D1%2596%25D0%25BC%27%25D1%258F",
        "Твоє ім'я",
        "Kimi no na wa.",
        "2016",
        "Two teenagers share a profound, magical connection upon discovering they are swapping bodies. Things manage to become even more complicated when the boy and girl decide to meet in person."
    );
}

#[test]
fn imdb_im_not_there() {
    check_site(
        "https://www.imdb.com/title/tt0368794/?ref_=nv_sr_srsg_3_tt_8_nm_0_q_i%27m",
        "Мене там немає",
        "I'm Not There",
        "2007",
        "Ruminations on the life of Bob Dylan, where six characters embody a different aspect of the musician's life and work."
    );
}

#[test]
fn imdb_yoru_no_kurage() {
    check_site(
        "https://www.imdb.com/title/tt0388629/?ref_=nv_sr_srsg_0_tt_7_nm_1_q_one%2520pi",
        "Ван Піс",
        "One Piece: Wan pîsu",
        "1999– ",
        "Monkey D. Luffy sets off on an adventure with his pirate crew in hopes of finding the greatest treasure ever, known as the \"One Piece.\""
    );
}
