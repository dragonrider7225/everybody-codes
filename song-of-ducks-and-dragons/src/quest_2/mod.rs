use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Complex<T> {
    real: T,
    imag: T,
}

macro_rules! complex {
    ($real:expr , $imag:expr $(,)?) => {
        Complex::new($real, $imag)
    };
}

impl<T> Complex<T> {
    fn new(real: T, imag: T) -> Self {
        Self { real, imag }
    }
}

impl<T> Display for Complex<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{},{}]", self.real, self.imag)
    }
}

impl<T, U, V> Add<Complex<U>> for Complex<T>
where
    T: Add<U, Output = V>,
{
    type Output = Complex<V>;

    fn add(self, rhs: Complex<U>) -> Self::Output {
        complex![self.real + rhs.real, self.imag + rhs.imag]
    }
}

impl<T, U> AddAssign<Complex<U>> for Complex<T>
where
    T: AddAssign<U>,
{
    fn add_assign(&mut self, rhs: Complex<U>) {
        self.real += rhs.real;
        self.imag += rhs.imag;
    }
}

impl<T, U, V, W> Mul<Complex<U>> for Complex<T>
where
    T: Mul<U, Output = W>,
    for<'a> &'a T: Mul<&'a U, Output = W>,
    W: Add<W, Output = V> + Sub<W, Output = V>,
{
    type Output = Complex<V>;

    fn mul(self, rhs: Complex<U>) -> Self::Output {
        complex![
            &self.real * &rhs.real - &self.imag * &rhs.imag,
            self.real * rhs.imag + self.imag * rhs.real,
        ]
    }
}

impl<T, U, W> MulAssign<Complex<U>> for Complex<T>
where
    for<'a> &'a T: Mul<&'a U, Output = W>,
    W: Add<Output = T> + Sub<Output = T>,
{
    fn mul_assign(&mut self, rhs: Complex<U>) {
        let new_real = &self.real * &rhs.real - &self.imag * &rhs.imag;
        let new_imag = &self.real * &rhs.imag + &self.imag * &rhs.real;
        self.real = new_real;
        self.imag = new_imag;
    }
}

impl<T, U, V> Div<Complex<U>> for Complex<T>
where
    T: Div<U, Output = V>,
{
    type Output = Complex<V>;

    fn div(self, rhs: Complex<U>) -> Self::Output {
        complex![self.real / rhs.real, self.imag / rhs.imag]
    }
}

impl<T, U> DivAssign<Complex<U>> for Complex<T>
where
    T: DivAssign<U>,
{
    fn div_assign(&mut self, rhs: Complex<U>) {
        self.real /= rhs.real;
        self.imag /= rhs.imag;
    }
}

impl<T> FromStr for Complex<T>
where
    T: FromStr<Err: Into<Box<dyn std::error::Error + Send + Sync>>>,
{
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const INVALID_FORMAT: &str = "Complex number must be of the form `[<real>,<imag>]`";

        let mk_error = |msg: &str| {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{INVALID_FORMAT}: {msg}"),
            ))
        };
        match s.chars().next() {
            Some('[') => {}
            _ => return mk_error("Complex number must start with '['")?,
        }
        match s.chars().next_back() {
            Some(']') => {}
            _ => return mk_error("Complex number must end with ']'")?,
        }
        let s = &s[1..(s.len() - 1)];
        let mut parts = s.split(',');
        let real = match parts.next() {
            Some(n) => n
                .parse::<T>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
            None => return mk_error("Couldn't parse real part")?,
        };
        let imag = match parts.next() {
            Some(n) => n
                .parse::<T>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
            None => return mk_error("Coludn't parse imaginary part")?,
        };
        Ok(complex![real, imag])
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<Complex<i64>> {
    {
        let mut buf = vec![];
        input.read_until(b'=', &mut buf)?;
        if buf != b"A=" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Input data must be of the form `A=<complex>`",
            ));
        }
    }
    let input = {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        buf
    };
    let a = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing value of A"))?
        .parse::<Complex<i64>>()?;
    let mut ret = complex![0, 0];
    for _ in 0..3 {
        ret *= ret;
        ret /= complex![10, 10];
        ret += a;
    }
    Ok(ret)
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    {
        let mut buf = vec![];
        input.read_until(b'=', &mut buf)?;
        if buf != b"A=" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Input data must be of the form `A=<complex>`",
            ));
        }
    }
    let input = {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        buf
    };
    let a = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing value of A"))?
        .parse::<Complex<i64>>()?;
    Ok((0..=100)
        .map(|d_real| d_real * 10)
        .flat_map(|d_real| {
            (0..=100)
                .map(|d_imag| d_imag * 10)
                .map(move |d_imag| complex![d_real, d_imag])
        })
        .map(|delta| a + delta)
        .filter(|&p| {
            (0..100)
                .try_fold(complex![0, 0], |mut acc, _| {
                    acc *= acc;
                    acc /= complex![100_000, 100_000];
                    acc += p;
                    if acc.real.abs() > 1_000_000 || acc.imag.abs() > 1_000_000 {
                        return None;
                    }
                    Some(acc)
                })
                .is_some()
        })
        .count())
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    {
        let mut buf = vec![];
        input.read_until(b'=', &mut buf)?;
        if buf != b"A=" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Input data must be of the form `A=<complex>`",
            ));
        }
    }
    let input = {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        buf
    };
    let a = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing value of A"))?
        .parse::<Complex<i64>>()?;
    Ok((0..=1000)
        .flat_map(|d_real| (0..=1000).map(move |d_imag| complex![d_real, d_imag]))
        .map(|delta| a + delta)
        .filter(|&p| {
            (0..100)
                .try_fold(complex![0, 0], |mut acc, _| {
                    acc *= acc;
                    acc /= complex![100_000, 100_000];
                    acc += p;
                    if acc.real.abs() > 1_000_000 || acc.imag.abs() > 1_000_000 {
                        return None;
                    }
                    Some(acc)
                })
                .is_some()
        })
        .count())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 2 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_02-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 2 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_02-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 2 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_02-3.txt"
            )?))?
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_part1() -> io::Result<()> {
        const TEST_DATA: &str = "A=[25,9]\n";
        let expected = complex![357, 862];
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = "A=[35300,-64910]\n";
        let expected = 4076;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = "A=[35300,-64910]\n";
        let expected = 406_954;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
