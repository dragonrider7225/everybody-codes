use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    iter,
    time::Instant,
};

fn part1(input: &mut dyn BufRead) -> io::Result<u32> {
    let input = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing notes"))??
        .into_bytes();
    let (_, num_pairs) = input
        .into_iter()
        .fold((0, 0), |(num_mentors, num_pairs), b| match b {
            b'A' => (num_mentors + 1, num_pairs),
            b'a' => (num_mentors, num_pairs + num_mentors),
            _ => (num_mentors, num_pairs),
        });
    Ok(num_pairs)
}

fn part2(input: &mut dyn BufRead) -> io::Result<u32> {
    let input = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing notes"))??
        .into_bytes();
    let (_, num_pairs) = input.into_iter().fold(
        (HashMap::<_, u32>::new(), HashMap::<_, u32>::new()),
        |(mut num_mentors, mut num_pairs), b| {
            match b {
                mentor if mentor.is_ascii_uppercase() => {
                    *num_mentors.entry(mentor).or_default() += 1
                }
                novice if novice.is_ascii_lowercase() => {
                    num_mentors
                        .get(&novice.to_ascii_uppercase())
                        .inspect(|&num_mentors| {
                            *num_pairs.entry(novice).or_default() += num_mentors
                        });
                }
                _ => {
                    eprintln!("Unexpected character {:?} in notes", b as char)
                }
            }
            (num_mentors, num_pairs)
        },
    );
    Ok(num_pairs.values().sum())
}

fn part3(input: &mut dyn BufRead, repeats: usize, range: usize) -> io::Result<usize> {
    let input = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing notes"))??
        .into_bytes();
    let start = Instant::now();
    let (_, _, num_pairs) = iter::repeat_n(input, repeats).flatten().enumerate().fold(
        (HashMap::<_, Vec<_>>::new(), HashMap::<_, Vec<_>>::new(), 0),
        |(mut mentors, mut novices, mut num_pairs), (idx, b)| {
            match b {
                mentor if mentor.is_ascii_uppercase() => {
                    mentors.entry(mentor).or_default().push(idx);
                    novices
                        .get_mut(&mentor.to_ascii_lowercase())
                        .map(|novices| {
                            let num_novices = novices
                                .iter()
                                .rev()
                                .take_while(|&&novice_idx| novice_idx >= idx.saturating_sub(range))
                                .count();
                            if num_novices < novices.len() / 2 {
                                novices.drain(..(novices.len() - num_novices));
                            }
                            num_novices
                        })
                        .inspect(|&num_novices| num_pairs += num_novices);
                }
                novice if novice.is_ascii_lowercase() => {
                    novices.entry(novice).or_default().push(idx);
                    mentors
                        .get_mut(&novice.to_ascii_uppercase())
                        .map(|mentors| {
                            let num_mentors = mentors
                                .iter()
                                .rev()
                                .take_while(|&&mentor_idx| mentor_idx >= idx.saturating_sub(range))
                                .count();
                            if num_mentors < mentors.len() / 2 {
                                mentors.drain(..(mentors.len() - num_mentors));
                            }
                            num_mentors
                        })
                        .inspect(|&num_mentors| num_pairs += num_mentors);
                }
                _ => {
                    eprintln!("Unexpected character {:?} in notes", b as char);
                }
            }
            (mentors, novices, num_pairs)
        },
    );
    let time_elapsed = Instant::now() - start;
    eprintln!("Calculation took {time_elapsed:?}");
    Ok(num_pairs)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 6 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_06-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 6 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_06-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 6 Part 3");
        println!(
            "{}",
            part3(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_06-3.txt")?),
                1000,
                1000
            )?
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
        const TEST_DATA: &str = "ABabACacBCbca\n";
        let expected = 5;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = "ABabACacBCbca\n";
        let expected = 11;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = "AABCBABCABCabcabcABCCBAACBCa\n";
        let expected = 34;
        let actual = part3(&mut Cursor::new(TEST_DATA), 1, 10)?;
        assert_eq!(expected, actual);
        let expected = 72;
        let actual = part3(&mut Cursor::new(TEST_DATA), 2, 10)?;
        assert_eq!(expected, actual);
        let expected = 3442321;
        let actual = part3(&mut Cursor::new(TEST_DATA), 1000, 1000)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
