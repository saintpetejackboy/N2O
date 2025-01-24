use std::net::TcpListener;
use std::time::Instant;
use dotenv::dotenv;
use std::env;

use n2o::*; // or `use crate::lib::*;` depending on naming
use std::sync::Mutex;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let start_time = Instant::now();

    let chosen_port = choose_port(1337, 1338);
    let valid_tokens = env::var("VALID_TOKENS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Build your store
    let initial_data = load_data("n2o_data.json");
    let store = Store::new(Mutex::new(initial_data));

    // Create routes
    let routes = create_routes(store.clone(), valid_tokens, start_time);

    println!("Listening on port {}", chosen_port);
    warp::serve(routes).run(([0, 0, 0, 0], chosen_port)).await;
}

fn choose_port(primary: u16, fallback: u16) -> u16 {
    match TcpListener::bind(("0.0.0.0", primary)) {
        Ok(_) => primary,
        Err(_) => {
            eprintln!("WARNING: Port {} is in use; switching to {}.", primary, fallback);
            fallback
        }
    }
}
