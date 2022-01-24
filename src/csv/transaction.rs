use crate::{client::ClientId, transactions::TransactionInfo};
use serde::{Deserialize, Serialize};

/// Type identifier for a transaction
pub(crate) type TransactionId = u32;

/// The input will be a CSV file with the columns type, client, tx, and amount. You can assume the
/// type is a string, the client column is a valid u16 client ID, the tx is a valid u32 transaction
/// ID, and the amount is a rust_decimal::Decimal value with a precision of up to four places past
/// the rust_decimal::Decimal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransactionLogCsv {
    /// Transaction Type.
    #[serde(rename = "type")]
    type_: TransactionType,
    /// Client ID.
    #[serde(rename = "client")]
    client_id: ClientId,
    /// Transaction ID.
    #[serde(rename = "tx")]
    tx_id: TransactionId,
    /// Transaction amount with a precision of up to four places past the rust_decimal::Decimal.
    #[serde(rename = "amount")]
    amount: Option<rust_decimal::Decimal>,
}

impl TransactionInfo for TransactionLogCsv {
    fn transaction_type(&self) -> TransactionType {
        self.type_.clone()
    }
    fn client_id(&self) -> ClientId {
        self.client_id
    }
    fn transaction_id(&self) -> TransactionId {
        self.tx_id
    }
    fn amount(&self) -> Option<rust_decimal::Decimal> {
        self.amount
    }
}
impl TransactionLogCsv {
    #[allow(dead_code)]
    pub(crate) fn log_info(&self) {
        tracing::info!(type_=?self.transaction_type(), client=self.client_id(), tx=%self.transaction_id(), amount=?self.amount());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[cfg(test)]
mod tests {
    use super::TransactionLogCsv;
    #[test]
    /// Basic CSV test, read some test input and write it back, it should be the same
    fn csv_sanity() -> anyhow::Result<()> {
        let test_input = "\
type,client,tx,amount
deposit,1,1,1
deposit,2,2,2
deposit,1,3,2
withdrawal,1,4,1.5
withdrawal,2,5,3
";
        let mut test_reader = csv::Reader::from_reader(test_input.as_bytes());
        let transactions = test_reader
            .deserialize::<TransactionLogCsv>()
            .enumerate()
            .map(|(_, t)| t)
            .collect::<Vec<_>>();

        let mut w = csv::Writer::from_writer(vec![]);
        for transaction in transactions {
            w.serialize(transaction?)?;
        }
        let output = String::from_utf8(w.into_inner()?)?;
        assert_eq!(test_input, output);
        Ok(())
    }

    /// Same test as csv_sanity but with spaces
    #[test]
    fn csv_sanity_with_spaces() -> anyhow::Result<()> {
        let test_input = "\
type, client, tx, amount
deposit, 1,1, 1
deposit, 2,2, 2
deposit, 1,3, 2
withdrawal, 1,4, 1.5
withdrawal, 2,5, 3
";
        let mut test_reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(test_input.as_bytes());
        let transactions = test_reader
            .deserialize::<TransactionLogCsv>()
            .enumerate()
            .map(|(_, t)| t.ok())
            .flatten()
            .collect::<Vec<_>>();

        let mut w = csv::Writer::from_writer(vec![]);
        for transaction in &transactions {
            w.serialize(transaction)?;
        }
        let output = String::from_utf8(w.into_inner()?)?;
        assert_eq!(test_input.replace(' ', ""), output);

        Ok(())
    }
}
