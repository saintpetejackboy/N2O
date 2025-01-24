# N2O
![Crate Name](https://img.shields.io/badge/crate-n2o-blue.svg)
![Rust Edition](https://img.shields.io/badge/rust-2021-orange.svg) 
![Warp](https://img.shields.io/badge/Warp-0.3-blue.svg) 
![Serde](https://img.shields.io/badge/Serde-1.0-blue.svg) 
![Tokio](https://img.shields.io/badge/Tokio-1.0-orange.svg)

A Rust library and web service built with Warp, Serde, and Tokio.

## Status
![License: Unlicensed](https://img.shields.io/badge/license-Unlicensed-blue.svg)
![Rust](https://img.shields.io/badge/Made%20with-Rust-orange.svg)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)

## License
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [API Endpoints](#api-endpoints)
  - [Authentication](#authentication)
  - [/add](#add)
  - [/addmulti](#addmulti)
  - [/dump](#dump)
  - [/clear](#clear)
  - [/status](#status)
- [Testing](#testing)
- [Data Persistence](#data-persistence)
- [License](#license)
- [Contributing](#contributing)
- [Contact](#contact)

## Features

- **Add Phone Numbers**: Add single or multiple phone numbers with associated senders.
- **Data Dump**: Export all stored data in CSV format.
- **Data Archiving**: Automatically archives data before clearing.
- **Status Monitoring**: Retrieve service status, including uptime and data statistics.
- **Secure Access**: Token-based authentication for all endpoints.
- **Data Persistence**: In-memory data store with JSON file persistence.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.60 or later)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (comes with Rust)

### Clone the Repository

```bash
git clone https://github.com/saintpetejackboy/n2o.git
cd n2o
```

### Build the Project

```bash
cargo build --release
```

## Usage

After building the project, you can run the server using:

```bash
cargo run --release
```

By default, the server listens on `localhost:3030`. You can modify the address and port in the `main.rs` file as needed.

## API Endpoints

All endpoints require an `Authorization` header with a valid token. Tokens are managed internally and should be provided to authorized users only.

### Authentication

Include the `Authorization` header in your requests:

```
Authorization: your_token_here
```

### `/add` - Add a Single Phone Number

**Endpoint:** `/add`  
**Method:** `POST`  
**Description:** Adds a single phone number with an associated sender.

**Request Body:**

```json
{
  "key": "your_phone_number",
  "val": "sender_identifier"
}
```

**Response:**

- **Success:**

  ```json
  {
    "status": "added",
    "message": "New number added"
  }
  ```

- **Error (Invalid Token):**

  ```json
  {
    "status": "error",
    "message": "Invalid token"
  }
  ```

- **Error (Number Exists):**

  ```json
  {
    "status": "exists",
    "message": "Number already texted"
  }
  ```

### `/addmulti` - Add Multiple Senders to a Phone Number

**Endpoint:** `/addmulti`  
**Method:** `POST`  
**Description:** Adds a sender to an existing phone number. Each phone number can have up to two senders.

**Request Body:**

```json
{
  "key": "your_phone_number",
  "val": "sender_identifier"
}
```

**Response:**

- **Success:**

  ```json
  {
    "status": "added",
    "message": "New sender added to existing key"
  }
  ```

- **Error (Invalid Token):**

  ```json
  {
    "status": "error",
    "message": "Invalid token"
  }
  ```

- **Error (Sender Exists):**

  ```json
  {
    "status": "exists",
    "message": "Number already texted from that sender"
  }
  ```

- **Error (Max Senders Reached):**

  ```json
  {
    "status": "exists",
    "message": "Number already texted. Max senders reached."
  }
  ```

### `/dump` - Export Data as CSV

**Endpoint:** `/dump`  
**Method:** `GET`  
**Description:** Retrieves all stored data in CSV format.

**Response:**

- **Success:** Returns a CSV file with the following structure:

  ```
  phone_number,senders
  1234567890,sender1|sender2
  ```

- **Error (Invalid Token):**

  ```json
  {
    "status": "error",
    "message": "Invalid token"
  }
  ```

### `/clear` - Clear All Data

**Endpoint:** `/clear`  
**Method:** `POST`  
**Description:** Archives current data and clears all entries from the store.

**Response:**

- **Success:**

  ```json
  {
    "status": "cleared",
    "message": "All data cleared and archived."
  }
  ```

- **Error (Invalid Token):**

  ```json
  {
    "status": "error",
    "message": "Invalid token"
  }
  ```

- **Error (Archiving Failed):**

  ```json
  {
    "status": "error",
    "message": "Failed to archive data before clearing."
  }
  ```

### `/status` - Get Service Status

**Endpoint:** `/status`  
**Method:** `GET`  
**Description:** Retrieves the current status of the service, including uptime and data statistics.

**Response:**

- **Success:**

  ```json
  {
    "status": "ok",
    "keys": 150,
    "values": 300,
    "uptime_seconds": 12345
  }
  ```

- **Error (Invalid Token):**

  ```json
  {
    "status": "error",
    "message": "Invalid token"
  }
  ```

## Testing

The project includes comprehensive test cases to ensure functionality and reliability.

### Run All Tests

Execute the following command to run all tests:

```bash
cargo test
```

**Sample Output:**

```
running 15 tests
test test_add_endpoint ... ok
test test_add_endpoint_invalid_input ... ok
...
test test_status_uptime ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.00s
```

## Data Persistence

n2o uses an in-memory `HashMap` wrapped in an `Arc<Mutex<...>>` for thread-safe data storage. Data is persisted to a JSON file (`n2o_data.json`) to ensure durability across restarts. Additionally, before clearing data via the `/clear` endpoint, the current state is archived in a compressed `.json.gz` file with a timestamp.

### Data Archiving

Archived files are named using the format:

```
n2o_data_backup_YYYYMMDDHHMMSS.json.gz
```

These archives are stored in the project root directory.

## License

This project is **unlicensed**. You are free to use, modify, and distribute it as you see fit.

## Contributing

Contributions are welcome! If you have suggestions, improvements, or bug fixes, please feel free to open an issue or submit a pull request.

## Contact

Created by [saintpetejackboy](https://github.com/saintpetejackboy). Feel free to reach out for any inquiries or support.

---

**Note:** Ensure that you manage your `Authorization` tokens securely and avoid exposing them publicly. Tokens are essential for maintaining the security and integrity of the service.
