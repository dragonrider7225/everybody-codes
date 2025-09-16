use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Red => write!(f, "R"),
            Self::Green => write!(f, "G"),
            Self::Blue => write!(f, "B"),
        }
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
struct Colors(pub Vec<Color>);

impl Display for Colors {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for c in &self.0 {
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

impl FromStr for Colors {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.chars()
            .map(|c| match c {
                'R' => Ok(Color::Red),
                'G' => Ok(Color::Green),
                'B' => Ok(Color::Blue),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid color {c:?}"),
                )),
            })
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

fn bolt_number_to_color(bolt_number: usize) -> Color {
    match bolt_number % 3 {
        0 => Color::Red,
        1 => Color::Green,
        2 => Color::Blue,
        _ => unreachable!(),
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut balloons = input
        .lines()
        .next()
        .unwrap()?
        .parse::<Colors>()?
        .0
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    for i in 0.. {
        let bolt_color = bolt_number_to_color(i);
        let mut popped = vec![];
        loop {
            match balloons.pop() {
                None if popped.is_empty() => return Ok(i),
                None => return Ok(i + 1),
                Some(balloon_color) if balloon_color != bolt_color => {
                    popped.push(balloon_color);
                    break;
                }
                Some(balloon_color) => popped.push(balloon_color),
            }
        }
    }
    panic!("Need more than {} darts to pop all the balloons", u32::MAX);
}

struct BalloonWheel {
    base: Vec<Color>,
    calculated: Vec<Option<Vec<Color>>>,
    num_balloons: usize,
}

impl BalloonWheel {
    pub fn new(base: Vec<Color>, num_repeats: usize) -> Self {
        Self {
            num_balloons: base.len() * num_repeats,
            base,
            calculated: vec![None; num_repeats],
        }
    }

    pub fn pop(&mut self, bolt_color: Color) -> usize {
        if self.num_balloons == 0 {
            return 0;
        }
        let mut last = loop {
            match self.calculated.pop().unwrap() {
                None => break self.base.clone(),
                Some(last) if !last.is_empty() => break last,
                Some(_) => {}
            }
        };
        let front_balloon = last.pop().unwrap();
        self.calculated.push(Some(last));
        let mut popped = Colors(vec![front_balloon]);
        if front_balloon == bolt_color && self.num_balloons % 2 == 0 {
            let separator_length = (self.num_balloons - 2) / 2;
            let definite_nones = separator_length / self.base.len();
            let num_balloons_skipped = definite_nones * self.base.len();
            let intermediate = &mut self.calculated[definite_nones];
            if intermediate.is_none() {
                intermediate.replace(self.base.clone());
            }
            let intermediate = intermediate.as_mut().unwrap();
            popped
                .0
                .push(intermediate.remove(separator_length - num_balloons_skipped));
            self.num_balloons -= 1;
        }
        #[cfg(test)]
        eprintln!("{bolt_color}: {popped}");
        self.num_balloons -= 1;
        self.num_balloons
    }
}

impl Display for BalloonWheel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} balloons: ", self.num_balloons)?;
        let wrapped_base = Colors(self.base.clone());
        for seq in &self.calculated {
            write!(f, "|")?;
            if seq.is_none() {
                write!(f, "{wrapped_base}")?;
            } else {
                write!(f, "{}", Colors(seq.as_ref().unwrap().clone()))?;
            }
            write!(f, "|")?;
        }
        Ok(())
    }
}

fn part2(input: &mut dyn BufRead, num_repeats: usize) -> io::Result<usize> {
    let base = {
        let mut v = input.lines().next().unwrap()?.parse::<Colors>()?.0;
        v.reverse();
        v
    };
    let mut balloons = BalloonWheel::new(base, num_repeats);
    for i in 0.. {
        #[cfg(test)]
        eprint!("{i:03} ");
        let bolt_color = bolt_number_to_color(i);
        if balloons.pop(bolt_color) == 0 {
            return Ok(i + 1);
        }
    }
    panic!("Need more than {} darts to pop all the balloons", u32::MAX);
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    part2(input, 100_000)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("The Entertainment Hub Quest 2 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "the-entertainment-hub_02-1.txt"
            )?))?
        );
    }
    {
        println!("The Entertainment Hub Quest 2 Part 2");
        println!(
            "{}",
            part2(
                &mut BufReader::new(File::open("the-entertainment-hub_02-2.txt")?),
                100,
            )?
        );
    }
    {
        println!("The Entertainment Hub Quest 2 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "the-entertainment-hub_02-3.txt"
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
        const TEST_DATA: &str = "GRBGGGBBBRRRRRRRR\n";
        let expected = 7;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA_4: &str = "GGBR\n";
        let expected = 14;
        let actual = part2(&mut Cursor::new(TEST_DATA_4), 5)?;
        assert_eq!(expected, actual);

        const TEST_DATA_35: &str = "BBRGGRRGBBRGGBRGBBRRBRRRBGGRRRBGBGG\n";
        let expected = 304;
        let actual = part2(&mut Cursor::new(TEST_DATA_35), 10)?;
        assert_eq!(expected, actual);
        let expected = 1464;
        let actual = part2(&mut Cursor::new(TEST_DATA_35), 50)?;
        assert_eq!(expected, actual);
        let expected = 2955;
        let actual = part2(&mut Cursor::new(TEST_DATA_35), 100)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
