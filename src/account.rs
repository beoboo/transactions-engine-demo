use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Account {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    pub fn new(client: u16, available: Decimal, held: Decimal, locked: bool) -> Self {
        Self { client, available, held, total: available + held, locked}
    }

    pub fn empty(client: u16) -> Self {
        Self::new(client, Decimal::from(0), Decimal::from(0), false)
    }

    pub fn deposit(&mut self, amount: Decimal) -> Result<(), String>{
        self.available += amount;
        self.total += amount;

        Ok(())
    }

    pub fn withdraw(&mut self, amount: Decimal) -> Result<(), String>{
        if amount > self.available {
            return Err(format!("Insufficient available funds"))
        }
        self.available -= amount;
        self.total -= amount;

        Ok(())
    }

    pub fn dispute(&mut self, amount: Decimal) -> Result<(), String>{
        if amount > self.available {
            return Err(format!("Insufficient available funds"))
        }
        self.available -= amount;
        self.held += amount;

        Ok(())
    }

    pub fn resolve(&mut self, amount: Decimal) -> Result<(), String>{
        if amount > self.held {
            return Err(format!("Insufficient held funds"))
        }
        self.available += amount;
        self.held -= amount;

        Ok(())
    }

    pub fn chargeback(&mut self, amount: Decimal) -> Result<(), String>{
        if amount > self.held {
            return Err(format!("Insufficient held funds"))
        }
        self.held -= amount;
        self.total -= amount;
        self.locked = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use hamcrest::*;
    use rust_decimal_macros::dec;
    use super::*;

    #[test]
    fn test_deposit() {
        let mut account = Account::empty(123);
        account.deposit(dec!(100)).unwrap();

        assert_that!(account.available, is(equal_to(dec!(100))));
        assert_that!(account.total, is(equal_to(dec!(100))));
    }

    #[test]
    fn test_withdraw() {
        let mut account = Account::empty(123);
        account.deposit(dec!(100)).unwrap();
        account.withdraw(dec!(50)).unwrap();

        assert_that!(account.available, is(equal_to(dec!(50))));
        assert_that!(account.total, is(equal_to(dec!(50))));
    }

    #[test]
    fn test_withdraw_insufficient_available_funds() {
        let mut account = Account::empty(123);
        assert!(account.withdraw(dec!(50)).is_err());
    }

    #[test]
    fn test_dispute() {
        let mut account = Account::empty(123);
        account.deposit(dec!(100)).unwrap();
        account.dispute(dec!(50)).unwrap();

        assert_that!(account.available, is(equal_to(dec!(50))));
        assert_that!(account.held, is(equal_to(dec!(50))));
        assert_that!(account.total, is(equal_to(dec!(100))));
    }

    #[test]
    fn test_dispute_insufficient_available_funds() {
        let mut account = Account::empty(123);
        assert!(account.dispute(dec!(50)).is_err());
    }

    #[test]
    fn test_resolve() {
        let mut account = Account::empty(123);
        account.deposit(dec!(100)).unwrap();
        account.dispute(dec!(50)).unwrap();
        account.resolve(dec!(50)).unwrap();

        assert_that!(account.available, is(equal_to(dec!(100))));
        assert_that!(account.held, is(equal_to(dec!(0))));
        assert_that!(account.total, is(equal_to(dec!(100))));
    }

    #[test]
    fn test_resolve_insufficient_held_funds() {
        let mut account = Account::empty(123);
        assert!(account.resolve(dec!(50)).is_err());
    }

    #[test]
    fn test_chargeback() {
        let mut account = Account::empty(123);
        account.deposit(dec!(100)).unwrap();
        account.dispute(dec!(50)).unwrap();
        account.chargeback(dec!(50)).unwrap();

        assert_that!(account.available, is(equal_to(dec!(50))));
        assert_that!(account.held, is(equal_to(dec!(0))));
        assert_that!(account.total, is(equal_to(dec!(50))));
    }

    #[test]
    fn test_chargeback_insufficient_held_funds() {
        let mut account = Account::empty(123);
        assert!(account.chargeback(dec!(50)).is_err());
    }
}

