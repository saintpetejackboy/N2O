// src/lib.rs

use warp::Filter;
use serde::{Deserialize, Serialize};
use flate2::write::GzEncoder;
use flate2::Compression;
use chrono::Local;

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// A global constant for the data file path.
pub const DATA_FILE: &str = "n2o_data.json";

/// Our in-memory store type used by all endpoints.
pub type Store = Arc<Mutex<HashMap<String, Vec<String>>>>;

/// Archives the current data to a compressed file with a timestamp.
///
/// This is a private helper (not tested directly) but used internally
/// by the `/clear` endpoint.
fn archive_data(file_path: &str) -> std::io::Result<()> {
    // Load current data
    let data = load_data(file_path);

    if data.is_empty() {
        // Nothing to archive
        return Ok(());
    }

    // Serialize data to JSON
    let json_data = serde_json::to_string_pretty(&PersistData(data.clone()))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Get current timestamp
    let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();

    // Define archive filename
    let archive_filename = format!("n2o_data_backup_{}.json.gz", timestamp);

    // Create the compressed file
    let file = fs::File::create(&archive_filename)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(json_data.as_bytes())?;
    encoder.finish()?;

    println!("Archived data to {}", archive_filename);
    Ok(())
}

/// Converts a single alphabetic character to its corresponding phone keypad digit.
///
/// This is private because we only expose `convert_to_ten_digits` publicly.
fn letter_to_digit(c: char) -> Option<char> {
    match c.to_ascii_uppercase() {
        'A' | 'B' | 'C' => Some('2'),
        'D' | 'E' | 'F' => Some('3'),
        'G' | 'H' | 'I' => Some('4'),
        'J' | 'K' | 'L' => Some('5'),
        'M' | 'N' | 'O' => Some('6'),
        'P' | 'Q' | 'R' | 'S' => Some('7'),
        'T' | 'U' | 'V' => Some('8'),
        'W' | 'X' | 'Y' | 'Z' => Some('9'),
        _ => None,
    }
}

/// Converts an input phone number string to its 10-digit representation.
/// - Extracts digits and converts letters based on the phone keypad.
/// - If the resulting number has <= 10 digits, returns as is.
/// - If more than 10 digits, returns the *last* 10 digits.
pub fn convert_to_ten_digits(number: &str) -> String {
    // Convert letters to digits and extract all digits
    let mut digits = String::new();
    for c in number.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else if c.is_ascii_alphabetic() {
            if let Some(digit) = letter_to_digit(c) {
                digits.push(digit);
            }
        }
        // Ignore other characters
    }

    // If there are 10 or fewer digits, return as-is
    if digits.len() <= 10 {
        digits
    } else {
        // Otherwise, return only the last 10
        digits.chars().rev().take(10).collect::<String>().chars().rev().collect()
    }
}

/// Creates the combined Warp routes (filters) for our endpoints.
///
/// Marked `pub` so integration tests in `tests/` can call it.
pub fn create_routes(
    store: Store,
    valid_tokens: Vec<String>,
    start_time: Instant,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // This filter checks if the Authorization header is a valid token
    let token_filter = warp::header::<String>("authorization")
        .map(move |token: String| valid_tokens.contains(&token))
        .boxed();

    // Reusable store filter
    let store_filter = warp::any().map(move || Arc::clone(&store));

    // /add endpoint
    let add_route = warp::path("add")
        .and(warp::post())
        .and(token_filter.clone())
        .and(store_filter.clone())
        .and(warp::body::json())
        .map(|is_valid: bool, store: Store, body: serde_json::Value| {
            if !is_valid {
                return warp::reply::json(&serde_json::json!({
                    "status": "error",
                    "message": "Invalid token"
                }));
            }

            let key = convert_to_ten_digits(body["key"].as_str().unwrap_or(""));
            let val = body["val"].as_str().unwrap_or("").to_string();

            let mut db = store.lock().unwrap();
            if db.contains_key(&key) {
                warp::reply::json(&serde_json::json!({
                    "status": "exists",
                    "message": "Number already texted"
                }))
            } else {
                db.insert(key, vec![val]);
                // Persist
                save_data(DATA_FILE, &db);
                warp::reply::json(&serde_json::json!({
                    "status": "added",
                    "message": "New number added"
                }))
            }
        });

    // /addmulti endpoint
    let addmulti_route = warp::path("addmulti")
        .and(warp::post())
        .and(token_filter.clone())
        .and(store_filter.clone())
        .and(warp::body::json())
        .map(|is_valid: bool, store: Store, body: serde_json::Value| {
            if !is_valid {
                return warp::reply::json(&serde_json::json!({
                    "status": "error",
                    "message": "Invalid token"
                }));
            }

            let key = convert_to_ten_digits(body["key"].as_str().unwrap_or(""));
            let val = body["val"].as_str().unwrap_or("").to_string();

            let mut db = store.lock().unwrap();
            match db.get_mut(&key) {
                Some(values) => {
                    if values.contains(&val) {
                        warp::reply::json(&serde_json::json!({
                            "status": "exists",
                            "message": "Number already texted from that sender"
                        }))
                    } else {
                        // Key exists, but add a new sender if < 2
                        if values.len() < 2 {
                            values.push(val);
                            save_data(DATA_FILE, &db);
                            warp::reply::json(&serde_json::json!({
                                "status": "added",
                                "message": "New sender added to existing key"
                            }))
                        } else {
                            warp::reply::json(&serde_json::json!({
                                "status": "exists",
                                "message": "Number already texted. Max senders reached."
                            }))
                        }
                    }
                }
                None => {
                    // Key doesn't exist yet
                    db.insert(key, vec![val]);
                    save_data(DATA_FILE, &db);
                    warp::reply::json(&serde_json::json!({
                        "status": "added",
                        "message": "New key/sender combination added"
                    }))
                }
            }
        });

    // /dump endpoint
    let dump_route = warp::path("dump")
        .and(warp::get())
        .and(token_filter.clone())
        .and(store_filter.clone())
        .map(|is_valid: bool, store: Store| -> Box<dyn warp::Reply> {
            if !is_valid {
                return Box::new(warp::reply::json(&serde_json::json!({
                    "status": "error",
                    "message": "Invalid token"
                })));
            }

            let db = store.lock().unwrap();
            let mut csv_data = String::from("phone_number,senders\n");
            for (k, v) in db.iter() {
                let senders = v.join("|");
                csv_data.push_str(&format!("{},{}\n", k, senders));
            }

            let response = warp::http::Response::builder()
                .header("Content-Type", "text/csv")
                .body(csv_data)
                .unwrap();

            Box::new(response)
        });

    // /clear endpoint
    let clear_route = warp::path("clear")
        .and(warp::post())
        .and(token_filter.clone())
        .and(store_filter.clone())
        .map(|is_valid: bool, store: Store| {
            if !is_valid {
                return warp::reply::json(&serde_json::json!({
                    "status": "error",
                    "message": "Invalid token"
                }));
            }

            // Attempt to archive data first
            match archive_data(DATA_FILE) {
                Ok(_) => println!("Data archived successfully."),
                Err(e) => {
                    eprintln!("Failed to archive data: {}", e);
                    return warp::reply::json(&serde_json::json!({
                        "status": "error",
                        "message": "Failed to archive data before clearing."
                    }));
                }
            }

            let mut db = store.lock().unwrap();
            db.clear();
            save_data(DATA_FILE, &db);
            warp::reply::json(&serde_json::json!({
                "status": "cleared",
                "message": "All data cleared and archived."
            }))
        });

    // /status endpoint
    let status_route = warp::path("status")
        .and(warp::get())
        .and(token_filter.clone())
        .and(store_filter.clone())
        .map(move |is_valid: bool, store: Store| {
            if !is_valid {
                return warp::reply::json(&serde_json::json!({
                    "status": "error",
                    "message": "Invalid token"
                }));
            }
            let db = store.lock().unwrap();
            let total_keys = db.len();
            let total_values: usize = db.values().map(|vals| vals.len()).sum();
            let uptime = Instant::now().duration_since(start_time);

            warp::reply::json(&serde_json::json!({
                "status": "ok",
                "keys": total_keys,
                "values": total_values,
                "uptime_seconds": uptime.as_secs()
            }))
        });

    // Combine them all
    add_route
        .or(addmulti_route)
        .or(dump_route)
        .or(clear_route)
        .or(status_route)
}

/// A simple wrapper for serialization/deserialization to/from JSON.
#[derive(Serialize, Deserialize)]
struct PersistData(HashMap<String, Vec<String>>);

/// Load data from the JSON file on disk into a `HashMap`.
///
/// Marked `pub` so integration tests (and main) can call it.
pub fn load_data(file_path: &str) -> HashMap<String, Vec<String>> {
    if let Ok(json_str) = fs::read_to_string(file_path) {
        if let Ok(parsed) = serde_json::from_str::<PersistData>(&json_str) {
            return parsed.0;
        }
    }
    HashMap::new()
}

/// Save the current store to disk as JSON.
///
/// Marked `pub` so integration tests (and main) can call it.
pub fn save_data(file_path: &str, data: &HashMap<String, Vec<String>>) {
    let wrapper = PersistData(data.clone());
    if let Ok(json_str) = serde_json::to_string_pretty(&wrapper) {
        let _ = fs::write(file_path, json_str);
    }
}
