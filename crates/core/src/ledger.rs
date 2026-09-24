use std::collections::HashMap;
use uuid::Uuid;

use crate::money::{CurrencyCode, Money, MoneyError};

/// Comprueba que una lista de importes cuadre
///
/// La regla: para cada moneda, la suma de sus importes es exactamente cero.
/// Las monedas no se mezclan - cada una lleva su propia cuenta.
pub fn is_balanced(amounts: &[Money]) -> Result<bool, MoneyError> {
    // Un cubo por moneda: la moneda es la clave, el total acumulado el valor
    let mut totals: HashMap<CurrencyCode, Money> = HashMap::new();

    for amount in amounts {
        // Ya hay cubo para esta moneda? Si no, uno nuevo en cero
        let current = totals
            .get(&amount.currency())
            .copied()
            .unwrap_or(Money::zero(amount.currency()));

        // Suma este importe al cubo y guardatelo de vuelta
        totals.insert(amount.currency(), current.try_add(*amount)?);
    }

    // Cuadra si todos los cubos quedaron en cero
    Ok(totals.values().all(Money::is_zero))
}

/// Un apunte: el movimiento de un importe en una cuenta.
///
/// Es una linea de una transaccion. Por si solo no significa nada; cobra
/// sentido dentro del conjunto que tiene que cuadrar
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    id: Uuid,
    account_id: Uuid,
    amount: Money,
    memo: Option<String>,
}

impl Posting {
    /// Crea un apunte nuevo con identidad propia.
    #[must_use]
    pub fn new(account_id: Uuid, amount: Money, memo: Option<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            account_id,
            amount,
            memo,
        }
    }

    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn account_id(&self) -> Uuid {
        self.account_id
    }

    #[must_use]
    pub fn amount(&self) -> Money {
        self.amount
    }

    #[must_use]
    pub fn memo(&self) -> Option<&str> {
        self.memo.as_deref()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn cop() -> CurrencyCode {
        CurrencyCode::new("COP").unwrap()
    }

    fn usd() -> CurrencyCode {
        CurrencyCode::new("USD").unwrap()
    }

    // Un almuerzo: sale de efectivo, entra en gastos.
    #[test]
    fn simple_transaction_balances() {
        let postings = [Money::new(-50_000, cop()), Money::new(50_000, cop())];
        assert!(is_balanced(&postings).unwrap());
    }

    #[test]
    fn detects_amounts_that_do_not_add_up() {
        let postings = [Money::new(-50_000, cop()), Money::new(40_000, cop())];
        assert!(!is_balanced(&postings).unwrap());
    }

    // Cuatro apuntes, dos monedas, cada una cuadra por su lado.
    #[test]
    fn currency_exchange_balances_per_currency() {
        let postings = [
            Money::new(-10_000, usd()),
            Money::new(10_000, usd()),
            Money::new(-400_000, cop()),
            Money::new(400_000, cop()),
        ];
        assert!(is_balanced(&postings).unwrap());
    }

    // El total general da cero, pero son monedas distintas.
    #[test]
    fn does_not_mix_currencies() {
        let postings = [Money::new(-10_000, usd()), Money::new(10_000, cop())];
        assert!(!is_balanced(&postings).unwrap());
    }

    #[test]
    fn empty_list_balances() {
        assert!(is_balanced(&[]).unwrap());
    }
}
