use futures::future;
use reqwest;
use scraper;
use scraper::{Html, Selector};

const GOODREADS_BASE_URL: &str = "https://www.goodreads.com";

#[derive(Debug)]
pub enum Error {
    UnableToReadListopiaUrl,
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

fn string_selector(html: &Html, selector: Selector) -> Option<String> {
    html.select(&selector)
        .next()
        .map(|element| element.text().collect::<String>())
}

// If any of the fields were not found (this sometime occurs and I don't know why)
// set the Book to be None, allowing us to ignore any books we couldn't parse to prevent
// the loop and later csv generation from panicking
fn parse_book(book_html: &Html) -> Option<Book> {
    let title_selector = Selector::parse("h1[data-testid='bookTitle']").ok();
    let author_selector = Selector::parse("span.ContributorLink__name[data-testid='name']").ok();
    let rating_selector = Selector::parse("div.RatingStatistics__rating").ok();
    let ratings_count_seletor = Selector::parse("span[data-testid='ratingsCount']").ok();
    let pages_count_selector =
        Selector::parse("div.FeaturedDetails p[data-testid='pagesFormat']").ok();
    let original_publish_date_selector =
        Selector::parse("div.FeaturedDetails p[data-testid='publicationInfo']").ok();
    let reviewers_count_selector = Selector::parse("span[data-testid='reviewsCount']").ok();
    let genres_selector = Selector::parse(
        "div.BookPageMetadataSection__genres[data-testid='genresList'] span.Button__labelItem",
    )
    .ok();

    let title = title_selector.and_then(|selector| string_selector(book_html, selector));
    let author = author_selector.and_then(|selector| string_selector(book_html, selector));
    let rating = rating_selector.and_then(|selector| string_selector(book_html, selector));
    let number_of_pages = pages_count_selector
        .and_then(|selector| string_selector(book_html, selector))
        .and_then(|page_count| {
            page_count
                .split(" ")
                .next()
                .map(|page_count| page_count.to_string())
        });

    let original_publish_date = original_publish_date_selector
        .and_then(|selector| string_selector(book_html, selector))
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

    let number_of_ratings = ratings_count_seletor
        .and_then(|selector| string_selector(book_html, selector))
        .and_then(|rating_count| {
            rating_count
                .split('\u{a0}')
                .next()
                .map(|rating_count| rating_count.to_string().replace(",", "").to_string())
        });

    let number_of_reviews = reviewers_count_selector
        .and_then(|selector| string_selector(book_html, selector))
        .and_then(|reviews_count| {
            reviews_count
                .split('\u{a0}')
                .next()
                .map(|reviews_count| reviews_count.to_string().replace(",", "").to_string())
        });

    let genres = genres_selector
        .map(|selector| {
            book_html
                .select(&selector)
                .map(|element| element.text().collect::<String>())
                .filter(|genre| genre != "...more")
                .collect::<Vec<String>>()
        })
        .unwrap_or_else(Vec::new);

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
            number_of_pages,
            original_publish_date,
            number_of_ratings,
            number_of_reviews,
            genres,
        })
    }
}
