#![forbid(unsafe_code)]

use std::{env, io};

use anyhow::anyhow;

mod account;
mod structs;

fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    if args.len() != 2 {
        return Err(anyhow!(
            "Expected exactly one argument: the path to the transaction csv file."
        ));
    }

    let file_path = args.next_back().expect("No arguments provided");

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .has_headers(true)
        .from_path(file_path)?;

    let mut account_ledger = account::Ledger::new();

    for result in reader.deserialize::<structs::Record>() {
        let record = match result {
            Ok(r) => r,
            Err(err) => {
                eprintln!("Failed to deserialize record: {err}");
                continue;
            }
        };
        if let Err(err) = record.validate() {
            eprintln!("Failed to validate the record: {err}");
            continue;
        }

        let account = account_ledger.get_or_insert_customer(record.client);

        let outcome = match record.record_type {
            structs::RecordType::Deposit => {
                let amount = record.amount.ok_or(anyhow!("Missing amount for deposit"))?;
                account.deposit(record.tx, amount)
            }
            structs::RecordType::Withdrawal => {
                let amount = record
                    .amount
                    .ok_or(anyhow!("Missing amount for withdrawal"))?;
                account.withdraw(record.tx, amount)
            }
            structs::RecordType::Dispute => account.dispute(record.tx),
            structs::RecordType::Resolve => account.resolve(record.tx),
            structs::RecordType::Chargeback => account.chargeback(record.tx),
        };

        if let Err(err) = outcome {
            eprintln!(
                "Failed to perform {} operation with transaction {} on account {}: {}",
                record.record_type, record.tx, record.client, err
            );
        };
    }

    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_writer(io::stdout());

    for account in account_ledger.client_records() {
        writer.serialize(account)?;
    }

    writer.flush()?;

    Ok(())
}
