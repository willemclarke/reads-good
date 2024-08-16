use reqwest;
use scraper;
use scraper::{Html, Selector};

const GOODREADS_BASE_URL: &str = "https://www.goodreads.com";

#[derive(Debug)]
enum Error {
    UnableToRetrieveListopia,
    UnableToRetrieveBook(String),
}

#[derive(Debug)]
struct Book {
    title: String,
    author: String,
    rating: String,
    original_publish_date: String,
    number_of_pages: String,
    number_of_ratings: String,
    number_of_reviews: String,
    genres: Vec<String>,
}

fn main() -> Result<(), Error> {
    let listopia_html =
        get_list_html("https://www.goodreads.com/list/show/3810.Best_Cozy_Mystery_Series");

    listopia_html.map(|html| {
        let book_urls = get_book_urls_from_list(html)[0..1].to_vec();
        let books: Result<Vec<Html>, Error> = book_urls
            .into_iter()
            .map(|url| get_book_html(&url))
            .collect();
        let parsed_books: Result<Vec<Book>, Error> = books.map(|vec_book| {
            vec_book
                .into_iter()
                .map(|book_html| parse_book(&book_html))
                .collect()
        });

        println!("parsed_books: {:#?}", parsed_books);
    })
}

fn get_html_from_url(url: &str) -> Result<Html, reqwest::Error> {
    let response = reqwest::blocking::get(url);

    response.map(|res| {
        let html_as_string = res.text().unwrap_or(String::from(""));
        let document = Html::parse_document(&html_as_string);
        return document;
    })
}

fn get_list_html(url: &str) -> Result<Html, Error> {
    get_html_from_url(url).map_err(|_| Error::UnableToRetrieveListopia)
}

fn get_book_html(url: &str) -> Result<Html, Error> {
    get_html_from_url(url).map_err(|_| Error::UnableToRetrieveBook(String::from(url)))
}

fn get_book_urls_from_list(html: Html) -> Vec<String> {
    let mut urls: Vec<String> = Vec::new();
    let title_selector = Selector::parse("a.bookTitle").unwrap();

    for element in html.select(&title_selector) {
        if let Some(href) = element.value().attr("href") {
            let url = format!("{}{}", GOODREADS_BASE_URL, href);
            urls.push(url);
        }
    }

    urls
}

fn string_selector(html: &Html, selector: Selector) -> String {
    html.select(&selector)
        .next()
        .map(|element| element.text().collect::<String>())
        .unwrap()
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
    let number_of_pages = string_selector(book_html, pages_count_selector)
        .split(" ")
        .next()
        .unwrap_or("")
        .to_string();

    let original_publish_date_string = string_selector(book_html, original_publish_date_selector);
    let original_publish_date_parts: Vec<&str> = original_publish_date_string
        .split("First published")
        .collect();
    let original_publish_date = original_publish_date_parts
        .get(1)
        .unwrap_or(&"")
        .trim()
        .to_string();

    let number_of_ratings = string_selector(book_html, ratings_count_seletor)
        .split('\u{a0}')
        .next()
        .unwrap_or("")
        .trim()
        .replace(",", "");

    let number_of_reviews = string_selector(book_html, reviewers_count_selector)
        .split('\u{a0}')
        .next()
        .unwrap_or("")
        .trim()
        .replace(",", "");

    let genres = book_html
        .select(&genres_selector)
        .map(|element| element.text().collect::<String>())
        .filter(|genre| genre != "...more")
        .collect::<Vec<String>>();

    Book {
        title,
        author,
        rating,
        number_of_pages,
        original_publish_date,
        number_of_ratings,
        number_of_reviews,
        genres,
    }
}
