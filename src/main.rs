#![forbid(unsafe_code)]

use std::{env, io};

mod ledger;
mod structs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    if args.len() != 2 {
        eprint!("Incorrect amount of arguments passed. Please only pass the transaction csv file path as first argument.");
        return Ok(());
    }

    let file_path = args.next_back().expect("Missing csv file path");

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .has_headers(true)
        .from_path(file_path)?;

    let mut account_ledger = ledger::Tracker::new();

    for record in reader.deserialize::<structs::Record>() {
        let record = match record {
            Ok(r) => r,
            Err(err) => {
                eprintln!("Failed to process the record because of: {err}");
                continue;
            }
        };
        if let Err(err) = record.validate() {
            eprintln!("Failed to verify the record: {err}");
            continue;
        }

        let outcome = match record.record_type {
            structs::RecordType::Deposit => account_ledger
                .get_or_create_customer(record.client)
                .deposit(record.tx, record.amount.unwrap()),
            structs::RecordType::Withdrawal => account_ledger
                .get_or_create_customer(record.client)
                .withdraw(record.tx, record.amount.unwrap()),
            structs::RecordType::Dispute => account_ledger
                .get_or_create_customer(record.client)
                .dispute(record.tx),
            structs::RecordType::Resolve => account_ledger
                .get_or_create_customer(record.client)
                .resolve(record.tx),
            structs::RecordType::Chargeback => account_ledger
                .get_or_create_customer(record.client)
                .chargeback(record.tx),
        };

        if let Err(err) = outcome {
            eprintln!(
                "Failed to perform {} on account {}: {err}",
                record.record_type, record.client
            );
        };
    }

    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_writer(io::stdout());

    for account in account_ledger.printable_accounts() {
        writer.serialize(account)?;
    }

    writer.flush()?;

    Ok(())
}
