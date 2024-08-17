mod csv;
mod scanner;
use inquire::validator::Validation;
use reqwest;
use std;
use tokio;

#[derive(Debug)]
enum ReadsGoodError {
    Scanner(scanner::Error),
    Inquire(inquire::InquireError),
}

#[tokio::main]
async fn main() -> Result<(), ReadsGoodError> {
    // init reqwest client which we will ping goodreads with
    let client = reqwest::Client::new();

    // user prompts
    let listopia_url = inquire::Text::new("Provide the listopia url you would like to export")
        .with_validator(validate_listopia_url)
        .prompt();

    let file_name = inquire::Text::new("Provide the name of your csv file: e.g. `books.csv`")
        .with_validator(validate_filename)
        .prompt();

    match listopia_url {
        Ok(url) => match file_name {
            Ok(name) => {
                _ = scanner::scan(&client, url)
                    .await
                    .and_then(|books| Ok(csv::create(books, name)));
                Ok(())
            }

            Err(err) => {
                println!("Error parsing provided filename");
                Err(ReadsGoodError::Inquire(err))
            }
        },

        Err(err) => {
            println!("Error parsing provided listopia url");
            Err(ReadsGoodError::Inquire(err))
        }
    }
}

fn validate_listopia_url(
    url: &str,
) -> Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    if url.is_empty() {
        Ok(Validation::Invalid(
            "Must provide a goodreads listpopia url".into(),
        ))
    } else if !url.starts_with("https://") {
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
