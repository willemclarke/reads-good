mod scraper;
use csv;
use inquire;
use reqwest;
use tokio;

// "https://www.goodreads.com/list/show/3810.Best_Cozy_Mystery_Series"
#[tokio::main]
async fn main() -> Result<(), scraper::Error> {
    // init reqwest client to make requests to goodreads
    let client = reqwest::Client::new();

    let listopia_url =
        inquire::Text::new("Provide the listopia url you would like to scrape").prompt();

    match listopia_url {
        Ok(url) => {
            _ = scraper::scrape(&client, String::from(url))
                .await
                .and_then(|books| Ok(to_csv(books)));
            Ok(())
        }
        Err(err) => {
            println!("Error reading provided url: {:?}", err);
            Err(scraper::Error::UnableToRetrieveListopia)
        }
    }
}

fn to_csv(books: Vec<scraper::Book>) {
    let path = std::path::Path::new("books.csv");
    let mut writer = csv::Writer::from_path(path).unwrap();

    writer
        .write_record(&[
            "title",
            "author",
            "original_publish_date",
            "rating",
            "number_of_ratings",
            "number_of_pages",
            "number_of_reviews",
            "genres",
        ])
        .unwrap();

    for book in books {
        let title = book.title.unwrap();
        let author = book.author.unwrap();
        let original_publish_date = book.original_publish_date.unwrap();
        let rating = book.rating.unwrap();
        let number_of_ratings = book.number_of_ratings.unwrap();
        let number_of_pages = book.number_of_pages.unwrap();
        let number_of_reviews = book.number_of_reviews.unwrap();
        let genres = book.genres.join(", ");

        println!("Writing book: {:?} by: {:?} to csv file", title, author);
        writer
            .write_record(&[
                title,
                author,
                original_publish_date,
                rating,
                number_of_ratings,
                number_of_pages,
                number_of_reviews,
                genres,
            ])
            .unwrap();
    }

    // free up the resources
    writer.flush().unwrap();
}
