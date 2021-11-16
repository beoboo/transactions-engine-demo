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

    pub fn analyze_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
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
                        return Err(format!("Error when handling transaction \"{}\": {}", transaction.tx, err));
                    }
                };
            }
            "dispute" => {
                let disputable = match self.applied_transactions.get(transaction.tx) {
                    Some(disputable) => disputable,
                    None => {
                        return Err(format!("Could not find applied transaction \"{}\" to dispute", transaction.tx));
                    }
                };

                match account.dispute(disputable) {
                    Ok(_) => self.disputed_transactions.add(transaction.tx, disputable),
                    Err(err) => {
                        return Err(format!("Could not dispute transaction \"{}\": {}", transaction.tx, err));
                    }
                };
            }
            "resolve" => {
                let resolvable = match self.disputed_transactions.get(transaction.tx) {
                    Some(amount) => amount,
                    None => {
                        return Err(format!("Could not find disputed transaction \"{}\" to resolve", transaction.tx));
                    }
                };

                match account.resolve(resolvable) {
                    Ok(_) => self.disputed_transactions.remove(transaction.tx),
                    Err(err) => {
                        return Err(format!("Could not resolve disputed transaction \"{}\": {}", transaction.tx, err));
                    }
                };
            }
            "chargeback" => {
                let back_chargeable = match self.disputed_transactions.get(transaction.tx) {
                    Some(amount) => amount,
                    None => {
                        return Err(format!("Could not find disputed transaction \"{}\" to charge", transaction.tx));
                    }
                };

                match account.chargeback(back_chargeable) {
                    Ok(_) => self.disputed_transactions.remove(transaction.tx),
                    Err(err) => {
                        return Err(format!("Could not charge back disputed transaction \"{}\": {}", transaction.tx, err));
                    }
                };
            }
            t => {
                return Err(format!("Unhandled transaction type: \"{}\"", t));
            },
        };

        Ok(())
    }
}

impl Engine for EngineExtended {
    fn analyze(&mut self, transactions: Vec<Transaction>) -> (Vec<Account>, Vec<String>) {
        let mut errors = vec![];

        for transaction in transactions {
            match self.analyze_transaction(transaction) {
                Ok(_) => {},
                Err(err) => errors.push(err),
            }
        }

        (self.account_repository.all(), errors)
    }
}
