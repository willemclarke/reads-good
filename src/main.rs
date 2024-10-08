mod csv;
mod scraper;
use inquire::{validator::Validation, InquireError};
use reqwest;
use std;
use tokio;

#[derive(Debug)]
enum ReadsGoodError {
    Scraper(scraper::Error),
    Inquire(InquireError),
}

#[tokio::main]
async fn main() -> Result<(), ReadsGoodError> {
    // init reqwest client which we will ping goodreads with
    let client = reqwest::Client::new();
    let (url, name, count) = gather_input()?;

    let _ = scraper::run(&client, url, count)
        .await
        .and_then(|books| Ok(csv::create(books, name)));

    Ok(())
}

fn gather_input() -> Result<(String, String, u32), ReadsGoodError> {
    let listopia_url = inquire::Text::new("Provide the listopia url you would like to export:")
        .with_help_message("Please ensure the url you provide begins on the first page")
        .with_validator(validate_listopia_url)
        .prompt()
        .map_err(|err| ReadsGoodError::Inquire(err))?;

    let file_name = inquire::Text::new("Provide the name of your csv file: e.g. `books.csv`:")
        .with_validator(validate_filename)
        .prompt()
        .map_err(|err| ReadsGoodError::Inquire(err))?;

    let page_count = inquire::CustomType::<u32>::new(
        "How many pages would you like to export? (Number between 1 - 10):",
    )
    .with_validator(validate_page_number)
    .prompt()
    .map_err(|err| ReadsGoodError::Inquire(err))?;

    Ok((listopia_url, file_name, page_count))
}

fn validate_listopia_url(
    url: &str,
) -> Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    if url.is_empty() {
        Ok(Validation::Invalid(
            "Must provide a goodreads listpopia url".into(),
        ))
    } else if url.contains("?page=") {
        Ok(Validation::Invalid(
            "Ensure goodreads listopia url starts from first page, there should be no `?page=X` query param".into(),
        ))
    } else if !url.starts_with("https://www.goodreads.com/list/show") {
        Ok(Validation::Invalid(
            "Ensure goodreads listopia url is valid".into(),
        ))
    } else {
        Ok(Validation::Valid)
    }
}

fn validate_filename(
    file_name: &str,
) -> Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    if !file_name.ends_with(".csv") {
        Ok(Validation::Invalid(
            "File names must end in `.csv`. e.g. `books.csv`".into(),
        ))
    } else {
        Ok(Validation::Valid)
    }
}

fn validate_page_number(
    page_number: &u32,
) -> Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    if *page_number > 1 && *page_number <= 10 {
        Ok(Validation::Valid)
    } else {
        Ok(Validation::Invalid(
            "Select a number between 1 and 10".into(),
        ))
    }
}
