//! Bounded exact arithmetic for authored DSL expressions and request settlement.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};

const MAX_BITS: u64 = 4096;
const MAX_LITERAL_DIGITS: usize = 1000;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// Numeric form input may use a sign, leading decimal point or exponent. Scripts
/// keep the smaller decimal grammar; normalization never passes through f64.
pub fn canonical_decimal(text: &str) -> Result<String, String> {
    let text = text.trim();
    let (negative, unsigned) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let (mantissa, exponent) = if let Some(index) = unsigned.find(['e', 'E']) {
        let exponent = unsigned[index + 1..]
            .parse::<i32>()
            .map_err(|_| "Invalid number exponent.")?;
        if exponent.unsigned_abs() > 1000 {
            return Err("Numeric literal exponent exceeds its size limit.".into());
        }
        (&unsigned[..index], exponent)
    } else {
        (unsigned, 0)
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if whole.len() + fraction.len() == 0
        || !whole
            .bytes()
            .chain(fraction.bytes())
            .all(|c| c.is_ascii_digit())
    {
        return Err("Expected a finite number.".into());
    }
    if whole.len() + fraction.len() > MAX_LITERAL_DIGITS {
        return Err("Numeric literal exceeds its size limit.".into());
    }
    let digits = format!("{whole}{fraction}");
    let point = whole.len() as i32 + exponent;
    let decimal = if point <= 0 {
        format!("0.{}{digits}", "0".repeat((-point) as usize))
    } else if point >= digits.len() as i32 {
        format!("{digits}{}", "0".repeat(point as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    };
    if decimal.bytes().filter(u8::is_ascii_digit).count() > MAX_LITERAL_DIGITS {
        return Err("Numeric literal exceeds its size limit.".into());
    }
    Ok(if negative {
        format!("-{decimal}")
    } else {
        decimal
    })
}

pub fn input_decimal(text: &str) -> Result<ExactNumber, String> {
    let canonical = canonical_decimal(text)?;
    if let Some(unsigned) = canonical.strip_prefix('-') {
        ExactNumber::decimal(unsigned).map(|number| number.negate())
    } else {
        ExactNumber::decimal(&canonical)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExactNumber(BigRational);

impl From<i64> for ExactNumber {
    fn from(value: i64) -> Self {
        Self(BigRational::from_integer(value.into()))
    }
}

impl ExactNumber {
    pub(crate) fn from_rational(value: BigRational) -> Result<Self, String> {
        Self::bounded(value)
    }

    fn bounded(value: BigRational) -> Result<Self, String> {
        if value.numer().bits() > MAX_BITS || value.denom().bits() > MAX_BITS {
            return Err("Exact arithmetic exceeds its size limit.".into());
        }
        Ok(Self(value))
    }

    /// Parse the unsigned decimal grammar emitted by the lexer; signs are unary AST nodes.
    pub fn decimal(text: &str) -> Result<Self, String> {
        let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
        if whole.is_empty()
            || !whole
                .bytes()
                .chain(fraction.bytes())
                .all(|c| c.is_ascii_digit())
            || (text.contains('.') && fraction.is_empty())
        {
            return Err("Expected a decimal literal.".into());
        }
        if whole.len() + fraction.len() > MAX_LITERAL_DIGITS {
            return Err("Numeric literal exceeds its size limit.".into());
        }
        let digits = format!("{whole}{fraction}");
        let numerator =
            BigInt::parse_bytes(digits.as_bytes(), 10).ok_or("Expected a decimal literal.")?;
        let denominator = BigInt::from(10).pow(fraction.len() as u32);
        Self::bounded(BigRational::new(numerator, denominator))
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn negate(&self) -> Self {
        Self(-&self.0)
    }

    pub fn absolute(&self) -> Self {
        Self(self.0.abs())
    }

    pub fn floor(&self) -> Self {
        Self(self.0.floor())
    }

    pub fn ceil(&self) -> Self {
        Self(self.0.ceil())
    }

    pub fn add(&self, other: &Self) -> Result<Self, String> {
        Self::bounded(&self.0 + &other.0)
    }

    pub fn subtract(&self, other: &Self) -> Result<Self, String> {
        Self::bounded(&self.0 - &other.0)
    }

    pub fn multiply(&self, other: &Self) -> Result<Self, String> {
        Self::bounded(&self.0 * &other.0)
    }

    pub fn divide(&self, other: &Self) -> Result<Self, String> {
        if other.is_zero() {
            return Err("Division by zero.".into());
        }
        Self::bounded(&self.0 / &other.0)
    }

    /// Remainder uses a quotient truncated towards zero, including negative operands.
    pub fn remainder(&self, other: &Self) -> Result<Self, String> {
        if other.is_zero() {
            return Err("Division by zero.".into());
        }
        let quotient = (&self.0 / &other.0).to_integer();
        Self::bounded(&self.0 - &other.0 * quotient)
    }

    pub fn to_safe_integer(&self) -> Result<u64, String> {
        if !self.0.denom().is_one() {
            return Err("Expected an integer.".into());
        }
        self.0
            .numer()
            .to_u64()
            .filter(|n| *n <= MAX_SAFE_INTEGER)
            .ok_or_else(|| "Expected a nonnegative safe integer.".into())
    }

    pub fn to_safe_ratio(&self) -> Result<(u64, u64), String> {
        let numerator = self.0.numer().to_u64().filter(|n| *n <= MAX_SAFE_INTEGER);
        let denominator = self.0.denom().to_u64().filter(|n| *n <= MAX_SAFE_INTEGER);
        numerator
            .zip(denominator)
            .ok_or_else(|| "Expected a nonnegative ratio within the safe integer range.".into())
    }

    /// Presentation only; request arithmetic and settlement must use exact methods.
    pub fn to_display_number(&self) -> Result<f64, String> {
        if self.0.abs() > BigRational::from_integer(BigInt::from(MAX_SAFE_INTEGER)) {
            return Err("Expected a finite safe number.".into());
        }
        self.0
            .to_f64()
            .filter(|v| v.is_finite())
            .ok_or_else(|| "Expected a finite safe number.".into())
    }

    /// Seconds settle once at the request boundary, without rounding sub-microsecond values.
    pub fn to_microseconds(&self) -> Result<u64, String> {
        self.multiply(&Self::from(1_000_000))?.to_safe_integer()
    }

    pub fn floor_product(&self, integer: u64) -> Result<u64, String> {
        Self::bounded(&self.0 * BigInt::from(integer))?
            .floor()
            .to_safe_integer()
    }
}
