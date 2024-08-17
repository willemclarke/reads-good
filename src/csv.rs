use crate::scanner;
use csv;

pub fn create(books: Vec<scanner::Book>, file_name: String) {
    let path = std::path::Path::new(&file_name);
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

        println!(
            "Writing book: {:?} by: {:?} to {:?}",
            title, author, file_name
        );
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
