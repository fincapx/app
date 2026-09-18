/// ISO 4217 currency code: three ASCII letters, stored uppercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CurrencyCode([u8; 3]);

impl CurrencyCode {
    /// Accepts three ASCII letters in any case. Normalizes to uppercase.
    pub fn new(code: &str) -> Result<Self, MoneyError> {
        // Text is seen like list of bytes. "COP" --> [67, 79, 80]
        let bytes = code.as_bytes();

        // exactly thee bytes?
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
        Ok(CurrencyCode([
            bytes[0].to_ascii_uppercase(),
            bytes[1].to_ascii_uppercase(),
            bytes[2].to_ascii_uppercase(),
        ]))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    InvalidCurrencyCode(String),
}

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
}
