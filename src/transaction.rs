use crate::{
    client::ClientId,
    csv::transaction::{TransactionId, TransactionLogCsv, TransactionType},
    transactions::TransactionInfo,
};
use serde::{Deserialize, Serialize};

impl TransactionInfo for TransactionLog {
    fn transaction_type(&self) -> TransactionType {
        match self {
            Self::Deposit { .. } => TransactionType::Deposit,
            Self::Withdrawal { .. } => TransactionType::Withdrawal,
            Self::Dispute { .. } => TransactionType::Dispute,
            Self::Resolve { .. } => TransactionType::Resolve,
            Self::Chargeback { .. } => TransactionType::Chargeback,
        }
    }
    fn client_id(&self) -> ClientId {
        match self {
            Self::Deposit { common, .. } => common.client_id,
            Self::Withdrawal { common, .. } => common.client_id,
            Self::Dispute { common } => common.client_id,
            Self::Resolve { common } => common.client_id,
            Self::Chargeback { common } => common.client_id,
        }
    }
    fn transaction_id(&self) -> TransactionId {
        match self {
            Self::Deposit { common, .. } => common.tx_id,
            Self::Withdrawal { common, .. } => common.tx_id,
            Self::Dispute { common } => common.tx_id,
            Self::Resolve { common } => common.tx_id,
            Self::Chargeback { common } => common.tx_id,
        }
    }
    fn amount(&self) -> Option<rust_decimal::Decimal> {
        match self {
            Self::Deposit { amount, .. } => Some(*amount),
            Self::Withdrawal { amount, .. } => Some(*amount),
            Self::Dispute { .. } => None,
            Self::Resolve { .. } => None,
            Self::Chargeback { .. } => None,
        }
    }
}

// https://github.com/BurntSushi/rust-csv/issues/211
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum TransactionLog {
    Deposit {
        #[serde(flatten)]
        common: TransactionLogCommon,
        /// Transaction amount with a precision of up to four places past the
        /// rust_decimal::Decimal.
        #[serde(rename = "amount")]
        amount: rust_decimal::Decimal,
    },
    Withdrawal {
        common: TransactionLogCommon,
        /// Transaction amount with a precision of up to four places past the
        /// rust_decimal::Decimal.
        #[serde(rename = "amount")]
        amount: rust_decimal::Decimal,
    },
    Dispute {
        #[serde(flatten)]
        common: TransactionLogCommon,
    },
    Resolve {
        #[serde(flatten)]
        common: TransactionLogCommon,
    },
    Chargeback {
        #[serde(flatten)]
        common: TransactionLogCommon,
    },
}

/// Dispute state of a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum DisputeSate {
    Undisputed,
    /// Currently being disputed.
    Disputed(rust_decimal::Decimal),
    /// Disputed and charged back.
    Chargeback,
}
impl Default for DisputeSate {
    fn default() -> Self {
        Self::Undisputed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransactionLogCommon {
    /// Client ID.
    #[serde(rename = "client")]
    client_id: ClientId,
    /// Transaction ID.
    #[serde(rename = "tx")]
    tx_id: TransactionId,
}
// impl TransactionLogCommon {
//     pub(crate) fn client_id(&self) -> ClientId {
//         self.client_id
//     }
//     pub(crate) fn transaction_id(&self) -> TransactionId {
//         self.tx_id
//     }
// }

impl From<TransactionLogCsv> for TransactionLog {
    fn from(tx: TransactionLogCsv) -> Self {
        let common = TransactionLogCommon {
            client_id: tx.client_id(),
            tx_id: tx.transaction_id(),
        };
        match tx.transaction_type() {
            TransactionType::Deposit => Self::Deposit {
                common,
                amount: tx.amount().expect("Deposit should contain the amount"),
            },
            TransactionType::Withdrawal => Self::Withdrawal {
                common,
                amount: tx.amount().expect("Withdrawal should contain the amount"),
            },
            TransactionType::Dispute => Self::Dispute { common },
            TransactionType::Resolve => Self::Resolve { common },
            TransactionType::Chargeback => Self::Chargeback { common },
        }
    }
}

impl TransactionLog {
    #[allow(dead_code)]
    pub(crate) fn log_info(&self) {
        tracing::info!(type_=?self.transaction_type(), client=self.client_id(), tx=%self.transaction_id(), amount=?self.amount());
    }
}
