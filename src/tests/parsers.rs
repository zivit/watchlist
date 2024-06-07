use crate::parsers;

#[test]
fn imdb_the_orville() {
    let show = parsers::imdb("https://www.imdb.com/title/tt5691552/?ref_=nv_sr_srsg_0_tt_2_nm_0_q_%25D0%25BE%25D1%2580%25D0%25B2%25D1%2596%25D0%25BB");
    assert_eq!(show.title.as_str(), "Орвіл");
    assert_eq!(show.alternative_title.as_str(), "The Orville");
    assert_eq!(show.release_date.as_str(), "2017–2022");
    assert_eq!(show.about.as_str(), "Set 400 years in the future, the crew of the U.S.S. Orville continue their mission of exploration, navigating both the mysteries of the universe, and the complexities of their own interpersonal relationships.");
}

#[test]
fn imdb_mushoku_tensei() {
    let show = parsers::imdb("https://www.imdb.com/title/tt13293588/?ref_=nv_sr_srsg_0_tt_3_nm_0_q_%25D1%2580%25D0%25B5%25D1%2596%25D0%25BD%25D0%25BA");
    assert_eq!(show.title.as_str(), "Реінкарнація безробітного: В інший світ на повному серйозі");
    assert_eq!(show.alternative_title.as_str(), "Mushoku Tensei: Isekai Ittara Honki Dasu");
    assert_eq!(show.release_date.as_str(), "2021– ");
    assert_eq!(show.about.as_str(), "A 34-year-old underachiever gets run over by a bus, but his story isn't over. Reincarnated as an infant, he'll embark on an epic adventure.");
}

#[test]
fn imdb_your_name() {
    let show = parsers::imdb("https://www.imdb.com/title/tt5311514/?ref_=nv_sr_srsg_0_tt_8_nm_0_q_%25D1%2596%25D0%25BC%27%25D1%258F");
    assert_eq!(show.title.as_str(), "Твоє ім'я");
    assert_eq!(show.alternative_title.as_str(), "Kimi no na wa.");
    assert_eq!(show.release_date.as_str(), "2016");
    assert_eq!(show.about.as_str(), "Two teenagers share a profound, magical connection upon discovering they are swapping bodies. Things manage to become even more complicated when the boy and girl decide to meet in person.");
}

#[test]
fn imdb_im_not_there() {
    let show = parsers::imdb("https://www.imdb.com/title/tt0368794/?ref_=nv_sr_srsg_3_tt_8_nm_0_q_i%27m");
    assert_eq!(show.title.as_str(), "Мене там немає");
    assert_eq!(show.alternative_title.as_str(), "I'm Not There");
    assert_eq!(show.release_date.as_str(), "2007");
    assert_eq!(show.about.as_str(), "Ruminations on the life of Bob Dylan, where six characters embody a different aspect of the musician's life and work.");
}

#[test]
fn imdb_yoru_no_kurage() {
    let show = parsers::imdb("https://www.imdb.com/title/tt27237459/?ref_=nv_sr_srsg_6_tt_8_nm_0_q_%25D0%25BC%25D0%25B5%25D0%25B4%25D1%2583%25D0%25B7%25D0%25B0");
    assert_eq!(show.title.as_str(), "Yoru no kurage wa oyogenai");
    assert_eq!(show.alternative_title.as_str(), "Yoru no kurage wa oyogenai");
    assert_eq!(show.release_date.as_str(), "2024– ");
    assert_eq!(show.about.as_str(), "Four girls, an illustrator, a former idol, a VTuber, and a composer, form an anonymous artist group called \"JELEE\", each pursuing their artistic dreams while supporting one another.");
}
