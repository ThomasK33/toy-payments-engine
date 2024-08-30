use std::fmt::Display;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

// CSV file contents

#[derive(Debug, PartialEq, Deserialize)]
pub struct Record {
    #[serde(rename = "type")]
    pub record_type: RecordType,

    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

impl Record {
    pub fn validate(&self) -> anyhow::Result<()> {
        if let (RecordType::Deposit | RecordType::Withdrawal, None) =
            (&self.record_type, self.amount)
        {
            return Err(anyhow!("Missing amount in record"));
        };

        if let (RecordType::Chargeback | RecordType::Resolve | RecordType::Dispute, Some(_)) =
            (&self.record_type, self.amount)
        {
            return Err(anyhow!(
                "Chargeback / Resolve / Dispute record may not contain amount"
            ));
        };

        Ok(())
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            RecordType::Deposit => "deposit",
            RecordType::Withdrawal => "withdrawal",
            RecordType::Dispute => "dispute",
            RecordType::Resolve => "resolve",
            RecordType::Chargeback => "chargeback",
        };

        write!(f, "{name}")
    }
}

// Outputs

#[derive(Debug, Serialize)]
pub struct ClientRecord {
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_deserialization() {
        let data = include_str!("../samples/transactions.csv").trim();

        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .has_headers(true)
            .from_reader(data.as_bytes());

        let results: Vec<Record> = reader.deserialize().filter_map(|a| a.ok()).collect();

        assert_eq!(
            results,
            vec![
                Record {
                    record_type: RecordType::Deposit,
                    client: 1,
                    tx: 1,
                    amount: Some(1.0)
                },
                Record {
                    record_type: RecordType::Deposit,
                    client: 2,
                    tx: 2,
                    amount: Some(2.0)
                },
                Record {
                    record_type: RecordType::Deposit,
                    client: 3,
                    tx: 3,
                    amount: Some(4.1234)
                },
                Record {
                    record_type: RecordType::Withdrawal,
                    client: 3,
                    tx: 4,
                    amount: Some(4.0)
                },
                Record {
                    record_type: RecordType::Dispute,
                    client: 1,
                    tx: 1,
                    amount: None
                },
                Record {
                    record_type: RecordType::Resolve,
                    client: 1,
                    tx: 1,
                    amount: None
                },
                Record {
                    record_type: RecordType::Dispute,
                    client: 2,
                    tx: 2,
                    amount: None
                },
                Record {
                    record_type: RecordType::Chargeback,
                    client: 2,
                    tx: 2,
                    amount: None
                },
            ]
        );
    }

    #[test]
    fn test_record_valid() {
        let data = "\
            type, client, tx, amount
            deposit, 1, 1, 1.0
        ";

        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .has_headers(true)
            .from_reader(data.as_bytes());

        let results: Vec<Record> = reader.deserialize().filter_map(|a| a.ok()).collect();

        assert!(results.into_iter().all(|record| record.validate().is_ok()));
    }

    #[test]
    fn test_record_invalid() {
        let data = "\
            type, client, tx, amount
            deposit, 1, 1, 
        ";

        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .has_headers(true)
            .from_reader(data.as_bytes());

        let results: Vec<Record> = reader.deserialize().filter_map(|a| a.ok()).collect();

        assert_eq!(
            results
                .into_iter()
                .filter(|record| record.validate().is_err())
                .count(),
            1
        );
    }

    #[test]
    fn test_record_invalid_2() {
        let data = "\
            type, client, tx, amount
            dispute, 1, 1, 1
            resolve, 1, 1,
            dispute, 2, 2, 2.1234
            chargeback, 2, 2, 0.1
        ";

        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .has_headers(true)
            .from_reader(data.as_bytes());

        let results: Vec<Record> = reader.deserialize().filter_map(|a| a.ok()).collect();

        assert_eq!(
            results
                .into_iter()
                .filter(|record| record.validate().is_err())
                .count(),
            3
        );
    }
}
