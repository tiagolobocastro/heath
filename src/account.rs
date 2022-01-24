use crate::{
    client::ClientId,
    csv::{account::AccountLog, transaction::TransactionId},
    transaction::DisputeSate,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub(crate) struct Account {
    /// Client identifier.
    client_id: ClientId,
    /// The total funds that are available for trading, staking, withdrawal, etc.
    /// This should be equal to the total - held amounts.
    available_funds: rust_decimal::Decimal,
    /// The total funds that are held for dispute.
    /// This should be equal to total - available amounts.
    held_funds: HashMap<TransactionId, rust_decimal::Decimal>,
    completed_disputes: HashMap<TransactionId, DisputeSate>,
    held_funds_cache: rust_decimal::Decimal,
    /// Whether the account is locked. An account is locked if a charge back occur.
    locked: bool,
}

// Assumed from the provided doc that there's only one account per client
pub(crate) type AccountId = crate::client::ClientId;

impl Account {
    pub(crate) fn new(account_id: AccountId) -> Self {
        Self {
            client_id: account_id,
            available_funds: rust_decimal::Decimal::new(0, 0),
            held_funds: Default::default(),
            completed_disputes: Default::default(),
            held_funds_cache: rust_decimal::Decimal::new(0, 0),
            locked: false,
        }
    }
    #[allow(dead_code)]
    pub(crate) fn log_info(&self) {
        tracing::info!(client=%self.client_id(), available=?self.available_funds(), held=?self.held_funds(), total=?self.total_funds(), locked=self.locked());
    }
    pub(crate) fn to_csv(&self) -> AccountLog {
        AccountLog::from(self)
    }
}

impl From<&Account> for AccountLog {
    fn from(acc: &Account) -> Self {
        AccountLog::new(
            acc.client_id,
            acc.available_funds().normalize(),
            acc.held_funds_cache.round_dp(4).normalize(),
            acc.total_funds().normalize(),
            acc.locked,
        )
    }
}

pub(crate) trait AccountInfo {
    fn client_id(&self) -> ClientId;
    fn available_funds(&self) -> rust_decimal::Decimal;
    fn held_funds(&self) -> rust_decimal::Decimal;
    fn total_funds(&self) -> rust_decimal::Decimal;
    fn locked(&self) -> bool;
    fn find_dispute(&self, transaction: TransactionId) -> DisputeSate {
        let _ = transaction;
        DisputeSate::Undisputed
    }
}

impl AccountInfo for Account {
    fn client_id(&self) -> ClientId {
        self.client_id
    }
    fn available_funds(&self) -> rust_decimal::Decimal {
        self.available_funds.round_dp(4)
    }
    fn held_funds(&self) -> rust_decimal::Decimal {
        self.held_funds_cache.round_dp(4)
    }
    fn total_funds(&self) -> rust_decimal::Decimal {
        self.held_funds() + self.available_funds()
    }
    fn locked(&self) -> bool {
        self.locked
    }
    fn find_dispute(&self, transaction: TransactionId) -> DisputeSate {
        if let Some(amount) = self.held_funds.get(&transaction) {
            DisputeSate::Disputed(*amount)
        } else {
            self.completed_disputes
                .get(&transaction)
                .cloned()
                .unwrap_or(DisputeSate::Undisputed)
        }
    }
}

pub(crate) trait SetAccountInfo {
    fn set_available_funds(&mut self, amount: rust_decimal::Decimal);
    fn add_held_funds(&mut self, amount: rust_decimal::Decimal, disputer_id: TransactionId);
    fn remove_held_funds(&mut self, disputer_id: TransactionId);
    fn set_locked(&mut self, locked: bool);
    fn complete_dispute(&mut self, disputer_id: TransactionId, state: DisputeSate);
}

impl SetAccountInfo for Account {
    fn set_available_funds(&mut self, amount: rust_decimal::Decimal) {
        self.available_funds = amount.round_dp(4);
    }
    fn add_held_funds(&mut self, amount: rust_decimal::Decimal, disputer_id: TransactionId) {
        let amount = amount.round_dp(4);
        self.held_funds.insert(disputer_id, amount);
        self.held_funds_cache += amount;
    }
    fn remove_held_funds(&mut self, disputer_id: TransactionId) {
        if let Some(d) = self.held_funds.remove(&disputer_id) {
            self.held_funds_cache -= d.round_dp(4);
        };
    }
    fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }
    fn complete_dispute(&mut self, disputer_id: TransactionId, state: DisputeSate) {
        match state {
            DisputeSate::Undisputed => {}
            DisputeSate::Disputed(_) => {}
            DisputeSate::Chargeback => {
                self.completed_disputes.insert(disputer_id, state);
            }
        }
    }
}
