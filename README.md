### reads-good

A CLI tool which takes a goodreads listopia public list url:
(e.g. goodreads.com/list/show/264.Books_That_Everyone_Should_Read_At_Least_Once)
and exports the *below* properties of each book into a csv file.


```rust
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
```

Behaviour:
- Input the url to the listopia list
- Input a desired filename, e.g. `books.csv`
- Input desired number of pages you would like to export (number between 1 - 10)

Some books for whatever reason may fail to parse some of the selectors (e.g. title/author/page_count), in this case, these books will be ignored from the CSV generation.
As such, if you asked for `3` pages, and see 298 rows in your csv file, thats the reason why.

Sometimes when I run the CLI, it will get to a given page and then silently fail with no warnings, I don't really know why. If that happens,
usually restarting is enough to get it working. If you can tell why, feel free to submit a PR.

To run:
- `git clone`
- `cargo build --release`
- `cd` into `reads-good/target/release/` and run `./reads-good` or double click executable
- Note: the csv file will be outputted within the directory of `reads-good/target/release/`

csv example:
![image](https://github.com/user-attachments/assets/faae5e90-35e1-48da-8195-9b6b27cda36d)
