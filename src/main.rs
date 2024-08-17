mod scraper;

// "https://www.goodreads.com/list/show/3810.Best_Cozy_Mystery_Series"
fn main() -> Result<(), scraper::Error> {
    let x = scraper::scrape(String::from(
        "https://www.goodreads.com/list/show/3810.Best_Cozy_Mystery_Series",
    ));
    Ok(())
}
