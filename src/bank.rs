use crate::{
    account::{Account, AccountId, AccountInfo, SetAccountInfo},
    client::ClientId,
    csv::transaction::TransactionId,
    transaction::{DisputeSate, TransactionLog},
    transactions::{BankTransaction, Transaction, TransactionInfo},
    Ledger,
};
use itertools::Itertools;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// A bank Account
/// todo: The way things are this could probably use a Cell instead of a Mutex
pub(crate) type BankAccount = Arc<Mutex<Account>>;

/// A Bank
/// It has a ledger of transactions and bank accounts.
#[derive(Debug)]
pub(crate) struct Bank {
    accounts: HashMap<AccountId, BankAccount>,
    ledger: Ledger,
}

impl Bank {
    /// Return a new `Self` with the provided `Ledger`
    pub(crate) fn new(ledger: Ledger) -> Self {
        Self {
            accounts: Default::default(),
            ledger,
        }
    }
    /// Get the BankAccount for the given account_id
    /// If the account does not exist a new default account will be created
    pub(crate) fn account(&mut self, account_id: AccountId) -> BankAccount {
        self.accounts
            .entry(account_id)
            .or_insert_with(|| Arc::new(Mutex::new(Account::new(account_id))))
            .clone()
    }
    /// Try to get the TransactionLog for the given transaction_id
    /// Searches the ledger only up to the chronologically ordered index max_ledger_search
    pub(crate) fn transaction(
        &mut self,
        max_ledger_search: usize,
        account_id: AccountId,
        transaction_id: TransactionId,
    ) -> anyhow::Result<Option<TransactionLog>> {
        Ok(self
            .ledger
            .iter()?
            .take(max_ledger_search)
            .find(|transaction| {
                transaction.transaction_id() == transaction_id
                    && account_id == transaction.client_id()
            }))
    }

    /// Get the ordered accounts balance as a String
    pub(crate) fn ordered_accounts_balance_buffer(&mut self) -> anyhow::Result<String> {
        // Note: if we ever wanted to "commit" the ledger into the accounts we'd have to either
        // trim the ledger or make sure the iterator can not be reset
        let _ = std::mem::take(&mut self.accounts);

        self.ledger.iter()?.enumerate().for_each(|(index, f)| {
            // as things stand most "errors"/invalid ops are simply ignored, but they're ignored
            // in the specific transaction as it's the one that knows what it should ignore
            BankTransaction::new(self, index, &f).execute().unwrap();
        });
        let mut w = csv::Writer::from_writer(vec![]);
        for account in self
            .accounts
            .iter()
            .map(|a| a.1.lock().unwrap().to_csv())
            .sorted_by(|a, b| a.client_id().cmp(&b.client_id()))
        {
            w.serialize(account)?;
        }
        let _ = std::mem::take(&mut self.accounts);

        Ok(String::from_utf8(w.into_inner()?)?)
    }
}

impl SetAccountInfo for BankAccount {
    fn set_available_funds(&mut self, amount: rust_decimal::Decimal) {
        self.lock().unwrap().set_available_funds(amount.round_dp(4))
    }
    fn add_held_funds(&mut self, amount: rust_decimal::Decimal, disputer_id: TransactionId) {
        self.lock().unwrap().add_held_funds(amount, disputer_id)
    }
    fn remove_held_funds(&mut self, disputer_id: TransactionId) {
        self.lock().unwrap().remove_held_funds(disputer_id)
    }
    fn set_locked(&mut self, locked: bool) {
        self.lock().unwrap().set_locked(locked)
    }
    fn complete_dispute(&mut self, disputer_id: TransactionId, state: DisputeSate) {
        self.lock().unwrap().complete_dispute(disputer_id, state)
    }
}
// todo: use Deref with an OwnedMutexGuard target?
impl AccountInfo for BankAccount {
    fn client_id(&self) -> ClientId {
        self.lock().unwrap().client_id()
    }
    fn available_funds(&self) -> rust_decimal::Decimal {
        self.lock().unwrap().available_funds()
    }
    fn held_funds(&self) -> rust_decimal::Decimal {
        self.lock().unwrap().held_funds()
    }
    fn total_funds(&self) -> rust_decimal::Decimal {
        self.lock().unwrap().total_funds()
    }
    fn locked(&self) -> bool {
        self.lock().unwrap().locked()
    }
    fn find_dispute(&self, transaction: TransactionId) -> DisputeSate {
        self.lock().unwrap().find_dispute(transaction)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{Bank, Ledger};

    /// Get a Bank usable for testing
    pub(crate) fn bank(test_file: std::path::PathBuf) -> anyhow::Result<Bank> {
        let ledger = Ledger::from_path(test_file)?;
        let bank = Bank::new(ledger);
        Ok(bank)
    }
    /// Test that the folder's test input and output succeed:
    /// The input is read into the bank which then returns the actual output.
    /// Returns a tuple with the expected output and the actual output.
    pub(crate) fn test(tests_folder: &std::path::Path) -> anyhow::Result<(String, String)> {
        let expected_output = std::fs::read_to_string(tests_folder.join("output.csv"))?;
        let mut bank = bank(tests_folder.join("input.csv"))?;

        let output = bank.ordered_accounts_balance_buffer()?;
        Ok((expected_output.trim().into(), output.trim().into()))
    }
}
