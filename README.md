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

- **Add Phone Numbers**: Add single or multiple phone numbers with associated senders, both converted to 10-digit representations.
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
**Description:** Adds a single phone number with an associated sender. Both the `key` (phone number) and `val` (sender identifier) are converted to their 10-digit representations before storage.

**Request Body:**

```json
{
  "key": "your_phone_number",
  "val": "sender_identifier"
}
```

**Behavior:**

- **Key Conversion:** The `key` field is converted to a 10-digit number using the `convert_to_ten_digits` function. Non-alphanumeric characters are ignored, and letters are mapped to their corresponding phone keypad digits.
- **Value Conversion:** The `val` field is also converted to a 10-digit number following the same conversion logic.

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

- **Error (Invalid Input):**

  ```json
  {
    "status": "error",
    "message": "Both key and value must be 10 digits after conversion."
  }
  ```

**Example Request:**

```bash
curl -X POST http://localhost:3030/add \
  -H "Content-Type: application/json" \
  -H "Authorization: your_token_here" \
  -d '{
        "key": "1-800-FLOWERS",
        "val": "SUPPORT"
      }'
```

**Example Processing:**

- **Key Conversion:**
  - Input: `"1-800-FLOWERS"`
  - Conversion: `"18003569377"` → Truncated to last 10 digits: `"8003569377"`

- **Value Conversion:**
  - Input: `"SUPPORT"`
  - Conversion: `"7877768"` → Padded or handled as per `convert_to_ten_digits` logic.

### `/addmulti` - Add Multiple Senders to a Phone Number

**Endpoint:** `/addmulti`  
**Method:** `POST`  
**Description:** Adds a sender to an existing phone number. Both the `key` and `val` are converted to 10-digit representations. Each phone number can have up to two senders.

**Request Body:**

```json
{
  "key": "your_phone_number",
  "val": "sender_identifier"
}
```

**Behavior:**

- **Key Conversion:** Converts the `key` to a 10-digit number.
- **Value Conversion:** Converts the `val` to a 10-digit number.
- **Sender Limits:** Each phone number can have a maximum of two unique senders.

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

- **Error (Invalid Input):**

  ```json
  {
    "status": "error",
    "message": "Both key and value must be 10 digits after conversion."
  }
  ```

**Example Request:**

```bash
curl -X POST http://localhost:3030/addmulti \
  -H "Content-Type: application/json" \
  -H "Authorization: your_token_here" \
  -d '{
        "key": "1-800-FLOWERS",
        "val": "HELPDESK"
      }'
```

**Example Processing:**

- **Key Conversion:**
  - Input: `"1-800-FLOWERS"`
  - Conversion: `"18003569377"` → Truncated to last 10 digits: `"8003569377"`

- **Value Conversion:**
  - Input: `"HELPDESK"`
  - Conversion: `"4357375"` → Padded or handled as per `convert_to_ten_digits` logic.

### `/dump` - Export Data as CSV

**Endpoint:** `/dump`  
**Method:** `GET`  
**Description:** Retrieves all stored data in CSV format.

**Response:**

- **Success:** Returns a CSV file with the following structure:

  ```
  phone_number,senders
  8003569377,4357375|...
  ```

- **Error (Invalid Token):**

  ```json
  {
    "status": "error",
    "message": "Invalid token"
  }
  ```

**Example Request:**

```bash
curl -X GET http://localhost:3030/dump \
  -H "Authorization: your_token_here" \
  -o data.csv
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

**Example Request:**

```bash
curl -X GET http://localhost:3030/status \
  -H "Authorization: your_token_here"
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

N2O uses an in-memory `HashMap` wrapped in an `Arc<Mutex<...>>` for thread-safe data storage. Data is persisted to a JSON file (`n2o_data.json`) to ensure durability across restarts. Additionally, before clearing data via the `/clear` endpoint, the current state is archived in a compressed `.json.gz` file with a timestamp.

### Data Archiving

Archived files are named using the format:

```
n2o_data_backup_YYYYMMDDHHMMSS.json.gz
```

These archives are stored in the project root directory.

### 10-Digit Conversion

Both **keys** (phone numbers) and **values** (sender identifiers) are converted to their 10-digit representations before being stored. This ensures consistency and standardization across all data entries.

- **Conversion Logic:**
  - **Digits:** Retained as-is.
  - **Letters:** Mapped to their corresponding phone keypad digits (e.g., A, B, C → 2).
  - **Others:** Ignored.
  - **Length Handling:**
    - If the resulting number has **10 or fewer digits**, it is stored as-is.
    - If it has **more than 10 digits**, only the **last 10 digits** are retained.

**Example:**

- **Key Conversion:**
  - Input: `"1-800-FLOWERS"`
  - Conversion: `"18003569377"` → Truncated to `"8003569377"`

- **Value Conversion:**
  - Input: `"SUPPORT"`
  - Conversion: `"7877768"` → Stored as `"7877768"`

**Note:** Ensure that both `key` and `val` fields result in exactly 10 digits after conversion. If not, consider implementing additional validation or padding mechanisms as needed.

## License

This project is **unlicensed**. You are free to use, modify, and distribute it as you see fit.

## Contributing

Contributions are welcome! If you have suggestions, improvements, or bug fixes, please feel free to open an issue or submit a pull request.

### Guidelines

1. **Fork the Repository:** Create your own fork of the project.
2. **Create a Feature Branch:** `git checkout -b feature/YourFeature`
3. **Commit Your Changes:** Ensure your code follows the project's coding standards.
4. **Push to Your Fork:** `git push origin feature/YourFeature`
5. **Open a Pull Request:** Provide a clear description of your changes and the problem they solve.

## Contact

Created by [saintpetejackboy](https://github.com/saintpetejackboy). Feel free to reach out for any inquiries or support.

---

**Note:** Ensure that you manage your `Authorization` tokens securely and avoid exposing them publicly. Tokens are essential for maintaining the security and integrity of the service.

