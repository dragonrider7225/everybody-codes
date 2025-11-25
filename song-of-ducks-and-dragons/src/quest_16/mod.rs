use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    iter,
    time::Instant,
};

fn run_spell(spell: impl IntoIterator<Item = usize>, wall_width: usize) -> usize {
    spell.into_iter().map(|n| wall_width / n).sum()
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let instructions = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.split(',')
                    .map(|n| {
                        n.parse::<usize>().map_err(|e| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("{e:?}: {n:?} is not a number"),
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
    Ok(run_spell(instructions, 90))
}

fn find_spell(mut wall_start: Vec<usize>) -> impl IntoIterator<Item = usize> {
    wall_start.insert(0, 0);
    let mut spell = vec![];
    for i in 1..wall_start.len() {
        let count = wall_start[i];
        if count == 0 {
            continue;
        }
        let wall_start_len = wall_start.len();
        for j in (1..).map(|j| j * i).take_while(|&j| j < wall_start_len) {
            wall_start[j] -= count;
        }
        spell.extend(iter::repeat_n(i, count));
    }
    spell
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let wall_start = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing notes"))??
        .split(',')
        .map(|n| {
            n.parse::<usize>().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{e:?}: {n:?} is not a number"),
                )
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    Ok(find_spell(wall_start).into_iter().product())
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let wall_start = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing notes"))??
        .split(',')
        .map(|n| {
            n.parse::<usize>().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{e:?}: {n:?} is not a number"),
                )
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    let start = Instant::now();
    let spell = find_spell(wall_start).into_iter().collect::<Vec<_>>();
    const TOTAL_BRICKS: usize = 202_520_252_025_000;
    let mut approx_len = (TOTAL_BRICKS as f64
        / (spell.iter().map(|&n| 1. / (n as f64)).sum::<f64>()))
    .round() as usize;
    while run_spell(spell.iter().copied(), approx_len) < TOTAL_BRICKS {
        approx_len += 1;
    }
    while run_spell(spell.iter().copied(), approx_len) > TOTAL_BRICKS {
        approx_len -= 1;
    }
    eprintln!("Calculation took {:?}", Instant::now() - start);
    Ok(approx_len)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 16 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_16-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 16 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_16-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 16 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_16-3.txt"
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
        const TEST_DATA: &str = "1,2,3,5,9\n";
        let expected = 193;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = "1,2,2,2,2,3,1,2,3,3,1,3,1,2,3,2,1,4,1,3,2,2,1,3,2,2\n";
        let expected = 270;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = "1,2,2,2,2,3,1,2,3,3,1,3,1,2,3,2,1,4,1,3,2,2,1,3,2,2\n";
        let expected = 94_439_495_762_954;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
