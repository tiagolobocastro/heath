use crate::{csv::transaction::TransactionLogCsv, transaction::TransactionLog};
use std::{fs::File, io::Seek, path::PathBuf};

#[derive(Debug)]
pub(crate) struct Ledger {
    csv_file: File,
}

impl Ledger {
    /// New `Self` from a given csv file
    pub(crate) fn from_path(path: PathBuf) -> anyhow::Result<Self> {
        let csv_file = File::open(path)?;
        Ok(Self { csv_file })
    }
    fn reader(&self) -> anyhow::Result<csv::Reader<File>> {
        let mut file = self.csv_file.try_clone()?;
        file.rewind()?;
        let reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(file);
        Ok(reader)
    }
    /// Print ledger transactions to stdout
    #[allow(dead_code)]
    pub(crate) fn print_transactions(&self) -> anyhow::Result<()> {
        let mut reader = self.reader()?;
        for record in reader.deserialize::<TransactionLogCsv>() {
            println!("{:?}", record?);
            let _ = record;
        }
        Ok(())
    }
    /// Get a Ledger iterator
    pub(crate) fn iter(&self) -> anyhow::Result<LedgerIter> {
        Ok(LedgerIter {
            reader: self.reader()?,
        })
    }
}

/// Ledger iterator
#[derive(Debug)]
pub(crate) struct LedgerIter {
    reader: csv::Reader<File>,
}

impl Iterator for LedgerIter {
    type Item = TransactionLog;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.deserialize::<TransactionLogCsv>().next() {
            None => None,
            Some(Ok(transaction)) => Some(transaction.into()),
            Some(Err(error)) => {
                let error = anyhow::anyhow!("Error in the csv file!!!: {}", error);
                panic!("{}", error);
            }
        }
    }
}
