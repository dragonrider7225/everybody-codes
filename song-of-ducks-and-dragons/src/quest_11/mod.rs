use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

/// Returns whether phase 1 has been completed.
fn phase1(columns: &mut [usize]) -> bool {
    let mut phase_done = true;
    for idx in 1..columns.len() {
        if columns[idx] < columns[idx - 1] {
            columns[idx - 1] -= 1;
            columns[idx] += 1;
            phase_done = false;
        }
    }
    phase_done
}

/// Returns whether phase 2 has been completed.
fn phase2(columns: &mut [usize]) -> bool {
    let mut phase_done = true;
    for idx in 1..columns.len() {
        if columns[idx] > columns[idx - 1] {
            columns[idx - 1] += 1;
            columns[idx] -= 1;
            phase_done = false;
        }
    }
    phase_done
}

fn balance_flock(columns: &mut [usize], max_rounds: impl Into<Option<usize>>) -> usize {
    let max_rounds = max_rounds.into().unwrap_or(usize::MAX);
    let mut phase = 1;
    let mut round = 0;
    while round < max_rounds {
        match phase {
            1 => {
                if phase1(columns) {
                    phase = 2;
                    continue;
                }
            }
            2 => {
                if phase2(columns) {
                    return round;
                }
            }
            _ => unreachable!("Unknown phase {phase:?}"),
        }
        round += 1;
    }
    max_rounds
}

fn read_flock(input: &mut dyn BufRead) -> io::Result<Vec<usize>> {
    input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<usize>().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("{e:?}: Cannot parse {line:?} as usize"),
                    )
                })
            })
        })
        .collect()
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    const MAX_ROUNDS: usize = 10;
    let mut columns = read_flock(input)?;
    let num_rounds = balance_flock(&mut columns, MAX_ROUNDS);
    if num_rounds < MAX_ROUNDS {
        eprintln!("Balanced flock in {num_rounds} rounds (less than {MAX_ROUNDS})");
    }
    Ok(columns
        .into_iter()
        .enumerate()
        .map(|(idx, len)| (idx + 1) * len)
        .sum())
}

/// Calculate the number of rounds a flock that has completed phase 1 will need to complete phase
/// 2.
fn time_to_balance_flock(columns: &[usize]) -> usize {
    let num_columns = columns.len();
    let num_birds = columns.iter().copied().sum::<usize>();
    let target = num_birds / num_columns;
    let remainder = num_birds % num_columns;
    assert_eq!(0, remainder, "Cannot balance flock with {target}*{num_columns}+{remainder} birds in {num_columns} columns");
    columns
        .iter()
        .copied()
        .take_while(|&column| column < target)
        .map(|column| target - column)
        .sum::<usize>()
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut columns = read_flock(input)?;
    let num_rounds = (0..).find(|_| phase1(&mut columns)).unwrap();
    Ok(num_rounds + time_to_balance_flock(&columns))
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    part2(input)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 11 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_11-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 11 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_11-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 11 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_11-3.txt"
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
        const TEST_DATA: &str = "9\n1\n1\n4\n9\n6\n";
        let expected = 109;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA_1: &str = "9\n1\n1\n4\n9\n6\n";
        let expected = 11;
        let actual = part2(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        const TEST_DATA_2: &str = "805\n706\n179\n48\n158\n150\n232\n885\n598\n524\n423\n";
        let expected = 1579;
        let actual = part2(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
