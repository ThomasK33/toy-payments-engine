use std::collections::HashMap;

use anyhow::anyhow;

use crate::structs;

pub struct Ledger {
    customer_map: HashMap<u16, Customer>,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            customer_map: HashMap::new(),
        }
    }

    pub fn get_or_insert_customer(&mut self, client_id: u16) -> &mut Customer {
        self.customer_map.entry(client_id).or_default()
    }

    pub fn client_records(&self) -> Vec<structs::ClientRecord> {
        self.customer_map
            .iter()
            .map(|(&client, customer)| structs::ClientRecord {
                client,
                // This is mostly for clipping of anything past four points of the decimal point
                available: ((customer.total_balance - customer.held_balance) * 10000.).round()
                    / 10000.,
                held: (customer.held_balance * 10000.).round() / 10000.,
                total: (customer.total_balance * 10000.).round() / 10000.,
                locked: customer.is_locked,
            })
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct Customer {
    total_balance: f32,
    held_balance: f32,
    is_locked: bool,

    /// Records is a map of transactions.
    /// A positive amount indicates a deposit,
    /// while a 0 amount indicates a withdrawal.
    records: HashMap<u32, f32>,

    disputed_transactions: Vec<u32>,
}

impl Customer {
    pub fn deposit(&mut self, tx: u32, amount: f32) -> anyhow::Result<()> {
        self.validate_amount_and_tx_id(amount, tx)?;
        self.validate_account_not_locked()?;

        self.total_balance += amount;
        self.records.insert(tx, amount);

        Ok(())
    }

    pub fn withdraw(&mut self, tx: u32, amount: f32) -> anyhow::Result<()> {
        self.validate_amount_and_tx_id(amount, tx)?;
        self.validate_account_not_locked()?;
        self.validate_sufficient_funds(amount)?;

        self.total_balance -= amount;
        // Inserting a zero here, so that transaction checks
        // can still work. If this were to be a negative number
        // a user could dispute a deposit and withdrawal at the same time
        // and get to a positive balance potentially.
        self.records.insert(tx, 0.);

        Ok(())
    }

    pub fn dispute(&mut self, tx: u32) -> anyhow::Result<()> {
        self.validate_transaction_exists(tx)?;
        self.validate_transaction_not_disputed(tx)?;

        let amount = self.get_transaction_amount(tx)?;
        self.held_balance += amount;
        self.disputed_transactions.push(tx);

        Ok(())
    }

    pub fn resolve(&mut self, tx: u32) -> anyhow::Result<()> {
        self.validate_transaction_exists(tx)?;
        self.validate_transaction_disputed(tx)?;

        let amount = self.get_transaction_amount(tx)?;
        self.held_balance -= amount;
        self.remove_disputed_transaction(tx);

        Ok(())
    }

    pub fn chargeback(&mut self, tx: u32) -> anyhow::Result<()> {
        self.validate_transaction_exists(tx)?;
        self.validate_transaction_disputed(tx)?;

        let amount = self.get_transaction_amount(tx)?;
        self.held_balance -= amount;
        self.total_balance -= amount;
        self.is_locked = true;

        Ok(())
    }

    fn validate_amount_and_tx_id(&self, amount: f32, tx: u32) -> anyhow::Result<()> {
        if amount < 0. {
            return Err(anyhow!("amount has to be positive"));
        }
        if self.records.contains_key(&tx) {
            return Err(anyhow!(
                "Customer already has a transaction with this tx id"
            ));
        }
        Ok(())
    }

    fn validate_account_not_locked(&self) -> anyhow::Result<()> {
        if self.is_locked {
            return Err(anyhow!("This account is locked"));
        }
        Ok(())
    }

    fn validate_sufficient_funds(&self, amount: f32) -> anyhow::Result<()> {
        if amount > (self.total_balance - self.held_balance) {
            return Err(anyhow!("Insufficient funds"));
        }
        Ok(())
    }

    fn validate_transaction_exists(&self, tx: u32) -> anyhow::Result<()> {
        if !self.records.contains_key(&tx) {
            return Err(anyhow!(
                "Customer does not has a transaction with this tx id"
            ));
        }
        Ok(())
    }

    fn validate_transaction_not_disputed(&self, tx: u32) -> anyhow::Result<()> {
        if self.disputed_transactions.contains(&tx) {
            return Err(anyhow!("Transaction is already disputed"));
        }
        Ok(())
    }

    fn validate_transaction_disputed(&self, tx: u32) -> anyhow::Result<()> {
        if !self.disputed_transactions.contains(&tx) {
            return Err(anyhow!("Transaction is not disputed"));
        }
        Ok(())
    }

    fn get_transaction_amount(&self, tx: u32) -> anyhow::Result<f32> {
        match self.records.get(&tx) {
            Some(amount) => Ok(*amount),
            None => Err(anyhow!("No transaction record found for the given id")),
        }
    }

    fn remove_disputed_transaction(&mut self, tx: u32) {
        if let Some(index) = self.disputed_transactions.iter().position(|a| a == &tx) {
            self.disputed_transactions.swap_remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;

        assert_eq!(customer.total_balance, 2.);

        Ok(())
    }

    #[test]
    fn test_deposit_2() {
        let mut customer = Customer {
            is_locked: true,
            ..Default::default()
        };
        let is_err = customer.deposit(1, 2.).is_err();

        assert!(is_err);
    }

    #[test]
    fn test_withdrawal() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;
        assert_eq!(customer.total_balance, 2.);

        customer.withdraw(2, 1.)?;
        assert_eq!(customer.total_balance, 1.);

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
        assert_eq!(customer.total_balance, 2.);

        customer.is_locked = true;

        let is_err = customer.withdraw(2, 1.).is_err();
        assert!(is_err);

        Ok(())
    }

    #[test]
    fn test_withdrawal_4() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;
        assert_eq!(customer.total_balance, 2.);

        customer.withdraw(2, 1.)?;
        let is_err = customer.withdraw(2, 1.).is_err();
        assert!(is_err);

        Ok(())
    }

    #[test]
    fn test_dispute() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;
        assert_eq!(customer.total_balance, 2.);

        customer.dispute(1)?;
        assert_eq!(customer.total_balance, 2.);
        assert_eq!(customer.held_balance, 2.);

        Ok(())
    }

    #[test]
    fn test_dispute_withdrawal() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;
        assert_eq!(customer.total_balance, 2.);
        customer.deposit(2, 1.)?;
        assert_eq!(customer.total_balance, 3.);

        customer.dispute(1)?;
        assert_eq!(customer.total_balance, 3.);
        assert_eq!(customer.held_balance, 2.);

        customer.withdraw(3, 1.)?;
        assert_eq!(customer.total_balance, 2.);
        assert_eq!(customer.held_balance, 2.);

        Ok(())
    }

    #[test]
    fn test_dispute_fail_withdrawal() -> anyhow::Result<()> {
        let mut customer = Customer::default();
        customer.deposit(1, 2.)?;
        assert_eq!(customer.total_balance, 2.);

        customer.dispute(1)?;
        assert_eq!(customer.total_balance, 2.);
        assert_eq!(customer.held_balance, 2.);

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
        assert_eq!(customer.total_balance, 5.);

        customer.dispute(1)?;
        assert_eq!(customer.total_balance, 5.);
        assert_eq!(customer.held_balance, 2.);

        customer.resolve(1)?;
        assert_eq!(customer.total_balance, 5.);
        assert_eq!(customer.held_balance, 0.);
        assert_eq!(customer.disputed_transactions.len(), 0);
        assert!(!customer.is_locked);

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
        assert_eq!(customer.total_balance, 5.);

        customer.dispute(1)?;
        assert_eq!(customer.total_balance, 5.);
        assert_eq!(customer.held_balance, 2.);

        customer.chargeback(1)?;
        assert_eq!(customer.total_balance, 3.);
        assert_eq!(customer.held_balance, 0.);
        assert_eq!(customer.disputed_transactions.len(), 1);
        assert!(customer.is_locked);

        Ok(())
    }

    #[test]
    fn test_chargeback_without_dispute() -> anyhow::Result<()> {
        let mut customer = Customer::default();

        customer.deposit(1, 2.)?;
        customer.deposit(2, 3.)?;
        assert_eq!(customer.total_balance, 5.);

        let is_err = customer.chargeback(1).is_err();
        assert!(is_err);
        assert_eq!(customer.total_balance, 5.);
        assert_eq!(customer.held_balance, 0.);
        assert_eq!(customer.disputed_transactions.len(), 0);
        assert!(!customer.is_locked);

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
        let tracker = Ledger::new();
        assert!(tracker.customer_map.is_empty());
    }

    #[test]
    fn test_tracker_get_or_create_customer() {
        let mut tracker = Ledger::new();
        let client_id = 1;
        let customer = tracker.get_or_insert_customer(client_id);
        assert_eq!(customer.total_balance, 0.);
        assert_eq!(customer.held_balance, 0.);
        assert!(!customer.is_locked);
        assert!(customer.records.is_empty());
        assert!(customer.disputed_transactions.is_empty());
    }

    #[test]
    fn test_tracker_printable_accounts() {
        let mut tracker = Ledger::new();
        let client_id = 1;
        let customer = tracker.get_or_insert_customer(client_id);
        customer.total_balance = 100.;
        customer.held_balance = 50.;
        let accounts = tracker.client_records();
        assert_eq!(accounts.len(), 1);
        let account = &accounts[0];
        assert_eq!(account.client, client_id);
        assert_eq!(account.available, 50.);
        assert_eq!(account.held, 50.);
        assert_eq!(account.total, 100.);
        assert!(!account.locked);
    }
}
