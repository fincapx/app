use std::fmt;

/// Código de moneda ISO 4217: tres letras ASCII, guardadas en mayúscula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CurrencyCode([u8; 3]);

impl CurrencyCode {
    /// Acepta tres letras ASCII en cualquier caja. Normaliza a mayúscula.
    pub fn new(code: &str) -> Result<Self, MoneyError> {
        // El texto se ve como una lista de bytes. "COP" --> [67, 79, 80]
        let bytes = code.as_bytes();

        // ¿exactamente tres bytes?
        if bytes.len() != 3 {
            return Err(MoneyError::InvalidCurrencyCode(code.to_string()));
        }

        // ¿son tres letras ASCII?
        for b in bytes {
            if !b.is_ascii_alphabetic() {
                return Err(MoneyError::InvalidCurrencyCode(code.to_string()));
            }
        }

        // Las tres a mayúscula, envueltas en Ok
        Ok(Self([
            bytes[0].to_ascii_uppercase(),
            bytes[1].to_ascii_uppercase(),
            bytes[2].to_ascii_uppercase(),
        ]))
    }

    /// Devuelve el código como slice de string.
    ///
    /// En la práctica nunca entra en pánico: 'new' es el único constructor
    /// y solo acepta ASCII, que siempre es UTF-8 válido.
    #[must_use]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }
}

/// Un importe en una sola moneda, guardado en unidades menores.
///
/// La escala (cuántos decimales tiene la moneda) vive en la tabla de
/// monedas, no aquí: toda la aritmética ocurre en unidades menores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    amount: i64,
    currency: CurrencyCode,
}

impl Money {
    #[must_use]
    pub fn new(amount: i64, currency: CurrencyCode) -> Self {
        Self { amount, currency }
    }

    #[must_use]
    pub fn zero(currency: CurrencyCode) -> Self {
        Self::new(0, currency)
    }

    #[must_use]
    pub fn amount(&self) -> i64 {
        self.amount
    }

    #[must_use]
    pub fn currency(&self) -> CurrencyCode {
        self.currency
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }

    /// Suma dos importes de la misma moneda.
    pub fn try_add(self, other: Self) -> Result<Self, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }

        let sum = self
            .amount
            .checked_add(other.amount)
            .ok_or(MoneyError::Overflow)?;

        Ok(Self::new(sum, self.currency))
    }

    /// Resta dos importes de la misma moneda.
    pub fn try_sub(self, other: Self) -> Result<Self, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }

        let rest = self
            .amount
            .checked_sub(other.amount)
            .ok_or(MoneyError::Overflow)?;

        Ok(Self::new(rest, self.currency))
    }

    /// Cambia el signo del importe.
    ///
    /// Solo falla con `i64::MIN`: el rango tiene un negativo de más, así que
    /// su opuesto no es representable.
    pub fn try_neg(self) -> Result<Self, MoneyError> {
        let negate = self.amount.checked_neg().ok_or(MoneyError::Overflow)?;
        Ok(Self::new(negate, self.currency))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    InvalidCurrencyCode(String),
    CurrencyMismatch {
        left: CurrencyCode,
        right: CurrencyCode,
    },
    Overflow,
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCurrencyCode(code) => {
                write!(f, "código de moneda inválido: {code:?}")
            }
            Self::CurrencyMismatch { left, right } => {
                write!(
                    f,
                    "monedas distintas: {} y {}",
                    left.as_str(),
                    right.as_str()
                )
            }
            Self::Overflow => write!(f, "importe fuera de rango"),
        }
    }
}

impl std::error::Error for MoneyError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Atajos para que cada test se lea como lo que comprueba, no como preparación.
    fn cop() -> CurrencyCode {
        CurrencyCode::new("COP").unwrap()
    }

    fn usd() -> CurrencyCode {
        CurrencyCode::new("USD").unwrap()
    }

    /// Lectura y normalización del código de tres letras. Nada de aquí
    /// toca importes.
    mod currency {
        use super::*;

        #[test]
        fn accepts_valid_code() {
            assert_eq!(CurrencyCode::new("COP").unwrap().as_str(), "COP");
        }

        #[test]
        fn normalizes_to_uppercase() {
            assert_eq!(CurrencyCode::new("usd").unwrap().as_str(), "USD");
            assert_eq!(CurrencyCode::new("eUr").unwrap().as_str(), "EUR");
        }

        #[test]
        fn rejects_wrong_length() {
            assert!(CurrencyCode::new("CO").is_err());
            assert!(CurrencyCode::new("COPP").is_err());
            assert!(CurrencyCode::new("").is_err());
        }

        #[test]
        fn rejects_non_alphabetic() {
            assert!(CurrencyCode::new("C0P").is_err());
            assert!(CurrencyCode::new("US ").is_err());
            assert!(CurrencyCode::new("CÖP").is_err());
        }

        /// La caja se normaliza, así que dos formas de escribir el mismo
        /// código son el mismo valor.
        #[test]
        fn equal_codes_compare_equal() {
            assert_eq!(
                CurrencyCode::new("COP").unwrap(),
                CurrencyCode::new("cop").unwrap()
            );
        }
    }

    /// Aritmética comprobada. Cada operación da un resultado exacto o falla
    /// en voz alta; ninguna puede dar la vuelta al contador ni mezclar monedas.
    mod arithmetic {
        use super::*;

        // --- suma ---

        #[test]
        fn adds_same_currency() {
            let a = Money::new(1500, cop());
            let b = Money::new(2500, cop());
            assert_eq!(a.try_add(b).unwrap().amount(), 4000);
        }

        #[test]
        fn rejects_different_currencies() {
            let a = Money::new(1500, cop());
            let b = Money::new(1500, usd());
            assert!(a.try_add(b).is_err());
        }

        #[test]
        fn zero_is_neutral() {
            let a = Money::new(1500, cop());
            let z = Money::zero(cop());
            assert_eq!(a.try_add(z).unwrap(), a);
        }

        /// Vigila el `checked_add` de `try_add`: un `+` normal daría la vuelta
        /// en silencio al compilar en release y convertiría un saldo en su opuesto.
        #[test]
        fn detects_overflow_on_add() {
            let a = Money::new(i64::MAX, cop());
            let b = Money::new(1, cop());
            assert_eq!(a.try_add(b), Err(MoneyError::Overflow));
        }

        /// La misma vigilancia, en el extremo bajo del rango.
        #[test]
        fn detects_underflow_on_add() {
            let a = Money::new(i64::MIN, cop());
            let b = Money::new(-1, cop());
            assert_eq!(a.try_add(b), Err(MoneyError::Overflow));
        }

        // --- resta ---

        #[test]
        fn subtracts_same_currency() {
            let a = Money::new(2500, cop());
            let b = Money::new(1500, cop());
            assert_eq!(a.try_sub(b).unwrap().amount(), 1000);
        }

        #[test]
        fn subtraction_rejects_different_currencies() {
            let a = Money::new(1500, cop());
            let b = Money::new(1500, usd());
            assert!(a.try_sub(b).is_err());
        }

        // --- negación ---

        #[test]
        fn negates_amount() {
            let a = Money::new(1500, cop());
            assert_eq!(a.try_neg().unwrap().amount(), -1500);
        }

        /// El rango de i64 tiene un negativo más que positivos, así que negar
        /// su suelo no da un resultado representable.
        #[test]
        fn detects_overflow_on_neg() {
            let a = Money::new(i64::MIN, cop());
            assert_eq!(a.try_neg(), Err(MoneyError::Overflow));
        }
    }

    /// Cómo llega un fallo a una persona. La redacción es para logs y para
    /// quien programa; la interfaz traduce la variante, no este texto.
    mod errors {
        use super::*;

        #[test]
        fn error_reads_as_a_sentence() {
            let a = Money::new(1, cop());
            let b = Money::new(1, usd());
            let err = a.try_add(b).unwrap_err();
            assert_eq!(err.to_string(), "monedas distintas: COP y USD");
        }
    }
}
