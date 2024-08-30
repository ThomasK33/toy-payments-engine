use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize)]
pub struct Record {
    #[serde(rename = "type")]
    pub record_type: RecordType,

    pub client: u16,
    pub tx: u32,
    pub amount: Option<f32>,
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

#[derive(Debug, Serialize)]
pub struct ClientRecord {
    pub client: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_deserialization() {
        let data = "\
            type, client, tx, amount
            deposit, 1, 1, 1.0
            deposit, 2, 2, 2.0
            deposit, 3, 3, 4.1234
            withdrawal, 3, 4, 4,1234
            dispute, 1, 1
            resolve, 1, 1,
            dispute, 2, 2
            chargeback, 2, 2
        "
        .trim();

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
}
