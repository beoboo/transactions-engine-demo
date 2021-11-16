use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename(deserialize = "type"))]
    pub transaction_type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Decimal,
}

#[cfg(test)]
impl Transaction {
    pub fn new(transaction_type: String, client: u16, tx: u32, amount: Decimal) -> Self {
        Self { transaction_type, client, tx, amount}
    }
}
