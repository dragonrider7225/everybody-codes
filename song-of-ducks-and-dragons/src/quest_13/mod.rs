use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

fn part1(input: &mut dyn BufRead) -> io::Result<u32> {
    const NUM_MOVES: usize = 2025;
    let nums = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<u32>().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("{e:?}: {line:?} is not a number"),
                    )
                })
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    let mut dial_right = vec![1];
    let mut dial_left = vec![];
    let mut dial = nums.iter().copied();
    while let Some(right) = dial.next() {
        dial_right.push(right);
        if let Some(left) = dial.next() {
            dial_left.push(left);
        }
    }
    let dial_size = 1 + nums.len();
    let position = NUM_MOVES % dial_size;
    Ok(dial_right
        .into_iter()
        .chain(dial_left.into_iter().rev())
        .nth(position)
        .expect("dial_right.len() + dial_left.len() == dial_size"))
}

fn part2(input: &mut dyn BufRead, num_moves: usize) -> io::Result<u32> {
    let nums = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                let (start, end) = line.split_once('-').ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Number range must contain exactly '-'",
                    )
                })?;
                let start = start.parse::<u32>().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("{e:?}: {start:?} is not a number"),
                    )
                })?;
                let end = end.parse::<u32>().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("{e:?}: {end:?} is not a number"),
                    )
                })?;
                Ok(start..=end)
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    let mut dial_right = vec![1..=1];
    let mut dial_left = vec![];
    let mut dial = nums.into_iter();
    let mut dial_size = 1;
    while let Some(right) = dial.next() {
        dial_size += right.clone().count();
        dial_right.push(right);
        if let Some(left) = dial.next() {
            dial_size += left.clone().count();
            dial_left.push(left);
        }
    }
    let position = num_moves % dial_size;
    Ok(dial_right
        .into_iter()
        .flatten()
        .chain(dial_left.into_iter().flatten().rev())
        .nth(position)
        .expect("dial.map(|range| range.len()).sum() + 1 == dial_size"))
}

fn part3(input: &mut dyn BufRead) -> io::Result<u32> {
    part2(input, 202_520_252_025)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 13 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_13-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 13 Part 2");
        println!(
            "{}",
            part2(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_13-2.txt")?),
                20_252_025,
            )?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 13 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_13-3.txt"
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
        const TEST_DATA: &str = "72\n58\n47\n61\n67\n";
        let expected = 67;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA_1: &str = "10-15\n12-13\n20-21\n19-23\n30-37\n";
        let expected = 30;
        let actual = part2(&mut Cursor::new(TEST_DATA_1), 20_252_025)?;
        assert_eq!(expected, actual);
        const TEST_DATA_2: &str = "10-15\n12-13\n20-21\n19-31\n";
        let expected = 31;
        let actual = part2(&mut Cursor::new(TEST_DATA_2), 20_252_025)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
