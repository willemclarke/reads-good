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
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub edition_publish_date: Option<String>,
    pub number_of_pages: Option<String>,
    pub number_of_ratings: Option<String>,
    pub number_of_reviews: Option<String>,
    pub genres: Vec<String>,
}

pub async fn scan(client: &reqwest::Client, url: String) -> Result<Vec<Book>, Error> {
    let listopia_html = get_list_html(client, url).await;

    match listopia_html {
        Ok(html) => {
            let book_urls = get_book_urls_from_list(html);
            println!("parsing following books: {:#?}", book_urls);

            let retrieve_books_html_futures =
                book_urls.into_iter().map(|url| get_book_html(client, url));

            let each_books_html: Result<Vec<Html>, Error> =
                future::join_all(retrieve_books_html_futures)
                    .await
                    .into_iter()
                    .collect();

            let parsed_books: Result<Vec<Book>, Error> = each_books_html
                .map(|book_html_vec| book_html_vec.iter().map(|html| parse_book(&html)).collect());

            parsed_books
        }
        Err(err) => Err(err),
    }
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

fn parse_book(book_html: &Html) -> Book {
    let title_selector = Selector::parse("h1[data-testid='bookTitle']").unwrap();
    let author_selector =
        Selector::parse("span.ContributorLink__name[data-testid='name']").unwrap();
    let rating_selector = Selector::parse("div.RatingStatistics__rating").unwrap();
    let ratings_count_seletor = Selector::parse("span[data-testid='ratingsCount']").unwrap();
    let pages_count_selector =
        Selector::parse("div.FeaturedDetails p[data-testid='pagesFormat']").unwrap();
    let original_publish_date_selector =
        Selector::parse("div.FeaturedDetails p[data-testid='publicationInfo']").unwrap();
    let reviewers_count_selector = Selector::parse("span[data-testid='reviewsCount']").unwrap();
    let genres_selector = Selector::parse(
        "div.BookPageMetadataSection__genres[data-testid='genresList'] span.Button__labelItem",
    )
    .unwrap();

    let title = string_selector(book_html, title_selector);
    let author = string_selector(book_html, author_selector);
    let rating = string_selector(book_html, rating_selector);
    let number_of_pages = string_selector(book_html, pages_count_selector).and_then(|page_count| {
        page_count
            .split(" ")
            .next()
            .map(|page_count| page_count.to_string())
    });

    let original_publish_date = string_selector(book_html, original_publish_date_selector);
    let original_publish_date = original_publish_date.and_then(|date| {
        date.split("First published")
            .nth(1)
            .map(|part| part.trim().to_string())
    });

    let number_of_ratings =
        string_selector(book_html, ratings_count_seletor).and_then(|page_count| {
            page_count
                .split('\u{a0}')
                .next()
                .map(|page_count| page_count.to_string().replace(",", "").to_string())
        });

    let number_of_reviews =
        string_selector(book_html, reviewers_count_selector).and_then(|page_count| {
            page_count
                .split('\u{a0}')
                .next()
                .map(|page_count| page_count.to_string().replace(",", "").to_string())
        });

    let genres = book_html
        .select(&genres_selector)
        .map(|element| element.text().collect::<String>())
        .filter(|genre| genre != "...more")
        .collect::<Vec<String>>();

    println!("Parsing book: {:?}, by: {:?}", title, author);

    Book {
        title,
        author,
        rating,
        number_of_pages,
        isbn: None,
        asin: None,
        edition_publish_date: None,
        original_publish_date,
        number_of_ratings,
        number_of_reviews,
        genres,
    }
}
