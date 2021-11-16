use std::{env, io};
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;

use csv::{ReaderBuilder, Trim};

use crate::account::Account;
use crate::account_repository::AccountRepository;
use crate::engine::Engine;
use crate::engine_extended::EngineExtended;
use crate::engine_simple::EngineSimple;
use crate::transaction::Transaction;

mod engine_simple;
mod engine_extended;
mod account_repository;
mod account;
mod transaction;
mod engine;

fn run() -> Result<(), Box<dyn Error>> {
    let file_path = get_first_arg()?;
    let file = File::open(file_path)?;

    let mut reader = ReaderBuilder::new()
        .trim(Trim::All).from_reader(file);

    let mut transactions = vec![];

    for result in reader.deserialize() {
        let record: Transaction = result?;
        transactions.push(record);
    }

    #[cfg(feature="simple")]
    let mut engine = EngineSimple::new();

    #[cfg(feature="extended")]
    let mut engine = EngineExtended::new(AccountRepository::new());

    // Returning a pair here so that we can handle valid transactions and report errors for invalid ones
    let (accounts, errors) = engine.analyze(transactions);

    for error in errors {
        eprintln!("{}", error)
    }

    let mut writer = csv::Writer::from_writer(io::stdout());

    for account in accounts {
        writer.serialize(account)?;
    }

    Ok(())
}

/// Returns the first positional argument sent to this process. If there are no
/// positional arguments, then this returns an error.
fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}