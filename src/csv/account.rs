use crate::{account::AccountInfo, client::ClientId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct AccountLog {
    /// Client identifier.
    #[serde(rename = "client")]
    client_id: ClientId,
    /// The total funds that are available for trading, staking, withdrawal, etc.
    /// This should be equal to the total - held amounts.
    #[serde(rename = "available")]
    available_funds: rust_decimal::Decimal,
    /// The total funds that are held for dispute.
    /// This should be equal to total - available amounts.
    #[serde(rename = "held")]
    held_funds: rust_decimal::Decimal,
    /// The total funds that are available or held. This should be equal to available + held.
    #[serde(rename = "total")]
    total_funds: rust_decimal::Decimal,
    /// Whether the account is locked. An account is locked if a charge back occur.
    #[serde(rename = "locked")]
    locked: bool,
}
impl AccountLog {
    pub(crate) fn new(
        client_id: ClientId,
        available_funds: rust_decimal::Decimal,
        held_funds: rust_decimal::Decimal,
        total_funds: rust_decimal::Decimal,
        locked: bool,
    ) -> Self {
        Self {
            client_id,
            available_funds,
            held_funds,
            total_funds,
            locked,
        }
    }
}

impl AccountInfo for AccountLog {
    fn client_id(&self) -> ClientId {
        self.client_id
    }
    fn available_funds(&self) -> rust_decimal::Decimal {
        self.available_funds
    }
    fn held_funds(&self) -> rust_decimal::Decimal {
        self.held_funds
    }
    fn total_funds(&self) -> rust_decimal::Decimal {
        self.total_funds
    }
    fn locked(&self) -> bool {
        self.locked
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::AccountLog;

    #[test]
    /// Basic CSV test, read some test input and write it back, it should be the same
    fn csv_sanity() -> anyhow::Result<()> {
        let test_input = "\
client,available,held,total,locked
1,1.5,0,1.5,false
2,2,0,2,false
";
        let mut test_reader = csv::Reader::from_reader(test_input.as_bytes());
        let accounts = test_reader
            .deserialize::<AccountLog>()
            .enumerate()
            .map(|(_, t)| t)
            .collect::<Vec<_>>();

        let mut w = csv::Writer::from_writer(vec![]);
        for account in accounts {
            w.serialize(account?)?;
        }
        let output = String::from_utf8(w.into_inner()?)?;
        assert_eq!(test_input, output);
        Ok(())
    }

    /// Same test as csv_sanity but with spaces
    #[test]
    fn csv_sanity_with_spaces() -> anyhow::Result<()> {
        let test_input = "\
client, available, held, total, locked
1, 1.5, 0, 1.5, false
2, 2, 0, 2, false
";
        let mut test_reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(test_input.as_bytes());
        let accounts = test_reader
            .deserialize::<AccountLog>()
            .enumerate()
            .map(|(_, t)| t)
            .collect::<Vec<_>>();

        let mut w = csv::Writer::from_writer(vec![]);
        for account in accounts {
            w.serialize(account.unwrap())?;
        }
        let output = String::from_utf8(w.into_inner()?)?;
        assert_eq!(test_input.replace(' ', ""), output);
        Ok(())
    }
}
