use std::fmt;

/// An exact rational quantity (numerator/denominator), kept reduced so
/// repeated scaling never accumulates floating point rounding error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quantity {
    pub num: i64,
    pub den: i64,
}

impl Quantity {
    pub fn new(num: i64, den: i64) -> Quantity {
        assert!(den != 0, "quantity denominator cannot be zero");
        let (num, den) = if den < 0 { (-num, -den) } else { (num, den) };
        let g = gcd(num.abs(), den).max(1);
        Quantity {
            num: num / g,
            den: den / g,
        }
    }

    pub fn whole(n: i64) -> Quantity {
        Quantity::new(n, 1)
    }

    pub fn scaled_by(self, factor: Quantity) -> Quantity {
        Quantity::new(self.num * factor.num, self.den * factor.den)
    }

    pub fn as_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// Parses the quantity at the front of an ingredient line, which may
    /// be a whole number ("3"), a decimal ("0.5"), a simple fraction
    /// ("1/2"), or a mixed number split across two tokens ("1 1/2").
    /// Returns the parsed quantity and how many tokens it consumed.
    pub fn parse_tokens(tokens: &[&str]) -> Result<(Quantity, usize), String> {
        if tokens.is_empty() {
            return Err("expected a quantity, found nothing".to_string());
        }
        let first = parse_single(tokens[0])?;
        let first_is_whole = !tokens[0].contains('/') && !tokens[0].contains('.');
        if first_is_whole && tokens.len() > 1 && tokens[1].contains('/') {
            if let Ok(frac) = parse_single(tokens[1]) {
                let combined = Quantity::new(
                    first.num * frac.den + frac.num * first.den,
                    first.den * frac.den,
                );
                return Ok((combined, 2));
            }
        }
        Ok((first, 1))
    }
}

fn parse_single(tok: &str) -> Result<Quantity, String> {
    if let Some((n, d)) = tok.split_once('/') {
        let n: i64 = n
            .parse()
            .map_err(|_| format!("bad fraction numerator: {}", tok))?;
        let d: i64 = d
            .parse()
            .map_err(|_| format!("bad fraction denominator: {}", tok))?;
        if d == 0 {
            return Err(format!("fraction with zero denominator: {}", tok));
        }
        Ok(Quantity::new(n, d))
    } else if let Some(dot) = tok.find('.') {
        let whole: i64 = tok[..dot]
            .parse()
            .map_err(|_| format!("bad number: {}", tok))?;
        let frac_str = &tok[dot + 1..];
        if frac_str.is_empty() || !frac_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("bad number: {}", tok));
        }
        let frac_digits: i64 = frac_str.parse().map_err(|_| format!("bad number: {}", tok))?;
        let den = 10i64.pow(frac_str.len() as u32);
        let sign = if whole < 0 { -1 } else { 1 };
        Ok(Quantity::new(whole * den + sign * frac_digits, den))
    } else {
        let n: i64 = tok.parse().map_err(|_| format!("not a number: {}", tok))?;
        Ok(Quantity::whole(n))
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.num / self.den;
        let rem = self.num % self.den;
        if rem == 0 {
            write!(f, "{}", whole)
        } else if whole == 0 {
            write!(f, "{}/{}", rem, self.den)
        } else {
            write!(f, "{} {}/{}", whole, rem.abs(), self.den)
        }
    }
}
