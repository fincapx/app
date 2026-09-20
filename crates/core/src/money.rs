use std::fmt;

// ISO 4217 currency code: three ASCII letters, stored uppercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CurrencyCode([u8; 3]);

impl CurrencyCode {
    /// Accepts three ASCII letters in any case. Normalizes to uppercase.
    pub fn new(code: &str) -> Result<Self, MoneyError> {
        // Text is seen like list of bytes. "COP" --> [67, 79, 80]
        let bytes = code.as_bytes();

        // exactly three bytes?
        if bytes.len() != 3 {
            return Err(MoneyError::InvalidCurrencyCode(code.to_string()));
        }

        // Are three letters ASCII?
        for b in bytes {
            if !b.is_ascii_alphabetic() {
                return Err(MoneyError::InvalidCurrencyCode(code.to_string()));
            }
        }

        // Uppercase all three, wrap in Ok
        Ok(Self([
            bytes[0].to_ascii_uppercase(),
            bytes[1].to_ascii_uppercase(),
            bytes[2].to_ascii_uppercase(),
        ]))
    }

    /// Returns the code as a string slice.
    ///
    /// Never panics in practice: 'new' is the only constructor and it
    /// only accepts ASCII, which is always valid UTF-8.
    #[must_use]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }
}

/// A monetary amount in a single currency, stored in minor units.
///
/// The scale (how many decimals the currency has) lives in the currency
/// table, not here: all arithmetic happens in minor units.
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

    /// Adds two amounts of the same currency
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
                write!(f, "invalid currency code: {code:?}")
            }
            Self::CurrencyMismatch { left, right } => {
                write!(
                    f,
                    "currency mismatch: {} and {}",
                    left.as_str(),
                    right.as_str()
                )
            }
            Self::Overflow => write!(f, "amount out of range"),
        }
    }
}

impl std::error::Error for MoneyError {}

#[cfg(test)]
mod tests {
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

    #[test]
    fn equal_codes_compare_equal() {
        assert_eq!(
            CurrencyCode::new("COP").unwrap(),
            CurrencyCode::new("cop").unwrap()
        );
    }

    fn cop() -> CurrencyCode {
        CurrencyCode::new("COP").unwrap()
    }

    fn usd() -> CurrencyCode {
        CurrencyCode::new("USD").unwrap()
    }

    #[test]
    fn adds_same_currency() {
        let a = Money::new(1500, cop());
        let b = Money::new(2500, cop());
        assert_eq!(a.try_add(b).unwrap().amount(), 4000);
    }

    #[test]
    fn subtracts_same_currency() {
        let a = Money::new(2500, cop());
        let b = Money::new(1500, cop());
        assert_eq!(a.try_sub(b).unwrap().amount(), 1000);
    }

    #[test]
    fn rejects_different_currencies() {
        let a = Money::new(1500, cop());
        let b = Money::new(1500, usd());
        assert!(a.try_add(b).is_err());
    }

    #[test]
    fn subtraction_rejects_different_currencies() {
        let a = Money::new(1500, cop());
        let b = Money::new(1500, usd());
        assert!(a.try_sub(b).is_err());
    }

    #[test]
    fn zero_is_neutral() {
        let a = Money::new(1500, cop());
        let z = Money::zero(cop());
        assert_eq!(a.try_add(z).unwrap(), a);
    }

    #[test]
    fn detects_overflow_on_add() {
        let a = Money::new(i64::MAX, cop());
        let b = Money::new(1, cop());
        assert_eq!(a.try_add(b), Err(MoneyError::Overflow));
    }

    #[test]
    fn detects_underflow_on_add() {
        let a = Money::new(i64::MIN, cop());
        let b = Money::new(-1, cop());
        assert_eq!(a.try_add(b), Err(MoneyError::Overflow));
    }

    #[test]
    fn error_reads_as_a_sentence() {
        let a = Money::new(1, cop());
        let b = Money::new(1, usd());
        let err = a.try_add(b).unwrap_err();
        assert_eq!(err.to_string(), "currency mismatch: COP and USD");
    }
}
