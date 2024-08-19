use futures::future;
use reqwest;
use scraper;
use scraper::{Html, Selector};

const GOODREADS_BASE_URL: &str = "https://www.goodreads.com";

#[derive(Debug)]
pub enum Error {
    UnableToRetrieveListopia,
    UnableToRetrieveBook(String),
}

#[derive(Debug)]
pub struct Book {
    pub title: Option<String>,
    pub author: Option<String>,
    pub rating: Option<String>,
    pub original_publish_date: Option<String>,
    pub number_of_pages: Option<String>,
    pub number_of_ratings: Option<String>,
    pub number_of_reviews: Option<String>,
    pub genres: Vec<String>,
}

pub async fn scrape(
    client: &reqwest::Client,
    url: String,
    page_count: u32,
) -> Result<Vec<Book>, Error> {
    let mut all_books = Vec::new();

    for page in 1..=page_count {
        // append the current page as a query param
        let paginated_url = format!("{}?page={}", url, page);
        println!(
            "Scraping page {} of {}, url: {}",
            page, page_count, paginated_url
        );

        // fetch the listopia list, from which we get each of the individual book urls
        let listopia_html = get_list_html(client, paginated_url).await;

        match listopia_html {
            Ok(html) => {
                let book_urls = get_book_urls_from_list(html);

                // get each individual book's html for processing
                let retrieve_books_html_futures =
                    book_urls.into_iter().map(|url| get_book_html(client, url));

                let each_books_html: Result<Vec<Html>, Error> =
                    future::join_all(retrieve_books_html_futures)
                        .await
                        .into_iter()
                        .collect();

                let parsed_books: Result<Vec<Book>, Error> = each_books_html.map(|book_html_vec| {
                    book_html_vec
                        .iter()
                        .filter_map(|html| parse_book(&html))
                        .collect()
                });

                match parsed_books {
                    Ok(books) => all_books.extend(books),
                    Err(err) => return Err(err),
                }
            }
            Err(err) => return Err(err),
        }
    }

    Ok(all_books)
}

async fn get_html_from_url(client: &reqwest::Client, url: &str) -> Result<Html, reqwest::Error> {
    let response = client.get(url).send().await?;

    let html_as_string = response.text().await.unwrap_or_else(|_| String::from(""));
    let document = Html::parse_document(&html_as_string);

    Ok(document)
}

pub async fn get_list_html(client: &reqwest::Client, url: String) -> Result<Html, Error> {
    get_html_from_url(client, &url)
        .await
        .map_err(|_| Error::UnableToRetrieveListopia)
}

pub async fn get_book_html(client: &reqwest::Client, url: String) -> Result<Html, Error> {
    get_html_from_url(client, &url)
        .await
        .map_err(|_| Error::UnableToRetrieveBook(String::from(url)))
}

pub fn get_book_urls_from_list(html: Html) -> Vec<String> {
    let title_selector = Selector::parse("a.bookTitle").unwrap();

    let book_urls = html
        .select(&title_selector)
        .map(|element| {
            let href = element.value().attr("href").unwrap().to_string();
            format!("{}{}", GOODREADS_BASE_URL, href)
        })
        .collect::<Vec<String>>();
    book_urls
}

fn string_selector(html: &Html, selector: &str) -> Option<String> {
    let as_selector = Selector::parse(selector).ok();

    as_selector.and_then(|selec| {
        html.select(&selec)
            .next()
            .map(|element| element.text().collect::<String>())
    })
}

fn list_selector(html: &Html, selector: &str) -> Vec<String> {
    let as_selector = Selector::parse(selector).ok();

    let to_vector = as_selector
        .map(|selec| {
            html.select(&selec)
                .map(|element| element.text().collect::<String>())
                .collect::<Vec<String>>()
        })
        .unwrap_or_else(Vec::new);

    to_vector
}

// If any of the fields were not found (this sometime occurs and I don't know why)
// set the Book to be None, allowing us to ignore any books we couldn't parse to prevent
// the loop and later csv generation from panicking
fn parse_book(book_html: &Html) -> Option<Book> {
    let title = string_selector(book_html, "h1[data-testid='bookTitle']");
    let author = string_selector(book_html, "span.ContributorLink__name[data-testid='name']");
    let rating = string_selector(book_html, "div.RatingStatistics__rating");
    let number_of_ratings = string_selector(book_html, "span[data-testid='ratingsCount']")
        .and_then(|rating_count| {
            rating_count
                .split('\u{a0}')
                .next()
                .map(|rating_count| rating_count.to_string().replace(",", "").to_string())
        });

    let number_of_pages = string_selector(
        book_html,
        "div.FeaturedDetails p[data-testid='pagesFormat']",
    )
    .and_then(|page_count| {
        page_count
            .split(" ")
            .next()
            .map(|page_count| page_count.to_string())
    });

    let original_publish_date = string_selector(
        book_html,
        "div.FeaturedDetails p[data-testid='publicationInfo']",
    )
    .and_then(|date| {
        // Sometimes goodreads doesn't prefix with "First published", just "Published"
        if date.contains("First published") {
            date.split("First published")
                .nth(1)
                .map(|part| part.trim().to_string())
        } else {
            date.split("Published")
                .nth(1)
                .map(|part| part.trim().to_string())
        }
    });

    let number_of_reviews = string_selector(book_html, "span[data-testid='reviewsCount']")
        .and_then(|reviews_count| {
            reviews_count
                .split('\u{a0}')
                .next()
                .map(|reviews_count| reviews_count.to_string().replace(",", "").to_string())
        });

    let genres = list_selector(
        book_html,
        "div.BookPageMetadataSection__genres[data-testid='genresList'] span.Button__labelItem",
    )
    .iter()
    .filter(|genre| **genre != "...more")
    .cloned()
    .collect();

    if title.is_none()
        || author.is_none()
        || rating.is_none()
        || number_of_pages.is_none()
        || original_publish_date.is_none()
        || number_of_ratings.is_none()
        || number_of_reviews.is_none()
    {
        None
    } else {
        println!("Parsing book: {:?}, by: {:?}", title, author,);

        Some(Book {
            title,
            author,
            rating,
            number_of_ratings,
            original_publish_date,
            number_of_pages,
            number_of_reviews,
            genres,
        })
    }
}
