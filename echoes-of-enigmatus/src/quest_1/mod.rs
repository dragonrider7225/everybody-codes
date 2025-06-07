use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

fn eni(n: u64, exp: u64, r#mod: u64) -> u64 {
    let mut score = 1;
    let mut remainders = vec![];
    for _ in 0..exp {
        score *= n;
        score %= r#mod;
        remainders.push(score);
    }
    remainders.into_iter().rev().fold(0, |acc, segment| {
        let width = 1 + segment.checked_ilog10().unwrap_or(0);
        acc * 10u64.pow(width) + segment
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Variable {
    A(u64),
    B(u64),
    C(u64),
    X(u64),
    Y(u64),
    Z(u64),
    M(u64),
}

fn small_eni(n: u64, exp: u64, r#mod: u64) -> u64 {
    let mut score = 1;
    let mut remainders = vec![];
    let mut num_throwaway_reverse_bits = exp.saturating_sub(5).reverse_bits();
    for _ in 0..u64::BITS {
        score *= score;
        score %= r#mod;
        if num_throwaway_reverse_bits % 2 == 1 {
            score = (score * n) % r#mod;
        }
        num_throwaway_reverse_bits /= 2;
    }
    for _ in 0..exp.min(5) {
        score *= n;
        score %= r#mod;
        remainders.push(score);
    }
    remainders.into_iter().rev().fold(0, |acc, segment| {
        let width = 1 + segment.checked_ilog10().unwrap_or(0);
        acc * 10u64.pow(width) + segment
    })
}

fn big_eni(n: u64, exp: u64, r#mod: u64) -> u64 {
    let mut score = 1;
    let mut remainders = vec![];
    for i in 0..exp {
        score *= n;
        score %= r#mod;
        if let Some(idx) = remainders.iter().position(|&remainder| remainder == score) {
            let precycle = remainders[..idx].iter().copied().sum::<u64>();
            let remaining = exp - i;
            let cycle_length = (remainders.len() - idx) as u64;
            // eprintln!("Reached end of {cycle_length}-step cycle in {i} steps");
            let full_cycles = remaining / cycle_length + 1;
            let leftover = remaining % cycle_length;
            // eprintln!("{full_cycles} cycles remaining");
            // eprintln!("{leftover} steps after final cycle");
            let cycles = full_cycles * remainders[idx..].iter().copied().sum::<u64>();
            let leftover = remainders[idx..(idx + leftover as usize)]
                .iter()
                .copied()
                .sum::<u64>();
            let ret = precycle + cycles + leftover;
            // eprintln!("{ret} = {precycle} + {cycles} + {leftover}");
            return ret;
        }
        remainders.push(score);
    }
    remainders.into_iter().rev().fold(0, |acc, segment| {
        let width = 1 + segment.checked_ilog10().unwrap_or(0);
        acc * 10u64.pow(width) + segment
    })
}

fn part1(input: &mut dyn BufRead) -> io::Result<u64> {
    input
        .lines()
        .map(|line| {
            let line = line?;
            let mut parts = line.split(' ').map(|value| match &value[..1] {
                "A" => Variable::A(value[2..].parse().unwrap()),
                "B" => Variable::B(value[2..].parse().unwrap()),
                "C" => Variable::C(value[2..].parse().unwrap()),
                "X" => Variable::X(value[2..].parse().unwrap()),
                "Y" => Variable::Y(value[2..].parse().unwrap()),
                "Z" => Variable::Z(value[2..].parse().unwrap()),
                "M" => Variable::M(value[2..].parse().unwrap()),
                name => panic!("Invalid variable name: {name:?}"),
            });
            let a = match parts.next() {
                Some(Variable::A(a)) => a,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-A variable first: {part:?}"),
                    ))
                }
            };
            let b = match parts.next() {
                Some(Variable::B(b)) => b,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-B variable second: {part:?}"),
                    ))
                }
            };
            let c = match parts.next() {
                Some(Variable::C(c)) => c,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-C variable third: {part:?}"),
                    ))
                }
            };
            let x = match parts.next() {
                Some(Variable::X(x)) => x,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-X variable fourth: {part:?}"),
                    ))
                }
            };
            let y = match parts.next() {
                Some(Variable::Y(y)) => y,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-Y variable fifth: {part:?}"),
                    ))
                }
            };
            let z = match parts.next() {
                Some(Variable::Z(z)) => z,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-Z variable sixth: {part:?}"),
                    ))
                }
            };
            let m = match parts.next() {
                Some(Variable::M(m)) => m,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-M variable seventh: {part:?}"),
                    ))
                }
            };
            Ok(eni(a, x, m) + eni(b, y, m) + eni(c, z, m))
        })
        .try_fold(None, |acc: Option<u64>, value| -> io::Result<_> {
            let value = value?;
            match acc {
                None => Ok(Some(value)),
                Some(acc) => {
                    if value > acc {
                        Ok(Some(value))
                    } else {
                        Ok(Some(acc))
                    }
                }
            }
        })
        .and_then(|value| {
            value.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Got no lines"))
        })
}

fn part2(input: &mut dyn BufRead) -> io::Result<u64> {
    input
        .lines()
        .map(|line| {
            let line = line?;
            let mut parts = line.split(' ').map(|value| match &value[..1] {
                "A" => Variable::A(value[2..].parse().unwrap()),
                "B" => Variable::B(value[2..].parse().unwrap()),
                "C" => Variable::C(value[2..].parse().unwrap()),
                "X" => Variable::X(value[2..].parse().unwrap()),
                "Y" => Variable::Y(value[2..].parse().unwrap()),
                "Z" => Variable::Z(value[2..].parse().unwrap()),
                "M" => Variable::M(value[2..].parse().unwrap()),
                name => panic!("Invalid variable name: {name:?}"),
            });
            let a = match parts.next() {
                Some(Variable::A(a)) => a,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-A variable first: {part:?}"),
                    ))
                }
            };
            let b = match parts.next() {
                Some(Variable::B(b)) => b,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-B variable second: {part:?}"),
                    ))
                }
            };
            let c = match parts.next() {
                Some(Variable::C(c)) => c,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-C variable third: {part:?}"),
                    ))
                }
            };
            let x = match parts.next() {
                Some(Variable::X(x)) => x,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-X variable fourth: {part:?}"),
                    ))
                }
            };
            let y = match parts.next() {
                Some(Variable::Y(y)) => y,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-Y variable fifth: {part:?}"),
                    ))
                }
            };
            let z = match parts.next() {
                Some(Variable::Z(z)) => z,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-Z variable sixth: {part:?}"),
                    ))
                }
            };
            let m = match parts.next() {
                Some(Variable::M(m)) => m,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-M variable seventh: {part:?}"),
                    ))
                }
            };
            Ok(small_eni(a, x, m) + small_eni(b, y, m) + small_eni(c, z, m))
        })
        .try_fold(None, |acc: Option<u64>, value| -> io::Result<_> {
            let value = value?;
            match acc {
                None => Ok(Some(value)),
                Some(acc) => {
                    if value > acc {
                        Ok(Some(value))
                    } else {
                        Ok(Some(acc))
                    }
                }
            }
        })
        .and_then(|value| {
            value.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Got no lines"))
        })
}

fn part3(input: &mut dyn BufRead) -> io::Result<u64> {
    input
        .lines()
        .map(|line| {
            let line = line?;
            let mut parts = line.split(' ').map(|value| match &value[..1] {
                "A" => Variable::A(value[2..].parse().unwrap()),
                "B" => Variable::B(value[2..].parse().unwrap()),
                "C" => Variable::C(value[2..].parse().unwrap()),
                "X" => Variable::X(value[2..].parse().unwrap()),
                "Y" => Variable::Y(value[2..].parse().unwrap()),
                "Z" => Variable::Z(value[2..].parse().unwrap()),
                "M" => Variable::M(value[2..].parse().unwrap()),
                name => panic!("Invalid variable name: {name:?}"),
            });
            let a = match parts.next() {
                Some(Variable::A(a)) => a,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-A variable first: {part:?}"),
                    ))
                }
            };
            let b = match parts.next() {
                Some(Variable::B(b)) => b,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-B variable second: {part:?}"),
                    ))
                }
            };
            let c = match parts.next() {
                Some(Variable::C(c)) => c,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-C variable third: {part:?}"),
                    ))
                }
            };
            let x = match parts.next() {
                Some(Variable::X(x)) => x,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-X variable fourth: {part:?}"),
                    ))
                }
            };
            let y = match parts.next() {
                Some(Variable::Y(y)) => y,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-Y variable fifth: {part:?}"),
                    ))
                }
            };
            let z = match parts.next() {
                Some(Variable::Z(z)) => z,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-Z variable sixth: {part:?}"),
                    ))
                }
            };
            let m = match parts.next() {
                Some(Variable::M(m)) => m,
                part => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Got non-M variable seventh: {part:?}"),
                    ))
                }
            };
            Ok(big_eni(a, x, m) + big_eni(b, y, m) + big_eni(c, z, m))
        })
        .try_fold(None, |acc: Option<u64>, value| -> io::Result<_> {
            let value = value?;
            match acc {
                None => Ok(Some(value)),
                Some(acc) => {
                    if value > acc {
                        Ok(Some(value))
                    } else {
                        Ok(Some(acc))
                    }
                }
            }
        })
        .and_then(|value| {
            value.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Got no lines"))
        })
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Echoes of Enigmatus Quest 1 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "echoes-of-enigmatus_01-1.txt"
            )?))?
        );
    }
    {
        println!("Echoes of Enigmatus Quest 1 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "echoes-of-enigmatus_01-2.txt"
            )?))?
        );
    }
    {
        println!("Echoes of Enigmatus Quest 1 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "echoes-of-enigmatus_01-3.txt"
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
        const TEST_DATA: &str = concat!(
            "A=4 B=4 C=6 X=3 Y=4 Z=5 M=11\n",
            "A=8 B=4 C=7 X=8 Y=4 Z=6 M=12\n",
            "A=2 B=8 C=6 X=2 Y=4 Z=5 M=13\n",
            "A=5 B=9 C=6 X=8 Y=6 Z=8 M=14\n",
            "A=5 B=9 C=7 X=6 Y=6 Z=8 M=15\n",
            "A=8 B=8 C=8 X=6 Y=9 Z=6 M=16\n",
        );

        let expected = 11611972920;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_small_eni() {
        let expected = 34213;
        let actual = small_eni(2, 7, 5);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "A=4 B=4 C=6 X=3 Y=14 Z=15 M=11\n",
            "A=8 B=4 C=7 X=8 Y=14 Z=16 M=12\n",
            "A=2 B=8 C=6 X=2 Y=14 Z=15 M=13\n",
            "A=5 B=9 C=6 X=8 Y=16 Z=18 M=14\n",
            "A=5 B=9 C=7 X=6 Y=16 Z=18 M=15\n",
            "A=8 B=8 C=8 X=6 Y=19 Z=16 M=16\n",
        );
        const TEST_DATA_2: &str = concat!(
            "A=3657 B=3583 C=9716 X=903056852 Y=9283895500 Z=85920867478 M=188\n",
            "A=6061 B=4425 C=5082 X=731145782 Y=1550090416 Z=87586428967 M=107\n",
            "A=7818 B=5395 C=9975 X=122388873 Y=4093041057 Z=58606045432 M=102\n",
            "A=7681 B=9603 C=5681 X=716116871 Y=6421884967 Z=66298999264 M=196\n",
            "A=7334 B=9016 C=8524 X=297284338 Y=1565962337 Z=86750102612 M=145\n",
        );

        let expected = 11051340;
        let actual = part2(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);

        let expected = 1507702060886;
        let actual = part2(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_big_eni() {
        let expected = 19;
        let actual = big_eni(2, 7, 5);
        assert_eq!(expected, actual);

        let expected = 48;
        let actual = big_eni(3, 8, 16);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "A=4 B=4 C=6 X=3000 Y=14000 Z=15000 M=110\n",
            "A=8 B=4 C=7 X=8000 Y=14000 Z=16000 M=120\n",
            "A=2 B=8 C=6 X=2000 Y=14000 Z=15000 M=130\n",
            "A=5 B=9 C=6 X=8000 Y=16000 Z=18000 M=140\n",
            "A=5 B=9 C=7 X=6000 Y=16000 Z=18000 M=150\n",
            "A=8 B=8 C=8 X=6000 Y=19000 Z=16000 M=160\n",
        );
        const TEST_DATA_2: &str = concat!(
            "A=3657 B=3583 C=9716 X=903056852 Y=9283895500 Z=85920867478 M=188\n",
            "A=6061 B=4425 C=5082 X=731145782 Y=1550090416 Z=87586428967 M=107\n",
            "A=7818 B=5395 C=9975 X=122388873 Y=4093041057 Z=58606045432 M=102\n",
            "A=7681 B=9603 C=5681 X=716116871 Y=6421884967 Z=66298999264 M=196\n",
            "A=7334 B=9016 C=8524 X=297284338 Y=1565962337 Z=86750102612 M=145\n",
        );

        let expected = 3279640;
        let actual = part3(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);

        let expected = 7276515438396;
        let actual = part3(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
