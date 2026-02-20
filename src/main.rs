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

    println!("[INFO] app_ids {:?}", app_ids);

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
        .expect("[ERROR] Failed to write");

    println!("[INFO] Starting scraping of {app_id}...");

    loop {
        let url = get_url(&app_id, &cursor);

        println!(
            "[INFO] Fetching reviews for app_id: {} url: {} cursor: {}",
            app_id, url, cursor
        );

        let mut json = fetch(&url);

        let success = json["success"].take();

        if success != 1 {
            panic!(
                "[ERROR] Unsuccessful response from url: {}. success: {}, Terminating.",
                url, success
            );
        }

        let reviews = json["reviews"].take();

        println!("[INFO] Review count for page {}", reviews.len());

        if reviews.len() == 0 {
            println!(
                "[INFO] No reviews in response of url: {}. Terminating.",
                url
            );
            break;
        }

        write_to_file(&mut file, &reviews)
            .map_err(|err| {
                panic!("[ERROR] Error writing reviews: {err}");
            })
            .ok();

        reviews_scraped += reviews.len();

        cursor = match json["cursor"].take_string() {
            Some(cursor) => cursor,
            None => {
                panic!("[ERROR] No cursor. Terminating.");
            }
        };
    }

    println!(
        "[INFO] Finished scraping of app_id: {}. Reviews scraped: {}",
        app_id, reviews_scraped
    );
    println!();
}

fn get_url(app_id: &u32, cursor: &String) -> String {
    let base_url = format!("https://store.steampowered.com/appreviews/{app_id}");
    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .append_pair("json", "1")
        .append_pair("cursor", cursor)
        .append_pair("day_range", "0")
        // Date is seconds since epoch
        .append_pair("start_date", "1") // "-1" | "1"
        .append_pair("end_date", "1771617810") // "-1" | "1771617810"
        .append_pair("date_range_type", "include") // "all" | "include" | "exclude"
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

    format!("{base_url}?{query}")
}

fn write_to_file(file: &mut LineWriter<File>, reviews: &JsonValue) -> std::io::Result<()> {
    for review in reviews.members() {
        let creation_timestamp = &review["timestamp_created"];
        let review_id = &review["recommendationid"];
        let review_text = &review["review"];

        if creation_timestamp.as_u32().unwrap() > 1771617810 {
            panic!("[ERROR] Review creation_date is more recent than end_date filter! Exiting.");
        }

        file.write_all(format!("Review {review_id}:\n{review_text}\n\n").as_bytes())?;

        file.flush()?;
    }

    Ok(())
}
