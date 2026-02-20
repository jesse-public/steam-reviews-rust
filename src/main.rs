mod fetch;
mod options;

use fetch::fetch;
use json::JsonValue;
use std::collections::HashMap;
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
    let mut review_counts = HashMap::new();
    let mut total_review_count: usize = 0;

    println!("[DEBUG] app_ids {:?}", app_ids);

    for &app_id in &app_ids {
        let review_count = scrape_reviews(app_id);

        total_review_count += review_count;
        review_counts.insert(app_id, review_count);
    }

    record_results(app_ids, review_counts);
}

fn record_results(app_ids: Vec<u32>, review_counts: HashMap<u32, usize>) {
    let mut total_review_count: usize = 0;

    println!();
    println!("[INFO] Results");

    for &app_id in &app_ids {
        let review_count = review_counts.get(&app_id).expect(
            format!(
                "[ERROR] Missing review_count for app_id: {}. Terminating.",
                app_id
            )
            .as_str(),
        );
        total_review_count += review_count;
        println!("[INFO] app_id: {} review_count: {}", app_id, review_count);
    }

    println!("[INFO] total_review_count: {}", total_review_count);
    println!();
}

fn scrape_reviews(app_id: u32) -> usize {
    let file_name = format!("{app_id}.txt");
    let file =
        File::create(&file_name).expect("[ERROR] Could not create file {file_name}. Terminating.");
    let mut file = LineWriter::new(file);
    let mut cursor = String::from("*");
    let mut reviews_scraped: usize = 0;

    file.write_all(format!("[Reviews for {app_id}]\n\n").as_bytes())
        .expect(
            format!(
                "[ERROR] Failed to write reviews for app_id: {}. Terminating.",
                app_id
            )
            .as_str(),
        );

    println!("[DEBUG] Starting scraping of {app_id}");

    loop {
        let url = get_url(&app_id, &cursor);

        println!(
            "[DEBUG] Fetching reviews for app_id: {} url: {} cursor: {}",
            app_id, url, cursor
        );

        let mut json = fetch(&url);

        let success = json["success"].take();

        if success != 1 {
            panic!(
                "[ERROR] Unsuccessful response from url: {}. Terminating.",
                url
            );

            // TODO(optional): Exponential backoff
        }

        let reviews = json["reviews"].take();

        println!("[DEBUG] Review count for page {}", reviews.len());

        if reviews.len() == 0 {
            println!(
                "[DEBUG] No reviews in response of url: {}. Terminating.",
                url
            );
            break;
        }

        write_to_file(&mut file, &reviews)
            .map_err(|err| {
                panic!("[ERROR] Error writing reviews: {err}. Terminating.");
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

    reviews_scraped
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
            panic!(
                "[ERROR] Review creation_date is more recent than end_date filter. Terminating."
            );
        }

        file.write_all(format!("Review {review_id}:\n{review_text}\n\n").as_bytes())?;

        file.flush()?;
    }

    Ok(())
}
