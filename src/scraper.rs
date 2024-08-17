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
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub edition_publish_date: Option<String>,
    pub number_of_pages: Option<String>,
    pub number_of_ratings: Option<String>,
    pub number_of_reviews: Option<String>,
    pub genres: Vec<String>,
}

pub fn scrape(url: String) -> Result<Vec<Book>, Error> {
    let listopia_html = get_list_html(&url);

    match listopia_html {
        Ok(html) => {
            let book_urls = get_book_urls_from_list(html)[0..10].to_vec();
            println!("parsing following urls: {:#?}", book_urls);

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

            parsed_books
        }

        Err(err) => Err(err),
    }
}

fn get_html_from_url(url: &str) -> Result<Html, reqwest::Error> {
    let response = reqwest::blocking::get(url);

    response.map(|res| {
        let html_as_string = res.text().unwrap_or(String::from(""));
        let document = Html::parse_document(&html_as_string);
        return document;
    })
}

pub fn get_list_html(url: &str) -> Result<Html, Error> {
    get_html_from_url(url).map_err(|_| Error::UnableToRetrieveListopia)
}

pub fn get_book_html(url: &str) -> Result<Html, Error> {
    get_html_from_url(url).map_err(|_| Error::UnableToRetrieveBook(String::from(url)))
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
