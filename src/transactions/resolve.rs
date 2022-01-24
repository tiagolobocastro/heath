use crate::{
    account::{AccountInfo, SetAccountInfo},
    bank::BankAccount,
    transaction::{DisputeSate, TransactionLog},
    transactions::{Transaction, TransactionInfo},
};

/// A resolve represents a resolution to a dispute, releasing the associated held funds. Funds that
/// were previously disputed are no longer disputed. This means that the clients held funds should
/// decrease by the amount no longer disputed, their available funds should increase by the
/// amount no longer disputed, and their total funds should remain the same.
/// A resolve looks like
/// type client tx amount
/// resolve 1 1
/// # Non-fatal Error:
/// Like disputes, resolves do not specify an amount. Instead they refer to a transaction that was
/// under dispute by ID. If the tx specified doesn't exist, or the tx isn't under dispute, you can
/// ignore the resolve and assume this is an error on our partner's side.
#[derive(Debug)]
pub(super) struct Resolve {
    account: BankAccount,
    disputed_tx: Option<TransactionLog>,
}
impl Resolve {
    pub(crate) fn new(account: BankAccount, disputed_tx: Option<TransactionLog>) -> Self {
        Self {
            account,
            disputed_tx,
        }
    }
}
impl Transaction for Resolve {
    #[tracing::instrument(err)]
    fn execute(&mut self) -> anyhow::Result<()> {
        if let Some(dispute) = &self.disputed_tx {
            match self.account.find_dispute(dispute.transaction_id()) {
                DisputeSate::Disputed(amount) => {
                    assert!(
                        amount <= self.account.held_funds(),
                        "Amount held and disputes got out of sync - BUG"
                    );
                    let available = self.account.available_funds();
                    let new_available = available + amount;
                    self.account.remove_held_funds(dispute.transaction_id());
                    self.account.set_available_funds(new_available);
                    // I'm guessing that we allow resolved disputes to be re-disputed?
                    self.account
                        .complete_dispute(dispute.transaction_id(), DisputeSate::Undisputed);
                }
                DisputeSate::Undisputed => {}
                DisputeSate::Chargeback => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{bank::tests::test, init_tracing};

    #[test]
    fn ok() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/resolve/ok");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn repeated() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/resolve/repeated");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn unknown() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/resolve/unknown");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn invalid_cid_tx() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/resolve/invalid_cid_tx");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
