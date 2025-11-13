use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    ops::{Bound, RangeBounds},
    time::Instant,
};

fn part1(input: &mut dyn BufRead, num_nails: usize) -> io::Result<usize> {
    let steps = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.split(',')
                    .map(|n| {
                        n.parse::<usize>().map_err(|e| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("{e:?}: Can't parse {n:?} as usize"),
                            )
                        })
                    })
                    .collect::<io::Result<Vec<_>>>()
            })
        })
        .try_fold(vec![], |mut acc, line| -> io::Result<_> {
            acc.extend(line?);
            Ok(acc)
        })?;
    let diff = match num_nails % 2 {
        0 => num_nails / 2,
        _ => return Ok(0),
    };
    Ok(steps
        .windows(2)
        .filter(|window| window[0].abs_diff(window[1]) == diff)
        .count())
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let steps = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.split(',')
                    .map(|n| {
                        n.parse::<usize>().map_err(|e| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("{e:?}: Can't parse {n:?} as usize"),
                            )
                        })
                    })
                    .collect::<io::Result<Vec<_>>>()
            })
        })
        .try_fold(vec![], |mut acc, line| -> io::Result<_> {
            acc.extend(line?);
            Ok(acc)
        })?;
    Ok(steps
        .windows(2)
        .fold(
            (vec![], 0),
            |(mut history, num_knots): (Vec<(Bound<_>, _)>, _), window| {
                let low = window[0].min(window[1]);
                let high = window[0].max(window[1]);
                let new_knots = history
                    .iter()
                    .filter(|&prev| match prev {
                        (Bound::Excluded(low), Bound::Excluded(high)) => {
                            !(window.contains(low) || window.contains(high))
                        }
                        _ => unreachable!("Only (Excluded, Excluded) Bounds are stored in history"),
                    })
                    .filter(|&prev| prev.contains(&low) ^ prev.contains(&high))
                    .count();
                history.push((
                    std::ops::Bound::Excluded(low),
                    std::ops::Bound::Excluded(high),
                ));
                (history, num_knots + new_knots)
            },
        )
        .1)
}

fn part3(input: &mut dyn BufRead, num_nails: usize) -> io::Result<usize> {
    let steps = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.split(',')
                    .map(|n| {
                        n.parse::<usize>().map_err(|e| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("{e:?}: Can't parse {n:?} as usize"),
                            )
                        })
                    })
                    .collect::<io::Result<Vec<_>>>()
            })
        })
        .try_fold(vec![], |mut acc, line| -> io::Result<_> {
            acc.extend(line?);
            Ok(acc)
        })?;
    let strings = steps
        .windows(2)
        .map(|window| [window[0].min(window[1]), window[0].max(window[1])])
        .collect::<Vec<_>>();
    let start = Instant::now();
    let max_cut = (1..=num_nails)
        .flat_map(|start| ((start + 1)..=num_nails).map(move |end| (start, end)))
        .fold(0, |max_cut, (cut_low, cut_high)| {
            let current_cut = strings
                .iter()
                .filter(|&string| {
                    string.contains(&cut_low) && string.contains(&cut_high)
                        || !(string.contains(&cut_low) ^ string.contains(&cut_high))
                            && ((string[0]..string[1]).contains(&cut_low)
                                ^ (string[0]..string[1]).contains(&cut_high))
                })
                .count();
            max_cut.max(current_cut)
        });
    eprintln!("Computation took {:?}", Instant::now() - start);
    Ok(max_cut)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 8 Part 1");
        println!(
            "{}",
            part1(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_08-1.txt")?),
                32
            )?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 8 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_08-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 8 Part 3");
        println!(
            "{}",
            part3(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_08-3.txt")?),
                256
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
        const TEST_DATA: &str = "1,5,2,6,8,4,1,7,3\n";
        let expected = 4;
        let actual = part1(&mut Cursor::new(TEST_DATA), 8)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = "1,5,2,6,8,4,1,7,3,5,7,8,2\n";
        let expected = 21;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = "1,5,2,6,8,4,1,7,3,6\n";
        let expected = 7;
        let actual = part3(&mut Cursor::new(TEST_DATA), 8)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
