use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

fn read_heights(input: &mut dyn BufRead) -> io::Result<Vec<u32>> {
    input
        .lines()
        .map(|line| {
            let line = line?;
            line.parse::<u32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        })
        .collect()
}

fn part1(input: &mut dyn BufRead) -> io::Result<u32> {
    let heights = read_heights(input)?;
    let min_height = heights.iter().copied().min().unwrap_or_default();
    Ok(heights.into_iter().map(|height| height - min_height).sum())
}

fn part2(input: &mut dyn BufRead) -> io::Result<u32> {
    part1(input)
}

fn part3(input: &mut dyn BufRead) -> io::Result<u32> {
    let mut heights = read_heights(input)?;
    heights.sort();
    let target_height = heights[heights.len() / 2];
    Ok(heights
        .into_iter()
        .map(|height| target_height.abs_diff(height))
        .sum())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Algorithmia Quest 4 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open("algorithmia_04-1.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 4 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open("algorithmia_04-2.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 4 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open("algorithmia_04-3.txt")?))?
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
        const TEST_DATA: &str = "3\n4\n7\n8\n";
        let expected = 10;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = "2\n4\n5\n6\n8\n";
        let expected = 8;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
