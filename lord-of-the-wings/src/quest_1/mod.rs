use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

fn run_part(
    input: &mut dyn BufRead,
    mut additional_pinecones_required: impl FnMut(u64) -> u64,
) -> io::Result<u64> {
    let mut buf = vec![];
    let mut ret = 0;
    while {
        buf.clear();
        0 != input.read_until(b',', &mut buf)?
    } {
        if buf.last().unwrap() == &b',' {
            buf.pop();
        }
        ret += buf
            .split(|&b| b == b'\n')
            .map(|buf| {
                if buf.is_empty() {
                    return Ok(0);
                }
                if !buf.iter().all(u8::is_ascii_digit) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Input data contains non-numerical segment: {buf:?}"),
                    ));
                }
                let remaining_pinecones =
                    str::from_utf8(buf).expect("all(u8::is_ascii_digit) is UTF-8");
                let remaining_pinecones = remaining_pinecones.parse::<u64>().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Couldn't parse {remaining_pinecones:?} as u64"),
                    )
                })?;
                let to_collect = additional_pinecones_required(remaining_pinecones);
                #[cfg(test)]
                eprintln!("{remaining_pinecones} remaining, need to collect {to_collect} more");
                Ok(to_collect)
            })
            .sum::<io::Result<u64>>()?;
    }
    Ok(ret)
}

fn part1(input: &mut dyn BufRead) -> io::Result<u64> {
    run_part(input, |remaining_pinecones| {
        let best_side_length = remaining_pinecones.isqrt();
        if best_side_length * best_side_length < remaining_pinecones {
            (best_side_length + 1).pow(2) - remaining_pinecones
        } else {
            0
        }
    })
}

fn part2(input: &mut dyn BufRead) -> io::Result<u64> {
    run_part(input, |remaining_pinecones| {
        let best_side_length = remaining_pinecones / 4;
        if best_side_length < 2 {
            8 - remaining_pinecones
        } else if best_side_length * 4 < remaining_pinecones {
            (best_side_length + 1) * 4 - remaining_pinecones
        } else {
            0
        }
    })
}

fn part3(input: &mut dyn BufRead) -> io::Result<u64> {
    run_part(input, |remaining_pinecones| {
        // \Sum_{i=1}^k 8i >= remaining_pinecones
        // 8\Sum_{i=1}^k i >= remaining_pinecones
        // 4*k*(k+1) >= remaining_pinecones
        // 4k**2 + 4k >= remaining_pinecones
        // k**2 + k >= 0.25*remaining_pinecones
        // k**2 + k + 0.25 >= 0.25*(remaining_pinecones + 1)
        // (k+0.5)**2 >= 0.25*(remaining_pinecones + 1)
        // k+0.5 >= 0.5*sqrt(remaining_pinecones + 1.0)
        // k >= 0.5*(sqrt(remaining_pinecones + 1.0) - 1)
        let best_available_side_length = (remaining_pinecones + 1).isqrt();
        let target_side_length = best_available_side_length
            + ((best_available_side_length * best_available_side_length - 1 < remaining_pinecones)
                as u64);
        let target_side_length = target_side_length + (target_side_length.is_multiple_of(2) as u64);
        target_side_length.pow(2) - 1 - remaining_pinecones
    })
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Lord of the wings Quest 1 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "lord-of-the-wings_01-1.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 1 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "lord-of-the-wings_01-2.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 1 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "lord-of-the-wings_01-3.txt"
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
        const TEST_DATA: &str = "3,11,12,16,24,25\n";
        let expected = 11;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = "3,11,12,16,24,25\n";
        let expected = 9;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = "3,11,12,16,24,25\n";
        let expected = 61;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
