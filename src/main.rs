mod fetch;
mod options;

use fetch::fetch;
use json::JsonValue;
use std::env;
use std::fs::File;
use std::io::{LineWriter, Write};
use std::process::exit;
use url::form_urlencoded;

const REVIEWS_PER_PAGE: u32 = 50;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <app id> [<app id>...]", args[0]);
        exit(0);
    }

    let app_ids = options::extract_app_ids(&args);

    println!("app_ids {:?}", app_ids);

    for app_id in app_ids {
        scrape_reviews(app_id);
    }
}

fn scrape_reviews(app_id: u32) {
    let file_name = format!("{app_id}.txt");
    let file = File::create(&file_name).expect("Could not create file {file_name}");
    let mut file = LineWriter::new(file);
    let mut cursor = String::from("*");
    let mut reviews_scraped: usize = 0;

    file.write_all(format!("[Reviews for {app_id}]\n\n").as_bytes())
        .expect("Failed to write");

    loop {
        let url = get_url(&app_id, &REVIEWS_PER_PAGE, &cursor);

        println!("Fetching reviews for {app_id}...");
        println!("url: {}", url);

        let mut json = fetch(&url);

        let reviews = json["reviews"].take();

        println!("review count for page {}", reviews.len());

        if reviews.len() == 0 {
            println!("No reviews in response. Terminating.");
            println!("Reviews scraped: {}", reviews_scraped);
            break;
        }

        write_to_file(&mut file, &reviews)
            .map_err(|err| {
                println!("Error writing reviews: {err}");
            })
            .ok();

        reviews_scraped += reviews.len();

        cursor = match json["cursor"].take_string() {
            Some(cursor) => {
                println!("cursor {}", cursor);
                cursor
            }
            None => {
                println!("No cursor. Terminating.");
                println!("Reviews scraped: {}", reviews_scraped);
                break;
            }
        };
    }
}

fn get_url(app_id: &u32, num_per_page: &u32, cursor: &String) -> String {
    let base_url = format!("https://store.steampowered.com/appreviews/{app_id}");
    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .append_pair("json", "1")
        .append_pair("purchase_type", "all")
        .append_pair("num_per_page", &format!("{num_per_page}"))
        .append_pair("filter", "recent")
        .append_pair("cursor", &format!("{cursor}"))
        .finish();

    let query = encoded.to_string();

    format!("{base_url}?{query}")
}

fn write_to_file(file: &mut LineWriter<File>, reviews: &JsonValue) -> std::io::Result<()> {
    for review in reviews.members() {
        let review_id = &review["recommendationid"];
        let review_text = &review["review"];

        println!("Writing review {review_id}...");

        file.write_all(format!("Review {review_id}:\n{review_text}\n\n").as_bytes())?;

        file.flush()?;
    }

    Ok(())
}
