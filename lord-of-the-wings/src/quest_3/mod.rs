use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    str::FromStr,
};

struct Vote {
    iso_standard: Vec<u64>,
    vote: Vec<u64>,
}

impl From<Vote> for u64 {
    fn from(this: Vote) -> Self {
        this.iso_standard
            .iter()
            .cycle()
            .zip(&this.vote)
            .enumerate()
            .fold(0, |acc, (position, (base, vote))| {
                acc + vote * base.pow(position as _)
            })
    }
}

impl Display for Vote {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ISO-")?;
        for base in self.iso_standard.iter().rev() {
            write!(f, "{base}")?;
        }
        write!(f, " ")?;
        for vote in self.vote.iter().rev() {
            write!(f, "{vote}")?;
        }
        Ok(())
    }
}

impl FromStr for Vote {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (iso, vote) = s.split_once(' ').ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Missing space in {s:?}"),
            )
        })?;
        let bases = iso.strip_prefix("ISO-").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Missing {:?} prefix in {s:?}", "ISO-"),
            )
        })?;
        let iso_standard = bases
            .chars()
            .map(|c| c.to_digit(10).map(|d| d as u64))
            .rev()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Non-numeric ISO standard in {s:?}"),
                )
            })?;
        let vote = vote
            .chars()
            .map(|c| c.to_digit(10).map(|d| d as u64))
            .rev()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Non-numeric vote in {s:?}"),
                )
            })?;
        Ok(Self { iso_standard, vote })
    }
}

struct Iso2Vote {
    iso_standard: Vec<u64>,
    vote: Vec<u64>,
    value: u64,
}

impl Display for Iso2Vote {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ISO2-")?;
        for base in self.iso_standard.iter().rev() {
            write!(f, "{base}")?;
        }
        write!(f, " ")?;
        for vote in self.vote.iter().rev() {
            write!(f, "{vote}")?;
        }
        Ok(())
    }
}

impl From<Iso2Vote> for u64 {
    fn from(this: Iso2Vote) -> Self {
        #[cfg(debug_assertions)]
        eprintln!("The value of \"{this}\" is {}", this.value);
        this.value
    }
}

impl FromStr for Iso2Vote {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        /// `vote1` and `vote2` must contain an equal number of items.
        fn cmp_votes(vote1: &[u64], vote2: &[u64]) -> Ordering {
            if vote1.is_empty() {
                Ordering::Equal
            } else {
                let (last1, vote1) = vote1.split_last().unwrap();
                let (last2, vote2) = vote2.split_last().unwrap();
                last1.cmp(last2).then_with(|| cmp_votes(vote1, vote2))
            }
        }

        let (iso, vote) = s.split_once(' ').ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Missing space in {s:?}"),
            )
        })?;
        let bases = iso.strip_prefix("ISO2-").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Missing {:?} prefix in {s:?}", "ISO-"),
            )
        })?;
        let iso_standard = bases
            .chars()
            .map(|c| c.to_digit(10).map(|d| d as u64))
            .rev()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Non-numeric ISO standard in {s:?}"),
                )
            })?;
        let vote = vote
            .chars()
            .map(|c| c.to_digit(10).map(|d| d as u64))
            .rev()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Non-numeric vote in {s:?}"),
                )
            })?;
        let mut value: u64 = Vote {
            iso_standard: iso_standard.clone(),
            vote: vote.clone(),
        }
        .into();
        let mut low_vote_start = vec![0; vote.len()];
        while value
            > (Vote {
                iso_standard: iso_standard.clone(),
                vote: low_vote_start.clone(),
            }
            .into())
        {
            match low_vote_start.iter_mut().find(|entry| **entry != 9) {
                None => {
                    // SAFETY: This branch can never be taken because that would require that every
                    // entry in `low_vote_start` is 9, which would guarantee that the value of
                    // `low_vote_start` is not less than the value of `vote`.
                    unsafe { std::hint::unreachable_unchecked() }
                }
                Some(entry) => *entry = 9,
            }
        }
        if let Some(entry) = low_vote_start.iter_mut().rev().find(|entry| **entry != 0) {
            *entry = 0;
        }
        let mut low_vote = low_vote_start.clone();
        while matches!(cmp_votes(&low_vote, &vote), Ordering::Less) {
            if value
                == (Vote {
                    iso_standard: iso_standard.clone(),
                    vote: low_vote.clone(),
                }
                .into())
            {
                value = 0;
                break;
            }
            for entry in &mut low_vote {
                if *entry == 9 {
                    *entry = 0;
                } else {
                    *entry += 1;
                    break;
                }
            }
        }
        Ok(Self {
            iso_standard,
            vote,
            value,
        })
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<u64> {
    input
        .lines()
        .map(|line| {
            line.and_then(|line| line.parse::<Vote>())
                .map(|vote| -> u64 { vote.into() })
        })
        .sum()
}

fn part2(input: &mut dyn BufRead) -> io::Result<u64> {
    part1(input)
}

fn part3(input: &mut dyn BufRead) -> io::Result<u64> {
    input
        .lines()
        .map(|line| {
            line.and_then(|line| line.parse::<Iso2Vote>())
                .map(|vote| -> u64 { vote.into() })
        })
        .sum()
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Lord of the wings Quest 3 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "lord-of-the-wings_03-1.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 3 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "lord-of-the-wings_03-2.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 3 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "lord-of-the-wings_03-3.txt"
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
        const TEST_DATA: &str = concat!("ISO-3210 7654\n", "ISO-8742 2468\n", "ISO-1111 1111\n");
        let expected = 1478;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "ISO-3210 7654\n",
            "ISO-8742 2468\n",
            "ISO-1 1111\n",
            "ISO-13 777\n",
            "ISO-34 140856\n",
            "ISO-789 2465112\n",
            "ISO-2341 945\n",
        );
        let expected = 1_161_463;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "ISO2-3 23\n",
            "ISO2-61 149\n",
            "ISO2-978 99\n",
            "ISO2-5887 5923\n",
            "ISO2-76988 123749\n",
            "ISO2-435657 929977\n",
            "ISO2-3949 870737\n",
        );
        let expected = 50_987;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
