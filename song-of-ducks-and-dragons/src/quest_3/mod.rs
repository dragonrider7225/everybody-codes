use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut crates = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Couldn't read crate sizes"))??
        .split(',')
        .map(|s| {
            s.parse::<usize>().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Couldn't parse {s:?} as crate size"),
                )
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    crates.sort();
    crates.dedup();
    Ok(crates.into_iter().sum::<usize>())
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut crates = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Couldn't read crate sizes"))??
        .split(',')
        .map(|s| {
            s.parse::<usize>().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Couldn't parse {s:?} as crate size"),
                )
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    crates.sort();
    crates.dedup();
    Ok(crates.into_iter().take(20).sum::<usize>())
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut crates = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Couldn't read crate sizes"))??
        .split(',')
        .map(|s| {
            s.parse::<usize>().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Couldn't parse {s:?} as crate size"),
                )
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    crates.sort();
    let mut max_freq = 0;
    let mut curr_start = 0;
    for i in 0..crates.len() {
        if crates[i] != crates[curr_start] {
            let curr_freq = i - curr_start;
            max_freq = max_freq.max(curr_freq);
            curr_start = i;
        }
    }
    Ok(max_freq.max(crates.len() - curr_start))
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 3 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_03-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 3 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_03-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 3 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_03-3.txt"
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
        const TEST_DATA: &str = "10,5,1,10,3,8,5,2,2\n";
        let expected = 29;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = "4,51,13,64,57,51,82,57,16,88,89,48,32,49,49,2,84,65,49,43,9,13,2,3,75,72,63,48,61,14,40,77\n";
        let expected = 781;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = "4,51,13,64,57,51,82,57,16,88,89,48,32,49,49,2,84,65,49,43,9,13,2,3,75,72,63,48,61,14,40,77\n";
        let expected = 3;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
