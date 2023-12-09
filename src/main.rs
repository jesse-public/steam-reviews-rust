mod fetch;
mod options;

use fetch::fetch;
use json::JsonValue;
use std::env;
use std::fs::File;
use std::io::{LineWriter, Write};
use std::process::exit;
use url::form_urlencoded;

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

    println!("Fetching reviews for {app_id}...");

    loop {
        let url = get_url(&app_id, &cursor);

        println!("{}", cursor);

        let mut json = fetch(&url);

        let reviews = json["reviews"].take();

        println!("review count for page {}", reviews.len());

        if reviews.len() == 0 {
            println!("No reviews in response of url: {}. Terminating.", url);
            break;
        }

        write_to_file(&mut file, &reviews)
            .map_err(|err| {
                println!("Error writing reviews: {err}");
            })
            .ok();

        reviews_scraped += reviews.len();

        cursor = match json["cursor"].take_string() {
            Some(cursor) => cursor,
            None => {
                println!("No cursor. Terminating.");
                break;
            }
        };
    }

    println!("Reviews scraped: {}", reviews_scraped);
}

fn get_url(app_id: &u32, cursor: &String) -> String {
    let base_url = format!("https://store.steampowered.com/appreviews/{app_id}");
    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .append_pair("json", "1")
        .append_pair("cursor", cursor)
        .append_pair("day_range", "0")
        .append_pair("start_date", "-1")
        .append_pair("end_date", "-1")
        .append_pair("date_range_type", "all")
        .append_pair("filter", "recent")
        .append_pair("language", "english")
        .append_pair("l", "english")
        .append_pair("review_type", "all")
        .append_pair("purchase_type", "all")
        .append_pair("playtime_filter_min", "0")
        .append_pair("playtime_filter_max", "0")
        .append_pair("filter_offtopic_activity", "0")
        .finish();
    let query = encoded.to_string();

    let url = format!("{base_url}?{query}");

    println!("url: {}", url);

    url
}

fn write_to_file(file: &mut LineWriter<File>, reviews: &JsonValue) -> std::io::Result<()> {
    for review in reviews.members() {
        let review_id = &review["recommendationid"];
        let review_text = &review["review"];

        // println!("Writing review {review_id}...");

        file.write_all(format!("Review {review_id}:\n{review_text}\n\n").as_bytes())?;

        file.flush()?;
    }

    Ok(())
}
