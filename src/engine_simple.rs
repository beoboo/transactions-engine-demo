use std::collections::HashMap;

use crate::{Account, Transaction};
use crate::engine::Engine;

pub struct EngineSimple {}

impl EngineSimple {
    pub fn new() -> Self {
        Self {}
    }
}

impl Engine for EngineSimple {
    fn analyze(&mut self, transactions: Vec<Transaction>) -> (Vec<Account>, Vec<String>) {
        let mut accounts : HashMap<u16, Account> = HashMap::new();
        let mut applied_transactions = HashMap::new();
        let mut disputed_transactions = HashMap::new();
        let mut errors = vec![];

        for transaction in transactions {
            let account = accounts.entry(transaction.client).or_insert_with(|| Account::empty(transaction.client));

            match transaction.transaction_type.as_str() {
                "deposit" => {
                    account.deposit(transaction.amount).unwrap();
                    applied_transactions.insert(transaction.tx, transaction.amount);
                }
                "withdrawal" => {
                    match account.withdraw(transaction.amount) {
                        Ok(_) => applied_transactions.insert(transaction.tx, transaction.amount),
                        Err(err) => {
                            errors.push(format!("Error when handling transaction \"{}\": {}", transaction.tx, err));
                            continue;
                        }
                    };
                }
                "dispute" => {
                    let disputable = match applied_transactions.get(&transaction.tx) {
                        Some(disputable) => disputable.clone(),
                        None => {
                            errors.push(format!("Could not find applied transaction \"{}\" to dispute", transaction.tx));
                            continue;
                        }
                    };

                    match account.dispute(disputable) {
                        Ok(_) => disputed_transactions.insert(transaction.tx, disputable),
                        Err(err) => {
                            errors.push(format!("Could not dispute transaction \"{}\": {}", transaction.tx, err));
                            continue;
                        }
                    };
                }
                "resolve" => {
                    let resolvable = match disputed_transactions.get(&transaction.tx) {
                        Some(amount) => amount.clone(),
                        None => {
                            errors.push(format!("Could not find disputed transaction \"{}\" to resolve", transaction.tx));
                            continue;
                        }
                    };

                    match account.resolve(resolvable) {
                        Ok(_) => disputed_transactions.remove(&transaction.tx),
                        Err(err) => {
                            errors.push(format!("Could not resolve disputed transaction \"{}\": {}", transaction.tx, err));
                            continue;
                        }
                    };
                }
                "chargeback" => {
                    let back_chargeable = match disputed_transactions.get(&transaction.tx) {
                        Some(amount) => amount.clone(),
                        None => {
                            errors.push(format!("Could not find disputed transaction \"{}\" to resolve", transaction.tx));
                            continue;
                        }
                    };

                    match account.chargeback(back_chargeable) {
                        Ok(_) => disputed_transactions.remove(&transaction.tx),
                        Err(err) => {
                            errors.push(format!("Could not charge back disputed transaction \"{}\": {}", transaction.tx, err));
                            continue;
                        }
                    };
                }
                t => panic!("Unhandled transaction type: {}", t),
            };
        }

        (accounts.into_iter().map(|(_id, account)| account).collect(), errors)
    }
}

#[cfg(test)]
mod tests {
    use hamcrest::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use crate::{Account, Transaction};

    use super::*;

    const CLIENT_ID: u16 = 123;

    #[test]
    fn test_no_transactions() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![]);
        assert_that!(accounts, is(equal_to(vec![])));
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_deposit() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![
            Transaction::new("deposit".into(), CLIENT_ID, 2, dec!(3.1234)),
        ]);

        assert_that!(accounts, is(equal_to(vec![
            Account::new(CLIENT_ID, dec!(3.1234), dec!(0.0), false),
        ])));
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_withdrawal() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![
            Transaction::new("deposit".into(), CLIENT_ID, 2, dec!(3.1234)),
            Transaction::new("withdrawal".into(), CLIENT_ID, 2, dec!(3.1234)),
        ]);

        assert_that!(accounts, is(equal_to(vec![
            Account::new(CLIENT_ID, dec!(0.0), dec!(0.0), false),
        ])));
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_withdrawal_from_insufficient_funds() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![
            Transaction::new("withdrawal".into(), CLIENT_ID, 2, dec!(3.1234))
        ]);

        assert_that!(accounts, is(equal_to(vec![Account::new(CLIENT_ID, dec!(0.0), dec!(0.0), false)])));
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_dispute() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![
            Transaction::new("deposit".into(), CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("dispute".into(), CLIENT_ID, 1, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(0.0), dec!(100.0), dec!(100.0), false);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_ignore_dispute_for_unknown_transaction() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![
            Transaction::new("deposit".into(), CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("dispute".into(), CLIENT_ID, 999, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 1);

    }

    #[test]
    fn test_resolve() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![
            Transaction::new("deposit".into(), CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("dispute".into(), CLIENT_ID, 1, dec!(0.0)),
            Transaction::new("resolve".into(), CLIENT_ID, 1, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_ignore_resolve_for_unknown_transaction() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![
            Transaction::new("deposit".into(), CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("resolve".into(), CLIENT_ID, 999, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_ignore_undisputed_resolve() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![
            Transaction::new("deposit".into(), CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("resolve".into(), CLIENT_ID, 1, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_chargeback() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![
            Transaction::new("deposit".into(), CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("dispute".into(), CLIENT_ID, 1, dec!(0.0)),
            Transaction::new("chargeback".into(), CLIENT_ID, 1, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(0.0), dec!(0.0), dec!(0.0), true);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_ignore_chargeback_for_unknown_transaction() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![
            Transaction::new("deposit".into(), CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("chargeback".into(), CLIENT_ID, 999, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_ignore_undisputed_chargeback() {
        let mut engine = EngineSimple::new();

        let (accounts, errors) = engine.analyze(vec![
            Transaction::new("deposit".into(), CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("chargeback".into(), CLIENT_ID, 1, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 1);
    }

    fn assert_account(account: &Account, available: Decimal, held: Decimal, total: Decimal, locked: bool) {
        assert_that!(account.available, is(equal_to(available)));
        assert_that!(account.held, is(equal_to(held)));
        assert_that!(account.total, is(equal_to(total)));
        assert_that!(account.locked, is(locked));
    }
}