use crate::account_repository::AccountRepository;
use crate::{Account, Transaction};
use crate::cached_amounts::CachedAmounts;
use crate::engine::Engine;

// This extended version shows how we could cache data for accounts or for transactions
pub struct EngineExtended {
    account_repository: AccountRepository,
    applied_transactions: CachedAmounts,
    disputed_transactions: CachedAmounts,
}

impl EngineExtended {
    pub fn new(account_repository: AccountRepository, applied_transactions: CachedAmounts, disputed_transactions: CachedAmounts) -> Self {
        Self {
            account_repository,
            applied_transactions,
            disputed_transactions
        }
    }
}

impl Engine for EngineExtended {
    fn analyze(&mut self, transactions: Vec<Transaction>) -> (Vec<Account>, Vec<String>) {
        let mut errors = vec![];

        for transaction in transactions {
            let account = self.account_repository.get_or_create(transaction.client);

            match transaction.transaction_type.as_str() {
                "deposit" => {
                    account.deposit(transaction.amount).unwrap();
                    self.applied_transactions.add(transaction.tx, transaction.amount);
                }
                "withdrawal" => {
                    match account.withdraw(transaction.amount) {
                        Ok(_) => self.applied_transactions.add(transaction.tx, transaction.amount),
                        Err(err) => {
                            errors.push(format!("Error when handling transaction \"{}\": {}", transaction.tx, err));
                            continue;
                        }
                    };
                }
                "dispute" => {
                    let disputable = match self.applied_transactions.get(transaction.tx) {
                        Some(disputable) => disputable,
                        None => {
                            errors.push(format!("Could not find applied transaction \"{}\" to dispute", transaction.tx));
                            continue;
                        }
                    };

                    match account.dispute(disputable) {
                        Ok(_) => self.disputed_transactions.add(transaction.tx, disputable),
                        Err(err) => {
                            errors.push(format!("Could not dispute transaction \"{}\": {}", transaction.tx, err));
                            continue;
                        }
                    };
                }
                "resolve" => {
                    let resolvable = match self.disputed_transactions.get(transaction.tx) {
                        Some(amount) => amount,
                        None => {
                            errors.push(format!("Could not find disputed transaction \"{}\" to resolve", transaction.tx));
                            continue;
                        }
                    };

                    match account.resolve(resolvable) {
                        Ok(_) => self.disputed_transactions.remove(transaction.tx),
                        Err(err) => {
                            errors.push(format!("Could not resolve disputed transaction \"{}\": {}", transaction.tx, err));
                            continue;
                        }
                    };
                }
                "chargeback" => {
                    let back_chargeable = match self.disputed_transactions.get(transaction.tx) {
                        Some(amount) => amount.clone(),
                        None => {
                            errors.push(format!("Could not find disputed transaction \"{}\" to resolve", transaction.tx));
                            continue;
                        }
                    };

                    match account.chargeback(back_chargeable) {
                        Ok(_) => self.disputed_transactions.remove(transaction.tx),
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
