use crate::{
    account::AccountId,
    client::ClientId,
    csv::transaction::{TransactionId, TransactionType},
    transaction::TransactionLog,
    transactions::{
        chargeback::ChargeBack, deposit::Deposit, dispute::Dispute, resolve::Resolve,
        withdrawal::Withdrawal,
    },
    Bank,
};

mod chargeback;
mod deposit;
mod dispute;
mod resolve;
mod withdrawal;

/// A transaction, that can be executed
pub(crate) trait Transaction {
    fn execute(&mut self) -> anyhow::Result<()>;
}

/// Information about a transaction
pub(crate) trait TransactionInfo {
    fn transaction_type(&self) -> TransactionType;
    fn client_id(&self) -> ClientId;
    fn transaction_id(&self) -> TransactionId;
    fn amount(&self) -> Option<rust_decimal::Decimal>;
}

/// A bank transaction helper that implements `Transaction`
pub(crate) struct BankTransaction<'a> {
    bank: &'a mut Bank,
    chronological_index: usize,
    transaction_log: &'a TransactionLog,
}

impl<'a> BankTransaction<'a> {
    /// Return a new `Self`
    pub(crate) fn new(
        bank: &'a mut Bank,
        chronological_index: usize,
        transaction_log: &'a TransactionLog,
    ) -> Self {
        Self {
            bank,
            chronological_index,
            transaction_log,
        }
    }
}

/// A transaction error - was intended to return an error but it turns out we have to ignore
/// a certain number of "errors" so ended up doing away with it
#[derive(thiserror::Error, Debug)]
pub(crate) enum TransactionError {
    #[error("Insufficient Funds (required {required:?}, available {available:?})")]
    InsufficientFunds {
        required: rust_decimal::Decimal,
        available: rust_decimal::Decimal,
    },
    #[error("Account({account:?}) is frozen")]
    AccountFrozen { account: AccountId },
}

impl<'a> Transaction for BankTransaction<'a> {
    fn execute(&mut self) -> anyhow::Result<()> {
        let account = self.bank.account(self.transaction_log.client_id());
        match self.transaction_log {
            TransactionLog::Deposit { amount, .. } => Deposit::new(account, *amount).execute(),
            TransactionLog::Withdrawal { amount, .. } => {
                Withdrawal::new(account, *amount).execute()
            }

            TransactionLog::Dispute { .. } => {
                let dispute = self.bank.transaction(
                    self.chronological_index,
                    self.transaction_log.client_id(),
                    self.transaction_log.transaction_id(),
                )?;
                Dispute::new(account, dispute).execute()
            }
            TransactionLog::Resolve { .. } => {
                let dispute = self.bank.transaction(
                    self.chronological_index,
                    self.transaction_log.client_id(),
                    self.transaction_log.transaction_id(),
                )?;
                Resolve::new(account, dispute).execute()
            }
            TransactionLog::Chargeback { .. } => {
                let dispute = self.bank.transaction(
                    self.chronological_index,
                    self.transaction_log.client_id(),
                    self.transaction_log.transaction_id(),
                )?;
                ChargeBack::new(account, dispute).execute()
            }
        }
    }
}
