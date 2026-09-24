use std::fmt;

/// Una fecha de calendario: ano, mes y dia. Sin hora ni zona horaria.
///
/// El orden de los campos no es casual: al comparar, Rust mira primero el
/// ano, luego el mes y luego el dia, que es justo el orden cronologico.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: i16,
    month: u8,
    day: u8,
}

impl Date {
    /// Crea una fecha, rechazando las que no existen.
    pub fn new(year: i16, month: u8, day: u8) -> Result<Self, DateError> {
        if !(1..=12).contains(&month) {
            return Err(DateError::InvalidMonth(month));
        }

        if day < 1 || day > days_in_month(year, month) {
            return Err(DateError::InvalidDay { year, month, day });
        }

        Ok(Self { year, month, day })
    }

    #[must_use]
    pub fn year(self) -> i16 {
        self.year
    }

    #[must_use]
    pub fn month(self) -> u8 {
        self.month
    }

    #[must_use]
    pub fn day(self) -> u8 {
        self.day
    }
}

/// Cuantos dias tiene ese mes de ese ano.
const fn days_in_month(year: i16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Bisiesto: divisible entre 4, salvo los siglos que no lo sean entre 400.
const fn is_leap_year(year: i16) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

impl fmt::Display for Date {
    /// Formato ISO-8601, que es tambien el que guarda `SQLite`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateError {
    InvalidMonth(u8),
    InvalidDay { year: i16, month: u8, day: u8 },
}

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMonth(month) => {
                write!(f, "mes fuera de rango: {month}")
            }
            Self::InvalidDay { year, month, day } => {
                write!(f, "el dia {day} no existe en el mes {month} de {year}")
            }
        }
    }
}

impl std::error::Error for DateError {}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn accepts_valid_date() {
        assert!(Date::new(2026, 9, 23).is_ok());
    }

    // Los dos bordes del rango.
    #[test]
    fn rejects_invalid_month() {
        assert!(Date::new(2026, 0, 23).is_err());
        assert!(Date::new(2026, 13, 23).is_err());
    }

    // El 31 de septiembre prueba que se consulta `days_in_month`, no un rango fijo.
    #[test]
    fn rejects_invalid_day() {
        assert!(Date::new(2026, 1, 0).is_err());
        assert!(Date::new(2026, 9, 31).is_err());
    }

    // 2000 es siglo, pero divisible entre 400: sí es bisiesto.
    #[test]
    fn accepts_leap_day() {
        assert!(Date::new(2024, 2, 29).is_ok());
        assert!(Date::new(2000, 2, 29).is_ok());
    }

    // 1900 divide entre 4 y no es bisiesto: caza la condición simplificada.
    #[test]
    fn rejects_leap_day_in_common_year() {
        assert!(Date::new(1900, 2, 29).is_err());
        assert!(Date::new(2025, 2, 29).is_err());
    }

    // Vigila el orden de los campos del struct.
    #[test]
    fn orders_chronologically() {
        let a = Date::new(2026, 1, 20).unwrap();
        let b = Date::new(2026, 3, 20).unwrap();
        assert!(b > a);
    }

    // Sin los ceros, `SQLite` ordenaría octubre antes que febrero.
    #[test]
    fn formats_with_leading_zeros() {
        assert_eq!(Date::new(2026, 1, 7).unwrap().to_string(), "2026-01-07");
    }
}
