use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Columns {
    columns: [Vec<usize>; 4],
}

impl Columns {
    fn read(input: &mut dyn BufRead) -> io::Result<Self> {
        input
            .lines()
            .map(|line| -> io::Result<_> {
                let line = line?;
                let nums = line
                    .split_whitespace()
                    .map(|segment| {
                        segment
                            .parse::<usize>()
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                    })
                    .collect::<io::Result<Vec<_>>>()?;
                Ok([nums[0], nums[1], nums[2], nums[3]])
            })
            .try_fold(Self::default(), |mut acc, row| {
                acc.columns
                    .iter_mut()
                    .zip(row?)
                    .for_each(|(column, value)| column.push(value));
                Ok(acc)
            })
    }

    fn dance_round(&mut self, round_number: usize) -> usize {
        let starting_column = round_number % self.columns.len();
        let ending_column = (round_number + 1) % self.columns.len();
        let value = self.columns[starting_column].remove(0);
        let effective_value = ((value - 1) % (2 * self.columns[ending_column].len())) + 1;
        let index = if effective_value - 1 < self.columns[ending_column].len() {
            effective_value - 1
        } else {
            2 * self.columns[ending_column].len() - effective_value + 1
        };
        self.columns[ending_column].insert(index, value);
        let shout_str = format!(
            "{}{}{}{}",
            self.columns[0][0], self.columns[1][0], self.columns[2][0], self.columns[3][0]
        );
        shout_str.parse().unwrap()
    }

    fn rows(&self) -> impl Iterator<Item = [usize; 4]> + '_ {
        fn zeroes<'a>() -> impl Iterator<Item = &'a usize> {
            std::iter::repeat(&0)
        }

        self.columns[0]
            .iter()
            .chain(zeroes())
            .zip(self.columns[1].iter().chain(zeroes()))
            .zip(self.columns[2].iter().chain(zeroes()))
            .zip(self.columns[3].iter().chain(zeroes()))
            .map(|(((&a, &b), &c), &d)| [a, b, c, d])
            .take_while(|row| row.iter().any(|&value| value != 0))
    }
}

impl Display for Columns {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for row in self.rows() {
            for i in 0..row.len() {
                if row[i] == 0 {
                    write!(f, " ")?;
                } else {
                    write!(f, "{}", row[i])?;
                }
                if i + 1 != row.len() {
                    write!(f, " ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut columns = Columns::read(input)?;
    Ok((0..10).fold(0, |_, i| columns.dance_round(i)))
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut columns = Columns::read(input)?;
    let mut shouted = HashMap::<_, usize>::new();
    for round_number in 0.. {
        let shout = columns.dance_round(round_number);
        let count = shouted.entry(shout).or_default();
        *count += 1;
        if *count == 2024 {
            return Ok((round_number + 1) * shout);
        }
    }
    unreachable!("Ran out of numbers")
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut columns = Columns::read(input)?;
    let mut shouted = HashMap::<_, Vec<_>>::new();
    for round_number in 0.. {
        let shout = columns.dance_round(round_number);
        let history = shouted.entry((shout, round_number % 4)).or_default();
        if history.contains(&columns) {
            return Ok(shouted.into_keys().map(|(shout, _)| shout).max().unwrap());
        } else {
            history.push(columns.clone());
        }
    }
    unreachable!("Ran out of numbers")
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Algorithmia Quest 5 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open("algorithmia_05-1.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 5 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open("algorithmia_05-2.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 5 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open("algorithmia_05-3.txt")?))?
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
        const TEST_DATA: &str = "2 3 4 5\n3 4 5 2\n4 5 2 3\n5 2 3 4\n";
        let expected = 2323;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = "2 3 4 5\n6 7 8 9\n";
        let expected = 50_877_075;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = "2 3 4 5\n6 7 8 9\n";
        let expected = 6584;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
