use crate::{
    account::{AccountInfo, SetAccountInfo},
    bank::BankAccount,
    transaction::{DisputeSate, TransactionLog},
    transactions::{Transaction, TransactionInfo},
};

/// A chargeback is the final state of a dispute and represents the client reversing a transaction.
/// Funds that were held have now been withdrawn. This means that the clients held funds and
/// total funds should decrease by the amount previously disputed. If a chargeback occurs the
/// client's account should be immediately frozen.
/// A chargeback looks like
/// type client tx amount
/// chargeback 1 1
/// # Non-fatal Error:
/// Like a dispute and a resolve a chargeback refers to the transaction by ID (tx) and does not
/// specify an amount. Like a resolve, if the tx specified doesn't exist, or the tx isn't under
/// dispute, you can ignore chargeback and assume this is an error on our partner's side.
#[derive(Debug)]
pub(super) struct ChargeBack {
    account: BankAccount,
    disputed_tx: Option<TransactionLog>,
}
impl ChargeBack {
    pub(crate) fn new(account: BankAccount, disputed_tx: Option<TransactionLog>) -> Self {
        Self {
            account,
            disputed_tx,
        }
    }
}
impl Transaction for ChargeBack {
    #[tracing::instrument(err)]
    fn execute(&mut self) -> anyhow::Result<()> {
        if let Some(dispute) = &self.disputed_tx {
            match self.account.find_dispute(dispute.transaction_id()) {
                DisputeSate::Disputed(amount) => {
                    assert!(
                        amount <= self.account.held_funds(),
                        "Amount held and disputes got out of sync - BUG"
                    );
                    self.account.remove_held_funds(dispute.transaction_id());
                    self.account
                        .complete_dispute(dispute.transaction_id(), DisputeSate::Chargeback);

                    // we're now frozen so we cannot issue any deposit/withdrawals?
                    self.account.set_locked(true);
                }
                DisputeSate::Undisputed => {
                    tracing::debug!(account=?self.account, disputed_tx=?dispute, "Transaction undisputed");
                }
                DisputeSate::Chargeback => {
                    tracing::debug!(account=?self.account, disputed_tx=?dispute, "Transaction has already been charged back");
                }
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

        let test_folder = std::path::Path::new("./test_data/chargeback/ok");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn repeated() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/chargeback/repeated");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn unknown() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/chargeback/unknown");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn invalid_cid_tx() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/chargeback/invalid_cid_tx");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn undisputed() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/chargeback/undisputed");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
