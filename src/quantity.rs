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

    /// Rounds to the nearest multiple of `step` (which must be positive),
    /// with ties going away from zero. Done in integers so 1/8 steps land
    /// exactly on measuring-spoon fractions.
    pub fn round_to(self, step: Quantity) -> Quantity {
        assert!(step.num > 0, "rounding step must be positive");
        let n = self.num * step.den;
        let d = self.den * step.num;
        let magnitude = (2 * n.abs() + d) / (2 * d);
        let k = if n < 0 { -magnitude } else { magnitude };
        Quantity::new(k * step.num, step.den)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reduces_to_lowest_terms() {
        let q = Quantity::new(4, 8);
        assert_eq!(q, Quantity::new(1, 2));
        assert_eq!(q.num, 1);
        assert_eq!(q.den, 2);
    }

    #[test]
    fn new_normalizes_negative_denominator() {
        let q = Quantity::new(1, -2);
        assert_eq!(q.num, -1);
        assert_eq!(q.den, 2);
    }

    #[test]
    #[should_panic(expected = "denominator cannot be zero")]
    fn new_rejects_zero_denominator() {
        Quantity::new(1, 0);
    }

    #[test]
    fn whole_is_denominator_one() {
        let q = Quantity::whole(3);
        assert_eq!(q, Quantity::new(3, 1));
    }

    #[test]
    fn scaled_by_multiplies_and_reduces() {
        // 1 1/2 (3/2) scaled by 3/2 is 2 1/4 (9/4), matching the README example.
        let onehalf = Quantity::new(3, 2);
        let factor = Quantity::new(3, 2);
        assert_eq!(onehalf.scaled_by(factor), Quantity::new(9, 4));
    }

    #[test]
    fn scaled_by_is_exact_over_repeated_applications() {
        // Scaling down then back up by the reciprocal should land exactly
        // on the original value, not something off by float error.
        let start = Quantity::new(1, 3);
        let down = start.scaled_by(Quantity::new(1, 7));
        let back = down.scaled_by(Quantity::new(7, 1));
        assert_eq!(back, start);
    }

    #[test]
    fn round_to_nearest_eighth() {
        // 5/16 is a tie between 2/8 and 3/8 and goes up.
        assert_eq!(Quantity::new(5, 16).round_to(Quantity::new(1, 8)), Quantity::new(3, 8));
        // 1/3 is closer to 3/8 than to 2/8.
        assert_eq!(Quantity::new(1, 3).round_to(Quantity::new(1, 8)), Quantity::new(3, 8));
    }

    #[test]
    fn round_to_keeps_exact_values() {
        let q = Quantity::new(9, 4);
        assert_eq!(q.round_to(Quantity::new(1, 8)), q);
    }

    #[test]
    fn round_to_can_reach_zero() {
        assert_eq!(Quantity::new(1, 20).round_to(Quantity::new(1, 4)), Quantity::whole(0));
    }

    #[test]
    fn round_to_handles_negative_ties_away_from_zero() {
        assert_eq!(Quantity::new(-5, 16).round_to(Quantity::new(1, 8)), Quantity::new(-3, 8));
    }

    #[test]
    fn round_to_accepts_non_unit_step() {
        // Nearest multiple of 1/2: 7/4 is a tie and rounds up to 2.
        assert_eq!(Quantity::new(7, 4).round_to(Quantity::new(1, 2)), Quantity::whole(2));
    }

    #[test]
    fn as_f64_converts() {
        assert_eq!(Quantity::new(1, 4).as_f64(), 0.25);
        assert_eq!(Quantity::new(3, 2).as_f64(), 1.5);
    }

    #[test]
    fn parse_tokens_whole_number() {
        let (q, consumed) = Quantity::parse_tokens(&["3", "eggs"]).unwrap();
        assert_eq!(q, Quantity::whole(3));
        assert_eq!(consumed, 1);
    }

    #[test]
    fn parse_tokens_decimal() {
        let (q, consumed) = Quantity::parse_tokens(&["0.5", "tsp"]).unwrap();
        assert_eq!(q, Quantity::new(1, 2));
        assert_eq!(consumed, 1);
    }

    #[test]
    fn parse_tokens_multi_digit_decimal() {
        let (q, _) = Quantity::parse_tokens(&["1.25"]).unwrap();
        assert_eq!(q, Quantity::new(5, 4));
    }

    #[test]
    fn parse_tokens_simple_fraction() {
        let (q, consumed) = Quantity::parse_tokens(&["1/2", "cup"]).unwrap();
        assert_eq!(q, Quantity::new(1, 2));
        assert_eq!(consumed, 1);
    }

    #[test]
    fn parse_tokens_mixed_number_consumes_two_tokens() {
        let (q, consumed) = Quantity::parse_tokens(&["1", "1/2", "cups", "milk"]).unwrap();
        assert_eq!(q, Quantity::new(3, 2));
        assert_eq!(consumed, 2);
    }

    #[test]
    fn parse_tokens_does_not_treat_trailing_fraction_as_mixed_number() {
        // "3" followed by a token that isn't a fraction (a unit) should not
        // be combined; only the whole number is consumed.
        let (q, consumed) = Quantity::parse_tokens(&["3", "cups", "flour"]).unwrap();
        assert_eq!(q, Quantity::whole(3));
        assert_eq!(consumed, 1);
    }

    #[test]
    fn parse_tokens_rejects_empty_input() {
        assert!(Quantity::parse_tokens(&[]).is_err());
    }

    #[test]
    fn parse_tokens_rejects_garbage() {
        assert!(Quantity::parse_tokens(&["banana"]).is_err());
    }

    #[test]
    fn parse_tokens_rejects_zero_denominator_fraction() {
        assert!(Quantity::parse_tokens(&["1/0"]).is_err());
    }

    #[test]
    fn display_whole_number() {
        assert_eq!(Quantity::whole(3).to_string(), "3");
    }

    #[test]
    fn display_fraction_only() {
        assert_eq!(Quantity::new(1, 2).to_string(), "1/2");
    }

    #[test]
    fn display_mixed_number() {
        assert_eq!(Quantity::new(9, 4).to_string(), "2 1/4");
    }

    #[test]
    fn display_zero() {
        assert_eq!(Quantity::new(0, 5).to_string(), "0");
    }
}
