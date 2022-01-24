use crate::{
    account::{AccountInfo, SetAccountInfo},
    bank::BankAccount,
    transactions::{Transaction, TransactionError},
};

/// A withdraw is a debit to the client's asset account, meaning it should decrease the available
/// and total funds of the client account
/// A withdrawal looks like
/// type client tx amount
/// withdrawal 2 2 1.0
/// # Non-Fatal Error
/// If a client does not have sufficient available funds the withdrawal should fail and the
/// total amount of funds should not change
#[derive(Debug)]
pub(super) struct Withdrawal {
    account: BankAccount,
    amount: rust_decimal::Decimal,
}
impl Withdrawal {
    pub(crate) fn new(account: BankAccount, amount: rust_decimal::Decimal) -> Self {
        Self { account, amount }
    }
}
impl Transaction for Withdrawal {
    #[tracing::instrument(err)]
    fn execute(&mut self) -> anyhow::Result<()> {
        if self.account.locked() {
            let error = TransactionError::AccountFrozen {
                account: self.account.client_id(),
            };
            tracing::debug!(error=%error, "non-fatal error occurred");
            return Ok(());
        }
        let available = self.account.available_funds();
        if available >= self.amount {
            let new_available = available - self.amount;
            self.account.set_available_funds(new_available);
        } else {
            let error = TransactionError::InsufficientFunds {
                required: self.amount,
                available,
            };
            tracing::debug!(error=%error, "non-fatal error occurred");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{bank::tests::test, init_tracing};

    #[test]
    fn no_funds() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/withdrawal/no_funds");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn ok() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/withdrawal/ok");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
