use std::{
    cmp::{Ordering, Reverse},
    fs::File,
    io::{self, BufRead, BufReader},
};

macro_rules! parse_color {
    ($fn_name:ident, $color:literal, $off:literal, $on:literal) => {
        fn $fn_name(s: &str) -> io::Result<u8> {
            s.bytes().try_fold(0, |acc, b| match b {
                $off => Ok(2 * acc),
                $on => Ok(2 * acc + 1),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        concat!($color, " string must consist of {off:?} and {on:?}"),
                        off = $off as char,
                        on = $on as char
                    ),
                )),
            })
        }
    };
}

parse_color!(parse_red, "Red", b'r', b'R');
parse_color!(parse_green, "Green", b'g', b'G');
parse_color!(parse_blue, "Blue", b'b', b'B');
parse_color!(parse_shine, "Shine", b's', b'S');

fn part1(input: &mut dyn BufRead) -> io::Result<u32> {
    struct Color {
        pub red: u8,
        pub green: u8,
        pub blue: u8,
    }

    fn parse_line(line: &str) -> io::Result<(u32, Color)> {
        let (id, colors) = line.split_once(':').ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{line:?} does not contain scale identifier"),
            )
        })?;
        let id = id.parse::<u32>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "{id:?} is not a valid identifier",
            )
        })?;
        let (red, gb) = colors
            .split_once(' ')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing space after red"))?;
        let (green, blue) = gb.split_once(' ').ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Missing space after green")
        })?;
        let (red, green, blue) = (parse_red(red)?, parse_green(green)?, parse_blue(blue)?);
        Ok((id, Color { red, green, blue }))
    }

    input
        .lines()
        .map(|line| line.and_then(|line| parse_line(&line)))
        .filter_map(|parsed_line| match parsed_line {
            Ok((id, Color { red, green, blue })) => {
                Some(Ok(id)).filter(|_| green > red && green > blue)
            }
            Err(e) => Some(Err(e)),
        })
        .sum()
}

fn part2(input: &mut dyn BufRead) -> io::Result<u32> {
    #[derive(Clone, Copy, Debug)]
    struct Color {
        pub red: u8,
        pub green: u8,
        pub blue: u8,
        pub shine: u8,
    }

    impl Color {
        pub fn brightness(&self) -> u8 {
            self.red + self.green + self.blue
        }
    }

    fn parse_line(line: &str) -> io::Result<(u32, Color)> {
        let (id, colors) = line.split_once(':').ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{line:?} does not contain scale identifier"),
            )
        })?;
        let id = id.parse::<u32>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "{id:?} is not a valid identifier",
            )
        })?;
        let (red, gbs) = colors
            .split_once(' ')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing space after red"))?;
        let (green, blue_shine) = gbs.split_once(' ').ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Missing space after green")
        })?;
        let (blue, shine) = blue_shine.split_once(' ').ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Missing space after blue")
        })?;
        let color = Color {
            red: parse_red(red)?,
            green: parse_green(green)?,
            blue: parse_blue(blue)?,
            shine: parse_shine(shine)?,
        };
        Ok((id, color))
    }

    input
        .lines()
        .map(|line| line.and_then(|line| parse_line(&line)))
        .min_by(|left, right| match (left, right) {
            (Err(_), _) => Ordering::Less,
            (_, Err(_)) => Ordering::Greater,
            (Ok((_, left)), Ok((_, right))) => Reverse(left.shine)
                .cmp(&Reverse(right.shine))
                .then_with(|| left.brightness().cmp(&right.brightness())),
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "No scales found"))
        .flatten()
        .map(|(id, _)| id)
}

fn part3(input: &mut dyn BufRead) -> io::Result<u32> {
    #[derive(Clone, Copy, Debug)]
    struct Color {
        pub red: u8,
        pub green: u8,
        pub blue: u8,
        pub shine: u8,
    }

    impl Color {
        pub fn is_matte(&self) -> bool {
            self.shine <= 30
        }

        pub fn is_shiny(&self) -> bool {
            self.shine >= 33
        }
    }

    fn parse_line(line: &str) -> io::Result<(u32, Color)> {
        let (id, colors) = line.split_once(':').ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{line:?} does not contain scale identifier"),
            )
        })?;
        let id = id.parse::<u32>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "{id:?} is not a valid identifier",
            )
        })?;
        let (red, gbs) = colors
            .split_once(' ')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing space after red"))?;
        let (green, blue_shine) = gbs.split_once(' ').ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Missing space after green")
        })?;
        let (blue, shine) = blue_shine.split_once(' ').ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Missing space after blue")
        })?;
        let color = Color {
            red: parse_red(red)?,
            green: parse_green(green)?,
            blue: parse_blue(blue)?,
            shine: parse_shine(shine)?,
        };
        Ok((id, color))
    }

    #[derive(Clone, Copy, Debug, Default)]
    struct Counts {
        red_matte: (usize, u32),
        red_shiny: (usize, u32),
        green_matte: (usize, u32),
        green_shiny: (usize, u32),
        blue_matte: (usize, u32),
        blue_shiny: (usize, u32),
    }

    impl Counts {
        pub fn largest_group(&self) -> u32 {
            if [
                self.red_shiny,
                self.green_matte,
                self.green_shiny,
                self.blue_matte,
                self.blue_shiny,
            ]
            .iter()
            .all(|&(count, _)| self.red_matte.0 > count)
            {
                self.red_matte.1
            } else if [
                self.green_matte,
                self.green_shiny,
                self.blue_matte,
                self.blue_shiny,
            ]
            .iter()
            .all(|&(count, _)| self.red_shiny.0 > count)
            {
                self.red_shiny.1
            } else if [self.green_shiny, self.blue_matte, self.blue_shiny]
                .iter()
                .all(|&(count, _)| self.green_matte.0 > count)
            {
                self.green_matte.1
            } else if [self.blue_matte, self.blue_shiny]
                .iter()
                .all(|&(count, _)| self.green_shiny.0 > count)
            {
                self.green_shiny.1
            } else if [self.blue_shiny]
                .iter()
                .all(|&(count, _)| self.blue_matte.0 > count)
            {
                self.blue_matte.1
            } else {
                self.blue_shiny.1
            }
        }
    }

    input
        .lines()
        .map(|line| line.and_then(|line| parse_line(&line)))
        .try_fold(Counts::default(), |mut acc, parsed_line| {
            let (id, color) = parsed_line?;
            if !color.is_matte() && !color.is_shiny() {
                return Ok(acc);
            }
            let rg = color.red.cmp(&color.green);
            let rb = color.red.cmp(&color.blue);
            let gb = color.green.cmp(&color.blue);
            let counter = match (rg, rb, gb) {
                (Ordering::Greater, Ordering::Greater, _) => {
                    if color.is_matte() {
                        &mut acc.red_matte
                    } else {
                        &mut acc.red_shiny
                    }
                }
                (Ordering::Less, _, Ordering::Greater) => {
                    if color.is_matte() {
                        &mut acc.green_matte
                    } else {
                        &mut acc.green_shiny
                    }
                }
                (_, Ordering::Less, Ordering::Less) => {
                    if color.is_matte() {
                        &mut acc.blue_matte
                    } else {
                        &mut acc.blue_shiny
                    }
                }
                // Any other triple is either "no dominant color" or mathematically impossible.
                _ => {
                    return Ok(acc);
                }
            };
            counter.0 += 1;
            counter.1 += id;
            Ok(acc)
        })
        .map(|counts| counts.largest_group())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Melody Made of Code Quest 1 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "melody-made-of-code_01-1.txt"
            )?))?
        );
    }
    {
        println!("Melody Made of Code Quest 1 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "melody-made-of-code_01-2.txt"
            )?))?
        );
    }
    {
        println!("Melody Made of Code Quest 1 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "melody-made-of-code_01-3.txt"
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
        const TEST_DATA: &str = concat!(
            "2456:rrrrrr ggGgGG bbbbBB\n",
            "7689:rrRrrr ggGggg bbbBBB\n",
            "3145:rrRrRr gggGgg bbbbBB\n",
            "6710:rrrRRr ggGGGg bbBBbB\n",
        );
        let expected = 9166;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "2456:rrrrrr ggGgGG bbbbBB sSsSsS\n",
            "7689:rrRrrr ggGggg bbbBBB ssSSss\n",
            "3145:rrRrRr gggGgg bbbbBB sSsSsS\n",
            "6710:rrrRRr ggGGGg bbBBbB ssSSss\n",
        );
        let expected = 2456;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "15437:rRrrRR gGGGGG BBBBBB sSSSSS\n",
            "94682:RrRrrR gGGggG bBBBBB ssSSSs\n",
            "56513:RRRrrr ggGGgG bbbBbb ssSsSS\n",
            "76346:rRRrrR GGgggg bbbBBB ssssSs\n",
            "87569:rrRRrR gGGGGg BbbbbB SssSss\n",
            "44191:rrrrrr gGgGGG bBBbbB sSssSS\n",
            "49176:rRRrRr GggggG BbBbbb sSSssS\n",
            "85071:RRrrrr GgGGgg BBbbbb SSsSss\n",
            "44303:rRRrrR gGggGg bBbBBB SsSSSs\n",
            "94978:rrRrRR ggGggG BBbBBb SSSSSS\n",
            "26325:rrRRrr gGGGgg BBbBbb SssssS\n",
            "43463:rrrrRR gGgGgg bBBbBB sSssSs\n",
            "15059:RRrrrR GGgggG bbBBbb sSSsSS\n",
            "85004:RRRrrR GgGgGG bbbBBB sSssss\n",
            "56121:RRrRrr gGgGgg BbbbBB sSsSSs\n",
            "80219:rRRrRR GGGggg BBbbbb SssSSs\n",
        );
        let expected = 292_320;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
