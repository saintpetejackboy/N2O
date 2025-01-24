// tests/tests.rs

use std::sync::Arc;
use std::time::Duration;

use warp::test::request;
use csv::ReaderBuilder;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use warp::Filter;
use n2o::{create_routes, convert_to_ten_digits, Store};


// ------------------- TESTS START HERE -------------------

/// Test basic behavior of our helper function.
#[test]
fn test_convert_to_ten_digits() {
	// Standard formats
	assert_eq!(convert_to_ten_digits("12345"), "12345"); // Fewer than 10 digits
	assert_eq!(convert_to_ten_digits("+1-555-123-4567"), "5551234567"); // Valid 10-digit US number
	assert_eq!(convert_to_ten_digits("1234567890123"), "4567890123"); // Over 10 digits

	// Edge cases
	assert_eq!(convert_to_ten_digits("(123) 456-7890"), "1234567890"); // US format with parentheses
	assert_eq!(convert_to_ten_digits("555.123.4567"), "5551234567"); // Dots as separators
	assert_eq!(convert_to_ten_digits("  555 123 4567  "), "5551234567"); // Leading/trailing spaces
	assert_eq!(convert_to_ten_digits("1 (800) FLOWERS"), "8003569377"); // Vanity number
	assert_eq!(convert_to_ten_digits("1111111111"), "1111111111"); // Exactly 10 digits

	// Invalid input edge cases
	assert_eq!(convert_to_ten_digits("abc123"), "222123"); // Non-digit characters translated
	assert_eq!(convert_to_ten_digits(""), ""); // Empty string
	assert_eq!(convert_to_ten_digits("---"), ""); // Only non-digits
}

/// Helper function to create routes with predefined tokens and store.
fn setup_routes() -> (impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone, Store, Vec<String>) {
	let store: Store = Arc::new(Mutex::new(HashMap::new()));
	let valid_tokens = vec!["validtoken".to_string(), "anothervalid".to_string()];
	let routes = create_routes(store.clone(), valid_tokens.clone(), Instant::now());
	(routes, store, valid_tokens)
}

/// Test the "add" endpoint with a valid token.
#[tokio::test]
async fn test_add_endpoint() {
	let (routes, store, _) = setup_routes();

	// Make a POST to /add with our JSON body
	let resp = request()
		.method("POST")
		.path("/add")
		.header("authorization", "validtoken")
		.json(&serde_json::json!({
			"key": "+1-555-123-4567",
			"val": "7272666666"
		}))
		.reply(&routes)
		.await;

	// Check the response
	assert_eq!(resp.status(), 200);
	let json_resp: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
	assert_eq!(json_resp["status"], "added");
	assert_eq!(json_resp["message"], "New number added");

	// Verify store was updated in memory
	let db = store.lock().unwrap();
	assert!(db.contains_key("5551234567"));
	assert_eq!(db["5551234567"], vec!["7272666666"]);
}

/// Test the "add" endpoint with an invalid token.
#[tokio::test]
async fn test_add_endpoint_invalid_token() {
	let (routes, store, _) = setup_routes();

	let resp = request()
		.method("POST")
		.path("/add")
		.header("authorization", "invalidtoken")
		.json(&serde_json::json!({
			"key": "some number",
			"val": "some val"
		}))
		.reply(&routes)
		.await;

	assert_eq!(resp.status(), 200);
	let json_resp: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
	assert_eq!(json_resp["status"], "error");
	assert_eq!(json_resp["message"], "Invalid token");

	// Ensure nothing got saved
	let db = store.lock().unwrap();
	assert!(db.is_empty());
}

/// Test the "addmulti" endpoint with a valid token.
#[tokio::test]
async fn test_addmulti_endpoint() {
	let (routes, store, _) = setup_routes();

	// Add first sender
	let resp1 = request()
		.method("POST")
		.path("/addmulti")
		.header("authorization", "validtoken")
		.json(&serde_json::json!({
			"key": "5551234567",
			"val": "7272666666"
		}))
		.reply(&routes)
		.await;

	assert_eq!(resp1.status(), 200);
	let json_resp1: serde_json::Value = serde_json::from_slice(resp1.body()).unwrap();
	assert_eq!(json_resp1["status"], "added");
	assert_eq!(json_resp1["message"], "New key/sender combination added");

	// Add second sender
	let resp2 = request()
		.method("POST")
		.path("/addmulti")
		.header("authorization", "validtoken")
		.json(&serde_json::json!({
			"key": "5551234567",
			"val": "7272555555"
		}))
		.reply(&routes)
		.await;

	assert_eq!(resp2.status(), 200);
	let json_resp2: serde_json::Value = serde_json::from_slice(resp2.body()).unwrap();
	assert_eq!(json_resp2["status"], "added");
	assert_eq!(json_resp2["message"], "New sender added to existing key");

	// Attempt to add a third sender, which should fail
	let resp3 = request()
		.method("POST")
		.path("/addmulti")
		.header("authorization", "validtoken")
		.json(&serde_json::json!({
			"key": "5551234567",
			"val": "7272333333"
		}))
		.reply(&routes)
		.await;

	assert_eq!(resp3.status(), 200);
	let json_resp3: serde_json::Value = serde_json::from_slice(resp3.body()).unwrap();
	assert_eq!(json_resp3["status"], "exists");
	assert_eq!(json_resp3["message"], "Number already texted. Max senders reached.");

	// Verify store state
	let db = store.lock().unwrap();
	assert!(db.contains_key("5551234567"));
	assert_eq!(db["5551234567"], vec!["7272666666", "7272555555"]);
}

/// Test the "addmulti" endpoint with an invalid token.
#[tokio::test]
async fn test_addmulti_endpoint_invalid_token() {
	let (routes, store, _) = setup_routes();

	let resp = request()
		.method("POST")
		.path("/addmulti")
		.header("authorization", "invalidtoken")
		.json(&serde_json::json!({
			"key": "5551234567",
			"val": "7272666666"
		}))
		.reply(&routes)
		.await;

	assert_eq!(resp.status(), 200);
	let json_resp: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
	assert_eq!(json_resp["status"], "error");
	assert_eq!(json_resp["message"], "Invalid token");

	// Ensure nothing got saved
	let db = store.lock().unwrap();
	assert!(db.is_empty());
}

/// Test the "dump" endpoint with a valid token.
#[tokio::test]
async fn test_dump_endpoint() {
	let (routes, store, _) = setup_routes();

	// Prepopulate the store
	{
		let mut db = store.lock().unwrap();
		db.insert("5551234567".to_string(), vec!["7272666666".to_string(), "7272555555".to_string()]);
		db.insert("5557654321".to_string(), vec!["SENDER3".to_string()]);
	}

	let resp = request()
		.method("GET")
		.path("/dump")
		.header("authorization", "validtoken")
		.reply(&routes)
		.await;

	assert_eq!(resp.status(), 200);
	assert_eq!(resp.headers()["content-type"], "text/csv");

	let csv_content = std::str::from_utf8(resp.body()).unwrap();
	let mut rdr = ReaderBuilder::new()
		.has_headers(true)
		.from_reader(csv_content.as_bytes());

	// Read the header separately:
	let header = rdr.headers().unwrap();
	assert_eq!(header.iter().collect::<Vec<_>>(), vec!["phone_number", "senders"]);


	// Now read data lines:
	let mut records = rdr.records();
	let record1 = records.next().unwrap().unwrap();
	let record2 = records.next().unwrap().unwrap();

	let mut entries = vec![record1, record2];
	entries.sort_by(|a, b| a[0].cmp(&b[0])); // Sort by phone_number

	assert_eq!(entries[0], csv::StringRecord::from(vec!["5551234567", "7272666666|7272555555"]));
	assert_eq!(entries[1], csv::StringRecord::from(vec!["5557654321", "SENDER3"]));
}


/// Test the "dump" endpoint with an invalid token.
#[tokio::test]
async fn test_dump_endpoint_invalid_token() {
	let (routes, store, _) = setup_routes();

	// Prepopulate the store
	{
		let mut db = store.lock().unwrap();
		db.insert("5551234567".to_string(), vec!["7272666666".to_string()]);
	}

	let resp = request()
		.method("GET")
		.path("/dump")
		.header("authorization", "invalidtoken")
		.reply(&routes)
		.await;

	assert_eq!(resp.status(), 200);
	let json_resp: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
	assert_eq!(json_resp["status"], "error");
	assert_eq!(json_resp["message"], "Invalid token");
}

/// Test the "clear" endpoint with a valid token.
#[tokio::test]
async fn test_clear_endpoint() {
	let (routes, store, _) = setup_routes();

	// Prepopulate the store
	{
		let mut db = store.lock().unwrap();
		db.insert("5551234567".to_string(), vec!["7272666666".to_string()]);
		db.insert("5557654321".to_string(), vec!["7272555555".to_string()]);
	}

	let resp = request()
		.method("POST")
		.path("/clear")
		.header("authorization", "validtoken")
		.reply(&routes)
		.await;

	assert_eq!(resp.status(), 200);
	let clear_json: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
	assert_eq!(clear_json["status"], "cleared");
	assert_eq!(clear_json["message"], "All data cleared and archived."); // Ensure clear_json is defined
}

/// Test the "clear" endpoint with an invalid token.
#[tokio::test]
async fn test_clear_endpoint_invalid_token() {
	let (routes, store, _) = setup_routes();

	// Prepopulate the store
	{
		let mut db = store.lock().unwrap();
		db.insert("5551234567".to_string(), vec!["7272666666".to_string()]);
	}

	let resp = request()
		.method("POST")
		.path("/clear")
		.header("authorization", "invalidtoken")
		.reply(&routes)
		.await;

	assert_eq!(resp.status(), 200);
	let json_resp: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
	assert_eq!(json_resp["status"], "error");
	assert_eq!(json_resp["message"], "Invalid token");

	// Ensure the store is not cleared
	let db = store.lock().unwrap();
	assert!(!db.is_empty());
}

/// Test the "status" endpoint with a valid token.
#[tokio::test]
async fn test_status_endpoint() {
	let (routes, store, _) = setup_routes();

	// Prepopulate the store
	{
		let mut db = store.lock().unwrap();
		db.insert("5551234567".to_string(), vec!["7272666666".to_string(), "7272555555".to_string()]);
		db.insert("5557654321".to_string(), vec!["SENDER3".to_string()]);
	}

	let resp = request()
		.method("GET")
		.path("/status")
		.header("authorization", "validtoken")
		.reply(&routes)
		.await;

	assert_eq!(resp.status(), 200);
	let json_resp: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
	assert_eq!(json_resp["status"], "ok");
	assert_eq!(json_resp["keys"], 2);
	assert_eq!(json_resp["values"], 3);
}

/// Test the "status" endpoint with an invalid token.
#[tokio::test]
async fn test_status_endpoint_invalid_token() {
	let (routes, store, _) = setup_routes();

	// Prepopulate the store
	{
		let mut db = store.lock().unwrap();
		db.insert("5551234567".to_string(), vec!["7272666666".to_string()]);
	}

	let resp = request()
		.method("GET")
		.path("/status")
		.header("authorization", "invalidtoken")
		.reply(&routes)
		.await;

	assert_eq!(resp.status(), 200);
	let json_resp: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
	assert_eq!(json_resp["status"], "error");
	assert_eq!(json_resp["message"], "Invalid token");
}

/// Test the full flow: add, addmulti, status, dump, clear.
#[tokio::test]
async fn test_full_flow() {
	let (routes, store, _) = setup_routes();

	// Add a number using /add
	let add_resp = request()
		.method("POST")
		.path("/add")
		.header("authorization", "validtoken")
		.json(&serde_json::json!({
			"key": "5551234567",
			"val": "7272666666"
		}))
		.reply(&routes)
		.await;

	assert_eq!(add_resp.status(), 200);
	let add_json: serde_json::Value = serde_json::from_slice(add_resp.body()).unwrap();
	assert_eq!(add_json["status"], "added");

	// Add another sender using /addmulti
	let addmulti_resp = request()
		.method("POST")
		.path("/addmulti")
		.header("authorization", "validtoken")
		.json(&serde_json::json!({
			"key": "5551234567",
			"val": "7272555555"
		}))
		.reply(&routes)
		.await;

	assert_eq!(addmulti_resp.status(), 200);
	let addmulti_json: serde_json::Value = serde_json::from_slice(addmulti_resp.body()).unwrap();
	assert_eq!(addmulti_json["status"], "added");

	// Check status
	let status_resp = request()
		.method("GET")
		.path("/status")
		.header("authorization", "validtoken")
		.reply(&routes)
		.await;

	assert_eq!(status_resp.status(), 200);
	let status_json: serde_json::Value = serde_json::from_slice(status_resp.body()).unwrap();
	assert_eq!(status_json["status"], "ok");
	assert_eq!(status_json["keys"], 1);
	assert_eq!(status_json["values"], 2);

	// Dump the data
	let dump_resp = request()
		.method("GET")
		.path("/dump")
		.header("authorization", "validtoken")
		.reply(&routes)
		.await;

	assert_eq!(dump_resp.status(), 200);
	let expected_csv = "phone_number,senders\n5551234567,7272666666|7272555555\n";
	assert_eq!(dump_resp.body(), expected_csv.as_bytes());

	// Clear the data with timeout
	let clear_resp = tokio::time::timeout(Duration::from_secs(5), request()
		.method("POST")
		.path("/clear")
		.header("authorization", "validtoken")
		.reply(&routes))
		.await
		.expect("Clear request timed out"); // Removed the second `.await`

	assert_eq!(clear_resp.status(), 200);
	let clear_json: serde_json::Value = serde_json::from_slice(clear_resp.body()).unwrap();
	assert_eq!(clear_json["status"], "cleared");
	assert_eq!(clear_json["message"], "All data cleared and archived.");

	// Verify the store is empty
	{
		let db = store.lock().unwrap();
		assert!(db.is_empty());
	}

	// Verify status reflects cleared state
	let final_status_resp = request()
		.method("GET")
		.path("/status")
		.header("authorization", "validtoken")
		.reply(&routes)
		.await;

	assert_eq!(final_status_resp.status(), 200);
	let final_status_json: serde_json::Value = serde_json::from_slice(final_status_resp.body()).unwrap();
	assert_eq!(final_status_json["status"], "ok");
	assert_eq!(final_status_json["keys"], 0);
	assert_eq!(final_status_json["values"], 0);
}


/// Test that multiple valid tokens are accepted.
#[tokio::test]
async fn test_multiple_valid_tokens() {
	let (routes, store, valid_tokens) = setup_routes();

	for token in valid_tokens.iter() {
		let resp = request()
			.method("POST")
			.path("/add")
			.header("authorization", token)
			.json(&serde_json::json!({
				"key": "5551234567",
				"val": "7272666666"
			}))
			.reply(&routes)
			.await;

		assert_eq!(resp.status(), 200);
		let json_resp: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
		// First token should add successfully
		if token == "validtoken" {
			assert_eq!(json_resp["status"], "added");
		} else {
			// Subsequent tokens should see the key already exists
			assert_eq!(json_resp["status"], "exists");
		}
	}

	// Verify store state
	let db = store.lock().unwrap();
	assert!(db.contains_key("5551234567"));
	assert_eq!(db["5551234567"], vec!["7272666666"]);
}

/// Test the "add" endpoint with invalid input data.
#[tokio::test]
async fn test_add_endpoint_invalid_input() {
	let (routes, _store, _) = setup_routes();

	// Missing "key" field
	let resp_missing_key = request()
		.method("POST")
		.path("/add")
		.header("authorization", "validtoken")
		.json(&serde_json::json!({
			"val": "7272666666"
		}))
		.reply(&routes)
		.await;

	assert_eq!(resp_missing_key.status(), 200);
	let json_resp1: serde_json::Value = serde_json::from_slice(resp_missing_key.body()).unwrap();
	assert_eq!(json_resp1["status"], "added"); // convert_to_ten_digits returns empty string
	assert_eq!(json_resp1["message"], "New number added");

	// Missing "val" field
	let resp_missing_val = request()
		.method("POST")
		.path("/add")
		.header("authorization", "validtoken")
		.json(&serde_json::json!({
			"key": "5551234567"
		}))
		.reply(&routes)
		.await;

	assert_eq!(resp_missing_val.status(), 200);
	let json_resp2: serde_json::Value = serde_json::from_slice(resp_missing_val.body()).unwrap();
	assert_eq!(json_resp2["status"], "added");
	assert_eq!(json_resp2["message"], "New number added");

	// Invalid JSON
	let resp_invalid_json = request()
		.method("POST")
		.path("/add")
		.header("authorization", "validtoken")
		.body("not a json")
		.reply(&routes)
		.await;

	assert_eq!(resp_invalid_json.status(), 400); // Warp returns 400 for bad JSON
}

/// Test the "status" endpoint uptime.
#[tokio::test]
async fn test_status_uptime() {
	let (routes, _store, _) = setup_routes();

	// Wait for a short duration
	tokio::time::sleep(Duration::from_secs(2)).await;

	let resp = request()
		.method("GET")
		.path("/status")
		.header("authorization", "validtoken")
		.reply(&routes)
		.await;

	assert_eq!(resp.status(), 200);
	
	let json_resp: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
	assert_eq!(json_resp["status"], "ok");
	assert_eq!(json_resp["keys"], 0);
	assert_eq!(json_resp["values"], 0);
}
