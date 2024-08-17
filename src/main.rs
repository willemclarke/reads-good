mod scraper;
use inquire;

// "https://www.goodreads.com/list/show/3810.Best_Cozy_Mystery_Series"
fn main() -> Result<(), scraper::Error> {
    let listopia_url =
        inquire::Text::new("Provide the listopia url you would like to scrape").prompt();

    match listopia_url {
        Ok(url) => {
            scraper::scrape(String::from(url));
            Ok(())
        }
        Err(err) => {
            println!("Error reading provided url: {:?}", err);
            Err(scraper::Error::UnableToRetrieveListopia)
        }
    }
}
