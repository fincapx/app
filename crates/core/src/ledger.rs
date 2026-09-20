use std::collections::HashMap;

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

#[cfg(test)]
mod test {

    use super::*;

    fn cop() -> CurrencyCode {
        CurrencyCode::new("COP").unwrap()
    }

    fn usd() -> CurrencyCode {
        CurrencyCode::new("USD").unwrap()
    }

    /// Un almuerzo: sale de efectivo, entra en gastos.
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

    /// Cambio de divisa con cuentas de intercambio: cuatro apuntes, dos
    /// monedas, y cada una cuadra por su lado
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

    /// La trampa que justifica el mapa: el total general da cero, pero son
    /// monedas distintas y no se pueden sumar entre ellas.
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
