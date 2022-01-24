use crate::{
    account::{AccountInfo, SetAccountInfo},
    bank::BankAccount,
    transaction::{DisputeSate, TransactionLog},
    transactions::{Transaction, TransactionInfo},
};

/// A dispute represents a client's claim that a transaction was erroneous and should be reversed.
/// The transaction shouldn't be reversed yet but the associated funds should be held. This means
/// that the clients available funds should decrease by the amount disputed, their held funds should
/// increase by the amount disputed, while their total funds should remain the same.
/// A dispute looks like
/// type client tx amount
/// dispute 1 1
/// # Non-Fatal Error:
/// Notice that a dispute does not state the amount disputed. Instead a dispute references the
/// transaction that is disputed by ID. If the tx specified by the dispute doesn't exist you can
/// ignore it and assume this is an error on our partners side
#[derive(Debug)]
pub(super) struct Dispute {
    account: BankAccount,
    disputed_tx: Option<TransactionLog>,
}
impl Dispute {
    pub(crate) fn new(account: BankAccount, disputed_tx: Option<TransactionLog>) -> Self {
        Self {
            account,
            disputed_tx,
        }
    }
}
impl Transaction for Dispute {
    #[tracing::instrument(err)]
    fn execute(&mut self) -> anyhow::Result<()> {
        // disputes for locked accounts are currently allowed
        match &self.disputed_tx {
            None => {
                tracing::debug!(account=?self.account, "Disputed Transaction not found.");
                Ok(())
            }
            Some(disputed_tx) => {
                // Check that we don't dispute the same account twice for the same transaction
                let disputed_id = disputed_tx.transaction_id();
                match self.account.find_dispute(disputed_id) {
                    DisputeSate::Undisputed => {
                        if let Some(amount) = disputed_tx.amount() {
                            let available = self.account.available_funds();
                            if available >= amount {
                                let new_available = available - amount;
                                self.account.set_available_funds(new_available);
                                self.account
                                    .add_held_funds(amount, disputed_tx.transaction_id());
                            } else {
                                // I did not find the correct procedure in the document so I'm
                                // assuming that here we take the
                                // same approach as a withdrawal? Or would we
                                // allow the account funds to go negative?
                                tracing::debug!(account=?self.account, disputed_tx=?disputed_tx, "Disputed account does not have the funds!");
                            }
                        }
                    }
                    DisputeSate::Disputed(_) => {
                        tracing::debug!(account=?self.account, disputed_tx=?disputed_tx, "Transaction is already disputed");
                    }
                    DisputeSate::Chargeback => {
                        tracing::debug!(account=?self.account, disputed_tx=?disputed_tx, "Transaction has already been charged back");
                    }
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{bank::tests::test, init_tracing};

    #[test]
    fn ok() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/dispute/ok");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn repeated_unresolved() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/dispute/repeated_unresolved");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn repeated_resolved() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/dispute/repeated_resolved");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn repeated_charged() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/dispute/repeated_charged");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn unknown() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/dispute/unknown");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn invalid_cid_tx() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/dispute/invalid_cid_tx");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
