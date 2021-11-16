use std::collections::HashMap;

use crate::account_repository::AccountRepository;
use crate::{Account, Transaction};

pub struct EngineExtended {
    account_repository: AccountRepository
}

impl EngineExtended {
    pub fn new(account_repository: AccountRepository) -> Self {
        Self {
            account_repository
        }
    }

    pub fn analyze(&mut self, transactions: Vec<Transaction>) -> (Vec<Account>, Vec<String>) {
        let mut applied_transactions = HashMap::new();
        let mut disputed_transactions = HashMap::new();
        let mut errors = vec![];

        for transaction in transactions {
            let account = self.account_repository.get_or_create(transaction.client);

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

        (self.account_repository.all(), errors)
    }
}
