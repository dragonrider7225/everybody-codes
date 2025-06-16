use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    ops::Add,
};

use dependencies::{
    nom::{bytes::complete as bytes, character::complete as character, IResult, Parser},
    nom_supreme::ParserExt,
    num::BigInt,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Point {
    x: u32,
    y: u32,
}

impl Point {
    fn nom_parse(s: &str) -> IResult<&str, Self> {
        bytes::tag("x=")
            .precedes(character::u32)
            .and(bytes::tag(" y=").precedes(character::u32))
            .map(|(x, y)| Self { x, y })
            .parse(s)
    }

    fn step(&self, duration: u32) -> Self {
        let path_len = self.path_len();
        let time = duration % path_len;
        let final_x = ((self.x + time - 1) % path_len) + 1;
        let effective_y = if self.y < time + 1 {
            self.y + path_len
        } else {
            self.y
        };
        let final_y = (effective_y - time - 1) % path_len + 1;
        Self {
            x: final_x,
            y: final_y,
        }
    }

    fn time_to_y1(&self) -> u32 {
        self.y - 1
    }

    fn path_len(&self) -> u32 {
        self.x + self.y - 1
    }
}

impl Add<u32> for Point {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        self.step(rhs)
    }
}

fn starting_positions(input: &mut dyn BufRead) -> io::Result<Vec<Point>> {
    input
        .lines()
        .map(|line| {
            let line = line?;
            Point::nom_parse(&line)
                .map(|(_, point)| point)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        })
        .collect()
}

fn part1(input: &mut dyn BufRead) -> io::Result<u32> {
    Ok(starting_positions(input)?
        .into_iter()
        .map(|starting_position| {
            let final_position = starting_position.step(100);
            final_position.x + (100 * final_position.y)
        })
        .sum())
}

fn part2(input: &mut dyn BufRead) -> io::Result<BigInt> {
    let starting_positions = starting_positions(input)?;
    let times_and_lens = starting_positions
        .into_iter()
        .map(|position| (position.time_to_y1().into(), position.path_len().into()))
        .collect::<Vec<_>>();
    dependencies::math::chinese_remainder_theorem(times_and_lens).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "The snails never line up in row one",
        )
    })
}

fn part3(input: &mut dyn BufRead) -> io::Result<BigInt> {
    part2(input)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Echoes of Enigmatus Quest 3 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "echoes-of-enigmatus_03-1.txt"
            )?))?
        );
    }
    {
        println!("Echoes of Enigmatus Quest 3 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "echoes-of-enigmatus_03-2.txt"
            )?))?
        );
    }
    {
        println!("Echoes of Enigmatus Quest 3 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "echoes-of-enigmatus_03-3.txt"
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
        const TEST_DATA: &str = "x=1 y=2\nx=2 y=3\nx=3 y=4\nx=4 y=4\n";

        let expected = 1310;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "x=12 y=2\n",
            "x=8 y=4\n",
            "x=7 y=1\n",
            "x=1 y=5\n",
            "x=1 y=3\n",
        );
        const TEST_DATA_2: &str = concat!(
            "x=3 y=1\n",
            "x=3 y=9\n",
            "x=1 y=5\n",
            "x=4 y=10\n",
            "x=5 y=3\n",
        );

        let expected = BigInt::from(14);
        let actual = part2(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        let expected = BigInt::from(13659);
        let actual = part2(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
