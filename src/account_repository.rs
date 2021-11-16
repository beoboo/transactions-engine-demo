use std::collections::HashMap;
use crate::Account;

// Simulates a remote/cached repository
pub struct AccountRepository {
    pub data: HashMap<u16, Account>
}

impl AccountRepository {
    pub fn new() -> Self {
        Self {
            data: HashMap::new()
        }
    }

    pub fn get_or_create(&mut self, client: u16) -> &mut Account {
        self.data.entry(client).or_insert_with(|| Account::empty(client))
    }

    pub fn all(&self) -> Vec<Account> {
        self.data.clone().into_iter().map(|(_, account)| account).collect()
    }
}