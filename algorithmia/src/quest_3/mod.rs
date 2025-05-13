use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

fn read_pond_map(input: &mut dyn BufRead) -> io::Result<Vec<(usize, usize)>> {
    input
        .lines()
        .enumerate()
        .map(|(y, line)| {
            line?
                .bytes()
                .enumerate()
                .filter_map(move |(x, b)| match b {
                    b'.' => None,
                    b'#' => Some(Ok((x + 1, y + 1))),
                    _ => Some(Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Invalid input character {:?}", b as char),
                    ))),
                })
                .collect::<io::Result<Vec<_>>>()
        })
        .try_fold(vec![], |mut acc, line| {
            acc.extend(line?);
            Ok(acc)
        })
}

fn safe_digging<F, I>(mut last_group: Vec<(usize, usize)>, neighbors: F) -> usize
where
    F: Fn((usize, usize)) -> I,
    I: Iterator<Item = (usize, usize)>,
{
    let mut next_group = vec![];
    let mut total = 0;
    while !last_group.is_empty() {
        total += last_group.len();
        for &(x, y) in &last_group {
            if neighbors((x, y)).all(|neighbor| last_group.contains(&neighbor)) {
                next_group.push((x, y));
            }
        }
        std::mem::swap(&mut last_group, &mut next_group);
        next_group.clear();
    }
    total
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    fn neighbors((x, y): (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
        [(x, y - 1), (x - 1, y), (x + 1, y), (x, y + 1)].into_iter()
    }

    Ok(safe_digging(read_pond_map(input)?, neighbors))
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    part1(input)
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    fn neighbors((x, y): (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
        #[rustfmt::skip]
        let neighbors = [
            (x - 1, y - 1), (x, y - 1), (x + 1, y - 1),
            (x - 1, y    ),             (x + 1, y    ),
            (x - 1, y + 1), (x, y + 1), (x + 1, y + 1),
        ];
        neighbors.into_iter()
    }

    Ok(safe_digging(read_pond_map(input)?, neighbors))
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Algorithmia Quest 3 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open("algorithmia_03-1.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 3 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open("algorithmia_03-2.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 3 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open("algorithmia_03-3.txt")?))?
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
        let expected = 35;
        let actual = part1(&mut Cursor::new(concat!(
            "..........\n",
            "..###.##..\n",
            "...####...\n",
            "..######..\n",
            "..######..\n",
            "...####...\n",
            "..........\n",
        )))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        let expected = 29;
        let actual = part3(&mut Cursor::new(concat!(
            "..........\n",
            "..###.##..\n",
            "...####...\n",
            "..######..\n",
            "..######..\n",
            "...####...\n",
            "..........\n",
        )))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
