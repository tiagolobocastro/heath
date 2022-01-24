mod account;
mod bank;
mod client;
mod csv;
mod ledger;
mod transaction;
mod transactions;

use crate::{bank::Bank, ledger::Ledger};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(structopt::StructOpt, Debug)]
struct CliArgs {
    /// Transactions file in a csv format.
    #[structopt(name = "transactions")]
    transactions: PathBuf,
}

fn init_tracing() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::from_default_env();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .pretty()
        .try_init()
        .map_err(|_| anyhow::anyhow!("Failed to init tracing (already inited?)"))?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::from_args();
    init_tracing()?;

    let ledger = Ledger::from_path(args.transactions)?;
    // ledger.print_transactions()?;

    let mut bank = Bank::new(ledger);

    // todo: this is probably not great for large datasets with around 2MB of account data
    println!("{}", bank.ordered_accounts_balance_buffer()?);

    Ok(())
}
