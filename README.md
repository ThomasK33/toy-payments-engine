# Toy Payments Engine

This project is a simple payments engine written in Rust.
It processes transactions from a CSV file and maintains a ledger of accounts.

## Project Structure

- **samples/**: Contains sample data files.
  - `transactions.csv`: A sample CSV file with transactions.
  - `extensive.csv`: A more extensive sample CSV file with transactions.
- **src/**: Contains the source code.
  - `ledger.rs`: Implements the ledger and related functionalities.
  - `main.rs`: The entry point of the application.
  - `structs.rs`: Defines the data structures used in the project.
- **target/**: Contains build artifacts.

## Correctness

Correctness is ensured through comprehensive unit tests and validation checks.
Each transaction type is thoroughly tested with various scenarios to handle edge
cases and ensure accurate processing. Sample data for testing is included in the
[`samples/transactions.csv`](./samples/transactions.csv) file. The type system
and error handling mechanisms are leveraged to prevent invalid operations and
ensure data integrity.

## Getting Started

### Prerequisites

- Rust (latest stable version)

### Building the Project

To build the project, run:

```sh
cargo build
```

### Running the Project

To run the project, use:

```sh
cargo run -- samples/transactions.csv
```

To output the results to a file, use:

```sh
cargo run -- samples/transactions.csv > accounts.csv
```

### Running Tests

To run the tests, execute:

```sh
cargo test
```

## License

This project is licensed under the MIT License. See the LICENSE file for details.
