use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

/// The number of times the last gear rotates for each full rotation of the first gear.
fn gear_ratio(first_gear: f64, last_gear: f64) -> f64 {
    first_gear / last_gear
}

fn part1(input: &mut dyn BufRead) -> io::Result<u32> {
    let gears = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<f64>().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Couldn't parse {line:?} as float"),
                    )
                })
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    if gears.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Couldn't read any gears",
        ));
    }
    let ratio = gear_ratio(*gears.first().unwrap(), *gears.last().unwrap());
    Ok((ratio * 2025.).floor() as _)
}

fn part2(input: &mut dyn BufRead) -> io::Result<u64> {
    let gears = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<f64>().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Couldn't parse {line:?} as float"),
                    )
                })
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    if gears.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Couldn't read any gears",
        ));
    }
    let ratio = gear_ratio(*gears.first().unwrap(), *gears.last().unwrap());
    Ok((10_000_000_000_000. / ratio).ceil() as _)
}

fn part3(input: &mut dyn BufRead) -> io::Result<u64> {
    let first_gear = input
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Couldn't read gears"))??
        .parse::<f64>()
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{e:?}: Couldn't parse gear"),
            )
        })?;
    let gears = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                match line.split_once('|') {
                    None => line.parse::<f64>().map(|back| (back, 0.)),
                    Some((back, forward)) => back
                        .parse::<f64>()
                        .and_then(|back| Ok((back, forward.parse::<f64>()?))),
                }
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("{e:?}: Couldn't parse {line:?} as gear"),
                    )
                })
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    let ratio = gear_ratio(first_gear, gears[0].0)
        * gears
            .windows(2)
            .map(|window| gear_ratio(window[0].1, window[1].0))
            .product::<f64>();
    Ok((ratio * 100.).floor() as _)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 4 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_04-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 4 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_04-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 4 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_04-3.txt"
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
        const TEST_DATA_1: &str = "128\n64\n32\n16\n8\n";
        let expected = 32400;
        let actual = part1(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        const TEST_DATA_2: &str = "102\n75\n50\n35\n13\n";
        let expected = 15888;
        let actual = part1(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA_1: &str = "128\n64\n32\n16\n8\n";
        let expected = 625_000_000_000;
        let actual = part2(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        const TEST_DATA_2: &str = "102\n75\n50\n35\n13\n";
        let expected = 1_274_509_803_922;
        let actual = part2(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA_1: &str = "5\n5|10\n10|20\n5\n";
        let expected = 400;
        let actual = part3(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        const TEST_DATA_2: &str = "5\n7|21\n18|36\n27|27\n10|50\n10|50\n11\n";
        let expected = 6818;
        let actual = part3(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
