use anyhow::Result;
use downloader::{Download, Downloader};

pub fn download_image_by_http(url: &std::path::Path) -> Result<std::path::PathBuf> {
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
            Err(e) => print!("Error: {}", e),
            Ok(s) => print!("Success: {}", &s),
        }
    }

    Ok(std::path::PathBuf::from(
        &p.join(url.file_name().unwrap_or_default()),
    ))
}
