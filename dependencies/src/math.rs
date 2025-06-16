use num::BigInt;

/// Compute the outputs of the [extended Euclidean
/// algorithm](https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtendedGcd {
    /// The initial values.
    pub values: (BigInt, BigInt),
    /// The coefficients of Bézout's identity. By definition, it is always true that `values.0 *
    /// coefficients.0 + values.1 * coefficients.1 == gcd` when no overflow occurs.
    pub coefficients: (BigInt, BigInt),
    /// The greatest common divisor of `values.0` and `values.1`.
    pub gcd: BigInt,
}

impl<A, B> From<(A, B)> for ExtendedGcd
where
    A: Into<BigInt>,
    B: Into<BigInt>,
{
    fn from((a, b): (A, B)) -> Self {
        let (a, b) = (a.into(), b.into());
        let (mut old_r, mut r) = (a.clone(), b.clone());
        let (mut old_s, mut s) = (BigInt::from(1), BigInt::ZERO);
        let (mut old_t, mut t) = (BigInt::ZERO, BigInt::from(1));

        while r != BigInt::ZERO {
            let quotient = &old_r / &r;
            (old_r, r) = (r.clone(), old_r - &quotient * r);
            (old_s, s) = (s.clone(), old_s - &quotient * s);
            (old_t, t) = (t.clone(), old_t - &quotient * t);
        }
        ExtendedGcd {
            values: (a, b),
            coefficients: (old_s, old_t),
            gcd: old_r,
        }
    }
}

/// Computes the minimum non-negative `x` such that `x % n == a` for all `(a, n)` in
/// `remainders_and_denominators`.
pub fn chinese_remainder_theorem(
    remainders_and_denominators: impl IntoIterator<Item = (BigInt, BigInt)>,
) -> Option<BigInt> {
    remainders_and_denominators
        .into_iter()
        .try_fold((BigInt::ZERO, BigInt::from(1)), |(a1, n1), (a2, n2)| {
            let combination = ExtendedGcd::from((n1.clone(), n2.clone()));
            if &a1 % &combination.gcd == &a2 % &combination.gcd {
                let denominator = &n1 * &n2 / &combination.gcd;
                let (n1, n2) = (n1 / &combination.gcd, n2 / combination.gcd);
                let remainder =
                    a1 * n2 * combination.coefficients.1 + a2 * n1 * combination.coefficients.0;
                let remainder = remainder % &denominator;
                Some((remainder, denominator))
            } else {
                None
            }
        })
        .map(|(x, denominator)| if x < BigInt::ZERO { x + denominator } else { x })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extended_gcd() {
        let expected = ExtendedGcd {
            values: (240.into(), 46.into()),
            coefficients: ((-9).into(), 47.into()),
            gcd: 2.into(),
        };
        let actual = ExtendedGcd::from((240, 46));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_crt() {
        // *** * | **** * | ******* *
        // *** * | **** * | ******* *
        // ***   | **** * | *******
        // ***   | ****
        // ***   | ****
        // ***
        // ***
        let remainders_and_denominators = [(2, 3), (3, 5), (2, 7)];
        let expected = Some(23.into());
        let actual = chinese_remainder_theorem(
            remainders_and_denominators
                .into_iter()
                .map(|(a, n)| (a.into(), n.into())),
        );
        assert_eq!(expected, actual);
    }
}
