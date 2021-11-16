use std::collections::HashMap;
use rust_decimal::Decimal;

pub struct CachedAmounts {
    data: HashMap<u32, Decimal>,
}

impl CachedAmounts {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn add(&mut self, tx: u32, amount: Decimal)  {
        self.data.insert(tx, amount);
    }

    pub fn remove(&mut self, tx: u32)  {
        self.data.remove(&tx);
    }

    pub fn get(&mut self, tx: u32) -> Option<Decimal> {
        match self.data.get(&tx) {
            Some(amount) => Some(amount.clone()),
            None => None,
        }
    }
}