use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    ops::{Bound, RangeBounds},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LimitedUsize {
    /// The actual value.
    inner: usize,
    /// The maximum value that can be stored in `inner`. `self.base + self.max_inner` will never
    /// overflow.
    max_inner: usize,
    /// The value represented by `Self { inner: 0, .. }`. `self.base + self.max_inner` will never
    /// overflow.
    base: usize,
}

impl LimitedUsize {
    #[must_use]
    pub fn new(value: usize, range: impl RangeBounds<usize>) -> Option<Self> {
        let base = match range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Excluded(&start) => start.checked_add(1)?,
            Bound::Included(&start) => start,
        };
        let max_inner = match range.end_bound() {
            Bound::Unbounded => usize::MAX,
            Bound::Excluded(&end) => end.checked_sub(base).and_then(|end| end.checked_sub(1))?,
            Bound::Included(&end) => end.checked_sub(base)?,
        }
        .checked_sub(base)?;
        let inner = value.saturating_sub(base).min(max_inner);
        Some(Self {
            inner,
            max_inner,
            base,
        })
    }

    #[must_use]
    pub fn saturating_add_signed(mut self, delta: isize) -> Self {
        self.replace_inner(self.inner.saturating_add_signed(delta));
        self
    }

    #[must_use]
    pub fn wrapping_add_signed(mut self, delta: isize) -> Self {
        if delta != 0 {
            let delta_abs = delta.abs_diff(0);
            match self.max_inner.checked_add(1) {
                None => {
                    if delta < 0 {
                        self.inner = self.inner.wrapping_sub(delta_abs)
                    } else {
                        self.inner = self.inner.wrapping_add(delta_abs);
                    }
                }
                Some(range_len) => {
                    let delta = if delta < 0 {
                        range_len - (delta_abs % range_len)
                    } else {
                        delta_abs % range_len
                    };
                    self.inner = if let Some(new_inner) = self.inner.checked_add(delta) {
                        new_inner % range_len
                    } else {
                        self.inner - (range_len - delta)
                    };
                }
            }
        }
        self
    }

    /// Replaces the current value with `value`, clamping to the range as necessary. Returns
    /// whether `value` had to be modified to insert it.
    pub fn replace_inner(&mut self, value: usize) -> bool {
        match value.checked_sub(self.base) {
            None => {
                self.inner = 0;
                true
            }
            Some(new_inner) => {
                if new_inner > self.max_inner {
                    self.inner = self.max_inner;
                    true
                } else {
                    self.inner = new_inner;
                    false
                }
            }
        }
    }

    pub fn value(&self) -> usize {
        self.inner + self.base
    }

    pub fn min_value(&self) -> usize {
        self.base
    }

    pub fn max_value(&self) -> usize {
        self.base + self.max_inner
    }
}

impl Display for LimitedUsize {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}..={})",
            self.value(),
            self.min_value(),
            self.max_value()
        )
    }
}

fn read_names(input: &mut dyn BufRead) -> io::Result<Vec<String>> {
    input
        .lines()
        .take_while(|line| !line.as_ref().is_ok_and(|line| line.is_empty()))
        .map(|line| line.map(|line| line.split(',').map(str::to_string).collect::<Vec<_>>()))
        .try_fold(vec![], |mut acc, names| {
            if acc.is_empty() {
                names
            } else {
                acc.extend(names?);
                Ok(acc)
            }
        })
}

fn read_instructions(input: &mut dyn BufRead) -> io::Result<Vec<isize>> {
    const INVALID_INSTRUCTION_MSG: &str =
        "Instructions must match the regular expression /[RL]\\d\\+/";
    input
        .lines()
        .map(|line| {
            let line = line?;
            line.split(',')
                .map(|instruction| match *instruction.as_bytes() {
                    [b'R', ..] => instruction[1..].parse::<isize>().map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidData, INVALID_INSTRUCTION_MSG)
                    }),
                    [b'L', ..] => instruction[1..]
                        .parse::<isize>()
                        .map_err(|_| {
                            io::Error::new(io::ErrorKind::InvalidData, INVALID_INSTRUCTION_MSG)
                        })
                        .map(|n| -n),
                    _ => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        INVALID_INSTRUCTION_MSG,
                    )),
                })
                .collect::<io::Result<Vec<_>>>()
        })
        .try_fold(vec![], |mut acc, instructions| {
            if acc.is_empty() {
                instructions
            } else {
                acc.extend(instructions?);
                Ok(acc)
            }
        })
}

fn part1(input: &mut dyn BufRead) -> io::Result<String> {
    let mut names = read_names(input)?;
    let instructions = read_instructions(input)?;
    let name_idx = instructions
        .into_iter()
        .fold(
            LimitedUsize::new(0, ..names.len()).expect("Missing names"),
            |acc, instruction| acc.saturating_add_signed(instruction),
        )
        .value();
    Ok(names.swap_remove(name_idx))
}

fn part2(input: &mut dyn BufRead) -> io::Result<String> {
    let mut names = read_names(input)?;
    let instructions = read_instructions(input)?;
    let name_idx = instructions
        .into_iter()
        .fold(
            LimitedUsize::new(0, ..names.len()).expect("Missing names"),
            |acc, instruction| acc.wrapping_add_signed(instruction),
        )
        .value();
    Ok(names.swap_remove(name_idx))
}

fn part3(input: &mut dyn BufRead) -> io::Result<String> {
    let mut names = read_names(input)?;
    let num_names = names.len();
    let instructions = read_instructions(input)?;
    instructions
        .into_iter()
        .filter_map(|instruction| {
            LimitedUsize::new(0, 0..num_names).map(|base| base.wrapping_add_signed(instruction))
        })
        .for_each(|instruction| names.swap(0, instruction.value()));
    Ok(names.swap_remove(0))
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 1 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_01-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 1 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_01-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 1 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_01-3.txt"
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
        const TEST_DATA: &str = "Vyrdax,Drakzyph,Fyrryn,Elarzris\n\nR3,L2,R3,L1\n";
        let expected = "Fyrryn";
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = "Vyrdax,Drakzyph,Fyrryn,Elarzris\n\nR3,L2,R3,L1\n";
        let expected = "Elarzris";
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = "Vyrdax,Drakzyph,Fyrryn,Elarzris\n\nR3,L2,R3,L3\n";
        let expected = "Drakzyph";
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
