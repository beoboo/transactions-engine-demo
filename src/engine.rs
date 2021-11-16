use crate::{Account, Transaction};

pub trait Engine {
    fn analyze(&mut self, transactions: Vec<Transaction>) -> (Vec<Account>, Vec<String>);
}
