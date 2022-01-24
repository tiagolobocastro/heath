use crate::{
    account::{AccountInfo, SetAccountInfo},
    bank::BankAccount,
    transactions::{Transaction, TransactionError},
};

/// A deposit is a credit to the client's asset account, meaning it should increase the available
/// and total funds of the client account
/// A deposit looks like
/// type client tx amount
/// deposit 1 1 1.0
/// Withdrawal
/// A withdraw is a debit to the client's asset account, meaning it should decrease the available
/// and total funds of the client account
#[derive(Debug)]
pub(super) struct Deposit {
    account: BankAccount,
    amount: rust_decimal::Decimal,
}

impl Deposit {
    pub(crate) fn new(account: BankAccount, amount: rust_decimal::Decimal) -> Self {
        Self { account, amount }
    }
}
impl Transaction for Deposit {
    #[tracing::instrument(err)]
    fn execute(&mut self) -> anyhow::Result<()> {
        if !self.account.locked() {
            let new_available = self.account.available_funds() + self.amount;
            self.account.set_available_funds(new_available);
        } else {
            let error = TransactionError::AccountFrozen {
                account: self.account.client_id(),
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
    fn ok() -> anyhow::Result<()> {
        init_tracing().ok();

        let test_folder = std::path::Path::new("./test_data/deposit/ok");
        let (expected, actual) = test(test_folder)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
