use std::collections::HashMap;

use anyhow::anyhow;

use crate::structs;

pub struct Tracker {
    map: HashMap<u16, Customer>,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get_or_create_customer(&mut self, client_id: u16) -> &mut Customer {
        self.map.entry(client_id).or_default()
    }

    pub fn printable_accounts(&self) -> Vec<structs::ClientRecord> {
        self.map
            .iter()
            .map(|(&client, customer)| structs::ClientRecord {
                client,
                available: customer.total - customer.held,
                held: customer.held,
                total: customer.total,
                locked: customer.locked,
            })
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct Customer {
    total: f64,
    held: f64,
    locked: bool,

    /// Records is a map of performed deposits or withdrawals.
    /// Positive amount indicates a deposit, while negative
    /// ones represent a withdrawal.
    records: HashMap<u32, f64>,

    disputed_transactions: Vec<u32>,
}

impl Customer {
    pub fn deposit(&mut self, tx: u32, amount: f64) -> anyhow::Result<()> {
        if amount < 0_f64 {
            return Err(anyhow!("amount has to be positive"));
        }
        if self.records.contains_key(&tx) {
            return Err(anyhow!(
                "Customer already has a transaction with this tx id"
            ));
        }
        if self.locked {
            return Err(anyhow!("This account is locked"));
        }

        self.records.insert(tx, amount);
        self.total += amount;

        Ok(())
    }

    pub fn withdraw(&mut self, tx: u32, amount: f64) -> anyhow::Result<()> {
        if amount < 0_f64 {
            return Err(anyhow!("amount has to be positive"));
        }
        if self.records.contains_key(&tx) {
            return Err(anyhow!(
                "Customer already has a transaction with this tx id"
            ));
        }
        if self.locked {
            return Err(anyhow!("This account is locked"));
        }
        if amount > (self.total - self.held) {
            return Err(anyhow!("Insufficient funds"));
        }

        self.records.insert(tx, -amount);
        self.total -= amount;

        Ok(())
    }

    pub fn dispute(&mut self, tx: u32) -> anyhow::Result<()> {
        if !self.records.contains_key(&tx) {
            return Err(anyhow!(
                "Customer does not has a transaction with this tx id"
            ));
        }
        if self.disputed_transactions.contains(&tx) {
            return Err(anyhow!("Transaction is already disputed"));
        }

        let Some(amount) = self.records.get(&tx) else {
            return Err(anyhow!("No transaction record found for the given id"));
        };

        self.held += amount;
        self.disputed_transactions.push(tx);

        Ok(())
    }

    pub fn resolve(&mut self, tx: u32) -> anyhow::Result<()> {
        if !self.records.contains_key(&tx) {
            return Err(anyhow!(
                "Customer does not has a transaction with this tx id"
            ));
        }
        if !self.disputed_transactions.contains(&tx) {
            return Err(anyhow!("Transaction is not disputed"));
        }

        let Some(amount) = self.records.get(&tx) else {
            return Err(anyhow!("No transaction record found for the given id"));
        };

        self.held -= amount;
        if let Some(index) = self.disputed_transactions.iter().position(|a| a == &tx) {
            self.disputed_transactions.swap_remove(index);
        };

        Ok(())
    }

    pub fn chargeback(&mut self, tx: u32) -> anyhow::Result<()> {
        if !self.records.contains_key(&tx) {
            return Err(anyhow!(
                "Customer does not has a transaction with this tx id"
            ));
        }
        if !self.disputed_transactions.contains(&tx) {
            return Err(anyhow!("Transaction is not disputed"));
        }

        let Some(amount) = self.records.get(&tx) else {
            return Err(anyhow!("No transaction record found for the given id"));
        };

        self.held -= amount;
        self.total -= amount;
        self.locked = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;

        assert_eq!(customer.total, 2.);

        Ok(())
    }

    #[test]
    fn test_deposit_2() {
        let mut customer = Customer {
            locked: true,
            ..Default::default()
        };
        let is_err = customer.deposit(1, 2.).is_err();

        assert!(is_err);
    }

    #[test]
    fn test_withdrawal() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;
        assert_eq!(customer.total, 2.);

        customer.withdraw(2, 1.)?;
        assert_eq!(customer.total, 1.);

        Ok(())
    }

    #[test]
    fn test_withdrawal_2() {
        let mut customer = Customer::default();
        let outcome = customer.withdraw(2, 1.).is_err();
        assert!(outcome);
    }

    #[test]
    fn test_withdrawal_3() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;
        assert_eq!(customer.total, 2.);

        customer.locked = true;

        let is_err = customer.withdraw(2, 1.).is_err();
        assert!(is_err);

        Ok(())
    }

    #[test]
    fn test_withdrawal_4() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;
        assert_eq!(customer.total, 2.);

        customer.withdraw(2, 1.)?;
        let is_err = customer.withdraw(2, 1.).is_err();
        assert!(is_err);

        Ok(())
    }

    #[test]
    fn test_dispute() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;
        assert_eq!(customer.total, 2.);

        customer.dispute(1)?;
        assert_eq!(customer.total, 2.);
        assert_eq!(customer.held, 2.);

        Ok(())
    }

    #[test]
    fn test_dispute_withdrawal() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;
        assert_eq!(customer.total, 2.);
        customer.deposit(2, 1.)?;
        assert_eq!(customer.total, 3.);

        customer.dispute(1)?;
        assert_eq!(customer.total, 3.);
        assert_eq!(customer.held, 2.);

        customer.withdraw(3, 1.)?;
        assert_eq!(customer.total, 2.);
        assert_eq!(customer.held, 2.);

        Ok(())
    }

    #[test]
    fn test_dispute_fail_withdrawal() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;
        assert_eq!(customer.total, 2.);

        customer.dispute(1)?;
        assert_eq!(customer.total, 2.);
        assert_eq!(customer.held, 2.);

        let is_err = customer.withdraw(2, 1.).is_err();
        assert!(is_err);

        Ok(())
    }

    #[test]
    fn test_dispute_without_tx() {
        let mut customer = Customer::default();

        let is_err = customer.dispute(1).is_err();
        assert!(is_err);
    }

    #[test]
    fn test_resolve() -> anyhow::Result<()> {
        let mut customer = Customer::default();

        customer.deposit(1, 2.)?;
        customer.deposit(2, 3.)?;
        assert_eq!(customer.total, 5.);

        customer.dispute(1)?;
        assert_eq!(customer.total, 5.);
        assert_eq!(customer.held, 2.);

        customer.resolve(1)?;
        assert_eq!(customer.total, 5.);
        assert_eq!(customer.held, 0.);
        assert_eq!(customer.disputed_transactions.len(), 0);
        assert!(!customer.locked);

        Ok(())
    }

    #[test]
    fn test_resolve_without_tx() {
        let mut customer = Customer::default();

        let is_err = customer.resolve(1).is_err();
        assert!(is_err);
    }

    #[test]
    fn test_chargeback() -> anyhow::Result<()> {
        let mut customer = Customer::default();

        customer.deposit(1, 2.)?;
        customer.deposit(2, 3.)?;
        assert_eq!(customer.total, 5.);

        customer.dispute(1)?;
        assert_eq!(customer.total, 5.);
        assert_eq!(customer.held, 2.);

        customer.chargeback(1)?;
        assert_eq!(customer.total, 3.);
        assert_eq!(customer.held, 0.);
        assert_eq!(customer.disputed_transactions.len(), 1);
        assert!(customer.locked);

        Ok(())
    }

    #[test]
    fn test_chargeback_without_dispute() -> anyhow::Result<()> {
        let mut customer = Customer::default();

        customer.deposit(1, 2.)?;
        customer.deposit(2, 3.)?;
        assert_eq!(customer.total, 5.);

        let is_err = customer.chargeback(1).is_err();
        assert!(is_err);
        assert_eq!(customer.total, 5.);
        assert_eq!(customer.held, 0.);
        assert_eq!(customer.disputed_transactions.len(), 0);
        assert!(!customer.locked);

        Ok(())
    }
    #[test]
    fn test_chargeback_without_tx() {
        let mut customer = Customer::default();

        let is_err = customer.chargeback(1).is_err();
        assert!(is_err);
    }

    #[test]
    fn test_new_tracker() {
        let tracker = Tracker::new();
        assert!(tracker.map.is_empty());
    }

    #[test]
    fn test_tracker_get_or_create_customer() {
        let mut tracker = Tracker::new();
        let client_id = 1;
        let customer = tracker.get_or_create_customer(client_id);
        assert_eq!(customer.total, 0.);
        assert_eq!(customer.held, 0.);
        assert!(!customer.locked);
        assert!(customer.records.is_empty());
        assert!(customer.disputed_transactions.is_empty());
    }

    #[test]
    fn test_tracker_printable_accounts() {
        let mut tracker = Tracker::new();
        let client_id = 1;
        let customer = tracker.get_or_create_customer(client_id);
        customer.total = 100.;
        customer.held = 50.;
        let accounts = tracker.printable_accounts();
        assert_eq!(accounts.len(), 1);
        let account = &accounts[0];
        assert_eq!(account.client, client_id);
        assert_eq!(account.available, 50.);
        assert_eq!(account.held, 50.);
        assert_eq!(account.total, 100.);
        assert!(!account.locked);
    }
}
