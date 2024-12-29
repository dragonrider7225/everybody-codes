use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    iter,
};

fn part1(input: &mut dyn BufRead) -> io::Result<u32> {
    const MISSING_LINE_ERROR: &str = "Input must contain three lines";
    const INVALID_WORDS_LINE_PREFIX_ERROR: &str = r#"First line must begin with "WORDS:""#;

    let mut lines = input.lines();
    let mut get_line = || {
        lines
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, MISSING_LINE_ERROR))
            .and_then(|o| o)
    };
    let first_line = get_line()?;
    let words = {
        let (empty, words) = first_line.split_once("WORDS:").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, INVALID_WORDS_LINE_PREFIX_ERROR)
        })?;
        if !empty.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                INVALID_WORDS_LINE_PREFIX_ERROR,
            ));
        }
        words
            .split(",")
            .map(|word| word.as_bytes())
            .collect::<Vec<_>>()
    };
    if !get_line()?.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Second line of input must be blank",
        ));
    }
    let inscription = get_line()?;
    let inscription = inscription.as_bytes();
    let mut num_matches = 0;
    for word in words {
        for position in 0..=(inscription.len() - word.len()) {
            if inscription[position..].starts_with(word) {
                num_matches += 1;
            }
        }
    }
    Ok(num_matches)
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    const MISSING_LINE_ERROR: &str = "Input must contain at least three lines";
    const INVALID_WORDS_LINE_PREFIX_ERROR: &str = r#"First line must begin with "WORDS:""#;

    let mut lines = input.lines();
    let mut get_line = || {
        lines
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, MISSING_LINE_ERROR))
            .and_then(|o| o)
    };
    let first_line = get_line()?;
    let words = {
        let (empty, words) = first_line.split_once("WORDS:").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, INVALID_WORDS_LINE_PREFIX_ERROR)
        })?;
        if !empty.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                INVALID_WORDS_LINE_PREFIX_ERROR,
            ));
        }
        words
            .split(",")
            .map(|word| word.as_bytes())
            .collect::<Vec<_>>()
    };
    if !get_line()?.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Second line of input must be blank",
        ));
    }
    let inscriptions = input.lines().collect::<io::Result<Vec<_>>>()?;
    let inscriptions = inscriptions
        .iter()
        .map(|inscription| inscription.as_bytes())
        .collect::<Vec<_>>();
    let mut num_matches = 0;
    for inscription in inscriptions {
        let mut found = vec![false; inscription.len()];
        for &word in &words {
            for position in 0..=(inscription.len() - word.len()) {
                if inscription[position..].starts_with(word) {
                    found[position..(position + word.len())].fill(true);
                }
                let reverse_range =
                    (inscription.len() - position - word.len())..(inscription.len() - position);
                if inscription[reverse_range.clone()]
                    .iter()
                    .rev()
                    .zip(word)
                    .all(|(i, w)| i == w)
                {
                    found[reverse_range].fill(true);
                }
            }
        }
        num_matches += found.into_iter().filter(|&b| b).count();
    }
    Ok(num_matches)
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    const MISSING_LINE_ERROR: &str = "Input must contain at least three lines";
    const INVALID_WORDS_LINE_PREFIX_ERROR: &str = r#"First line must begin with "WORDS:""#;

    fn test_range<I>(range: I, found: &mut [Vec<bool>], word: &[u8], rows: &[&[u8]])
    where
        I: Iterator<Item = (usize, usize)> + Clone,
    {
        if range
            .clone()
            .zip(word)
            .all(|((col, row), &b)| rows[row][col] == b)
        {
            range.for_each(|(col, row)| found[row][col] = true);
        }
    }

    let mut lines = input.lines();
    let mut get_line = || {
        lines
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, MISSING_LINE_ERROR))
            .and_then(|o| o)
    };
    let first_line = get_line()?;
    let words = {
        let (empty, words) = first_line.split_once("WORDS:").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, INVALID_WORDS_LINE_PREFIX_ERROR)
        })?;
        if !empty.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                INVALID_WORDS_LINE_PREFIX_ERROR,
            ));
        }
        words
            .split(",")
            .map(|word| word.as_bytes())
            .collect::<Vec<_>>()
    };
    if !get_line()?.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Second line of input must be blank",
        ));
    }
    let rows = input.lines().collect::<io::Result<Vec<_>>>()?;
    let rows = rows
        .iter()
        .map(|inscription| inscription.as_bytes())
        .collect::<Vec<_>>();
    let mut found = iter::repeat_n(vec![false; rows[0].len()], rows.len()).collect::<Vec<_>>();
    for word in words {
        let rows = &rows;
        let rightward = |(column, row)| {
            (0..word.len())
                .map(move |delta_column| (column + delta_column) % rows[0].len())
                .map(move |column| (column, row))
        };
        let leftward = |(column, row)| {
            (0..word.len())
                .map(move |delta_column| (column + rows[0].len() - delta_column) % rows[0].len())
                .map(move |column| (column, row))
        };
        let upward = |(column, row)| {
            Some(0..word.len())
                .filter(|_| word.len() <= row + 1)
                .into_iter()
                .flatten()
                .map(move |delta_row| row - delta_row)
                .map(move |row| (column, row))
        };
        let downward = |(column, row)| {
            Some(0..word.len())
                .filter(|_| row + word.len() <= rows.len())
                .into_iter()
                .flatten()
                .map(move |delta_row| (row + delta_row) % rows.len())
                .map(move |row| (column, row))
        };
        for row in 0..rows.len() {
            for col in 0..rows[row].len() {
                test_range(rightward((col, row)), &mut found, word, rows);
                test_range(leftward((col, row)), &mut found, word, rows);
                test_range(upward((col, row)), &mut found, word, rows);
                test_range(downward((col, row)), &mut found, word, rows);
            }
        }
    }
    Ok(found
        .into_iter()
        .flat_map(|row| row.into_iter().filter(|&b| b))
        .count())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Algorithmia Quest 2 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open("algorithmia_02-1.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 2 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open("algorithmia_02-2.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 2 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open("algorithmia_02-3.txt")?))?
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use io::Cursor;

    use super::*;

    #[test]
    fn test_part1() -> io::Result<()> {
        let expected = 4;
        let actual = part1(&mut Cursor::new(concat!(
            "WORDS:THE,OWE,MES,ROD,HER\n",
            "\n",
            "AWAKEN THE POWER ADORNED WITH THE FLAMES BRIGHT IRE\n",
        )))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        let expected = 42;
        let actual = part2(&mut Cursor::new(concat!(
            "WORDS:THE,OWE,MES,ROD,HER,QAQ\n",
            "\n",
            "AWAKEN THE POWE ADORNED WITH THE FLAMES BRIGHT IRE\n",
            "THE FLAME SHIELDED THE HEART OF THE KINGS\n",
            "POWE PO WER P OWE R\n",
            "THERE IS THE END\n",
            "QAQAQ\n",
        )))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        let expected = 10;
        let actual = part3(&mut Cursor::new(concat!(
            "WORDS:THE,OWE,MES,ROD,RODEO\n",
            "\n",
            "HELWORLT\n",
            "ENIGWDXL\n",
            "TRODEOAL\n",
        )))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
