use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    str::FromStr,
};

use itertools::{Itertools, MinMaxResult};

struct TokenPosition {
    row: usize,
    column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Pegboard {
    rows: Vec<Vec<bool>>,
}

impl Pegboard {
    fn step(&self, token: &mut Token, token_position: TokenPosition) -> TokenPosition {
        if token_position.row >= self.rows.len() {
            token_position
        } else if token_position.row % 2 != token_position.column % 2 {
            unreachable!("token_position.row + token_position.column always changes by 0 or 2")
        } else if !self.rows[token_position.row][token_position.column / 2] {
            TokenPosition {
                row: token_position.row + 2,
                ..token_position
            }
        } else {
            // Handle the collision with the peg at `token_position`.

            // Always consume a bounce, even if there's a wall in the way.
            let direction = token
                .next()
                .expect("Token ran out of bounces before exiting pegboard");
            let direction = if token_position.column == 0 {
                Bounce::Right
            } else if token_position.column == 2 * (self.rows[0].len() - 1) {
                Bounce::Left
            } else {
                direction
            };
            match direction {
                Bounce::Left => TokenPosition {
                    row: token_position.row + 1,
                    column: token_position.column - 1,
                },
                Bounce::Right => TokenPosition {
                    row: token_position.row + 1,
                    column: token_position.column + 1,
                },
            }
        }
    }

    pub fn num_slots(&self) -> usize {
        self.rows[0].len()
    }

    /// Insert a token into the slot marked `slot`. Slots start counting from 1.
    fn insert_token(&self, slot: usize) -> Option<TokenPosition> {
        if slot <= self.num_slots() {
            Some(TokenPosition {
                row: 0,
                column: 2 * (slot - 1),
            })
        } else {
            None
        }
    }

    /// Insert a token into the slot marked `slot` and calculate the payout.
    pub fn drop_token(&self, slot: usize, token: &mut Token) -> Option<usize> {
        let mut position = self.insert_token(slot)?;
        while position.row < self.rows.len() {
            position = self.step(token, position);
        }
        Some((position.column + 2).saturating_sub(slot))
    }
}

impl FromStr for Pegboard {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s
            .lines()
            .enumerate()
            .map(|(idx, line)| {
                let bytes = &line.as_bytes()[(idx % 2)..];
                for i in (1..bytes.len()).step_by(2) {
                    if bytes[i] != b'.' {
                        return Err(format!(
                            "Found non-space at column {} of row {idx}",
                            i + (idx % 2)
                        ));
                    }
                }
                Ok(bytes.iter().step_by(2).map(|&peg| peg == b'*').collect())
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { rows })
    }
}

struct Token {
    bounces: Vec<Bounce>,
    index: usize,
}

impl Token {
    pub fn reset(&mut self) {
        self.index = 0;
    }
}

impl FromIterator<Bounce> for Token {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Bounce>,
    {
        Self {
            bounces: iter.into_iter().collect(),
            index: 0,
        }
    }
}

impl Iterator for Token {
    type Item = Bounce;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.bounces.len() {
            None
        } else {
            let index = self.index;
            self.index += 1;
            self.bounces.get(index).copied()
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Bounce {
    Left,
    Right,
}

fn read_pegboard_and_tokens(input: &mut dyn BufRead) -> io::Result<(Pegboard, Vec<Token>)> {
    let s = {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        buf
    };
    let (pegboard, tokens) = match s.split_once("\n\n") {
        Some(pair) => pair,
        None => s
            .split_once("\r\n\r\n")
            .expect("Only supports '\\n' and '\\r\\n' line endings"),
    };
    let pegboard = pegboard
        .parse::<Pegboard>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let tokens = tokens
        .lines()
        .map(|token| {
            token
                .chars()
                .map(|c| match c {
                    'R' => Ok(Bounce::Right),
                    'L' => Ok(Bounce::Left),
                    _ => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Found invalid character {c:?} in token string {token:?}"),
                    )),
                })
                .collect::<Result<Token, _>>()
        })
        .collect::<io::Result<_>>()?;
    Ok((pegboard, tokens))
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let (pegboard, tokens) = read_pegboard_and_tokens(input)?;
    tokens
        .into_iter()
        .enumerate()
        .map(|(i, mut token)| {
            Ok(pegboard
                .drop_token(i + 1, &mut token)
                .expect("Too many tokens"))
        })
        .try_fold(0, |acc, winnings: io::Result<_>| Ok(acc + winnings?))
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let (pegboard, tokens) = read_pegboard_and_tokens(input)?;
    Ok(tokens
        .into_iter()
        .map(|mut token| {
            (1..=pegboard.num_slots())
                .map(|slot| {
                    token.reset();
                    pegboard.drop_token(slot, &mut token).unwrap()
                })
                .max()
                .expect("Pegboard contains no slots")
        })
        .sum())
}

fn part3(input: &mut dyn BufRead) -> io::Result<(usize, usize)> {
    let (pegboard, tokens) = read_pegboard_and_tokens(input)?;
    let cache = tokens
        .into_iter()
        .map(|mut token| {
            (1..=pegboard.num_slots())
                .map(|slot| {
                    token.reset();
                    pegboard.drop_token(slot, &mut token).unwrap()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let extremes = (0..pegboard.num_slots())
        .permutations(cache.len())
        .map(|slots| {
            cache
                .iter()
                .zip(slots)
                .map(|(results, slot)| results[slot])
                .sum::<usize>()
        })
        .minmax();
    match extremes {
        MinMaxResult::NoElements => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Pegboard must have at least as many slots ({}) as tokens to insert ({})",
                pegboard.num_slots(),
                cache.len()
            ),
        )),
        MinMaxResult::OneElement(total) => Ok((total, total)),
        MinMaxResult::MinMax(least, most) => Ok((least, most)),
    }
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("The Entertainment Hub Quest 1 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "the-entertainment-hub_01-1.txt"
            )?))?
        );
    }
    {
        println!("The Entertainment Hub Quest 1 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "the-entertainment-hub_01-2.txt"
            )?))?
        );
    }
    {
        println!("The Entertainment Hub Quest 1 Part 3");
        let (min, max) = part3(&mut BufReader::new(File::open(
            "the-entertainment-hub_01-3.txt",
        )?))?;
        println!("{min} {max}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_pegboard_parse() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "*.*.*.*.*\n",
            ".*.*.*.*.\n",
            "*...*.*.*\n",
            ".*.*...*.\n",
            "*.*...*..\n",
            ".*.*.*.*.\n",
        );
        let mut rows: Vec<Vec<bool>> = vec![vec![true; 5], vec![true; 4]]
            .into_iter()
            .cycle()
            .take(6)
            .collect();
        rows[2][1] = false;
        rows[3][2] = false;
        rows[4][2] = false;
        rows[4][4] = false;
        let expected = Pegboard { rows };
        let actual = TEST_DATA.parse::<Pegboard>().unwrap();
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part1() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.\n",
            "*.*.*...*.*...*..\n",
            ".*.*.*.*.*...*.*.\n",
            "*.*.....*...*.*.*\n",
            ".*.*.*.*.*.*.*.*.\n",
            "*...*...*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.\n",
            "*.*.*...*.*.*.*.*\n",
            ".*...*...*.*.*.*.\n",
            "*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.\n",
            "\n",
            "RRRLRLRRRRRL\n",
            "LLLLRLRRRRRR\n",
            "RLLLLLRLRLRL\n",
            "LRLLLRRRLRLR\n",
            "LLRLLRLLLRRL\n",
            "LRLRLLLRRRRL\n",
            "LRLLLLLLRLLL\n",
            "RRLLLRLLRLRR\n",
            "RLLLLLRLLLRL\n",
        );
        let expected = 26;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "*.*.*.*.*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.*.*.*.*.\n",
            "..*.*.*.*...*.*...*.*.*..\n",
            ".*...*.*.*.*.*.*.....*.*.\n",
            "*.*...*.*.*.*.*.*...*.*.*\n",
            ".*.*.*.*.*.*.*.*.......*.\n",
            "*.*.*.*.*.*.*.*.*.*...*..\n",
            ".*.*.*.*.*.*.*.*.....*.*.\n",
            "*.*...*.*.*.*.*.*.*.*....\n",
            ".*.*.*.*.*.*.*.*.*.*.*.*.\n",
            "*.*.*.*.*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.*...*.*.\n",
            "*.*.*.*.*.*.*.*.*...*.*.*\n",
            ".*.*.*.*.*.*.*.*.....*.*.\n",
            "*.*.*.*.*.*.*.*...*...*.*\n",
            ".*.*.*.*.*.*.*.*.*.*.*.*.\n",
            "*.*.*...*.*.*.*.*.*.*.*.*\n",
            ".*...*.*.*.*...*.*.*...*.\n",
            "*.*.*.*.*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.*.*.*.*.\n",
            "\n",
            "RRRLLRRRLLRLRRLLLRLR\n",
            "RRRRRRRRRRLRRRRRLLRR\n",
            "LLLLLLLLRLRRLLRRLRLL\n",
            "RRRLLRRRLLRLLRLLLRRL\n",
            "RLRLLLRRLRRRLRRLRRRL\n",
            "LLLLLLLLRLLRRLLRLLLL\n",
            "LRLLRRLRLLLLLLLRLRRL\n",
            "LRLLRRLLLRRRRRLRRLRR\n",
            "LRLLRRLRLLRLRRLLLRLL\n",
            "RLLRRRRLRLRLRLRLLRRL\n",
        );
        let expected = 115;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_1() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.\n",
            "*.*.*...*.*...*..\n",
            ".*.*.*.*.*...*.*.\n",
            "*.*.....*...*.*.*\n",
            ".*.*.*.*.*.*.*.*.\n",
            "*...*...*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.\n",
            "*.*.*...*.*.*.*.*\n",
            ".*...*...*.*.*.*.\n",
            "*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.\n",
            "\n",
            "RRRLRLRRRRRL\n",
            "LLLLRLRRRRRR\n",
            "RLLLLLRLRLRL\n",
            "LRLLLRRRLRLR\n",
            "LLRLLRLLLRRL\n",
            "LRLRLLLRRRRL\n",
        );
        let expected = (13, 43);
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "*.*.*.*.*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.*.*.*.*.\n",
            "..*.*.*.*...*.*...*.*.*..\n",
            ".*...*.*.*.*.*.*.....*.*.\n",
            "*.*...*.*.*.*.*.*...*.*.*\n",
            ".*.*.*.*.*.*.*.*.......*.\n",
            "*.*.*.*.*.*.*.*.*.*...*..\n",
            ".*.*.*.*.*.*.*.*.....*.*.\n",
            "*.*...*.*.*.*.*.*.*.*....\n",
            ".*.*.*.*.*.*.*.*.*.*.*.*.\n",
            "*.*.*.*.*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.*...*.*.\n",
            "*.*.*.*.*.*.*.*.*...*.*.*\n",
            ".*.*.*.*.*.*.*.*.....*.*.\n",
            "*.*.*.*.*.*.*.*...*...*.*\n",
            ".*.*.*.*.*.*.*.*.*.*.*.*.\n",
            "*.*.*...*.*.*.*.*.*.*.*.*\n",
            ".*...*.*.*.*...*.*.*...*.\n",
            "*.*.*.*.*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.*.*.*.*.\n",
            "\n",
            "RRRLLRRRLLRLRRLLLRLR\n",
            "RRRRRRRRRRLRRRRRLLRR\n",
            "LLLLLLLLRLRRLLRRLRLL\n",
            "RRRLLRRRLLRLLRLLLRRL\n",
            "RLRLLLRRLRRRLRRLRRRL\n",
            "LLLLLLLLRLLRRLLRLLLL\n",
        );
        let expected = (25, 66);
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_3() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.\n",
            "..*.*.*.*.*.*.........*.*.*.*.....*.*.*\n",
            ".*.*...*.*.*.*.*.*.*.*.*.*.*...*.*.*.*.\n",
            "*.*.*.*...*.*.*.*.*.....*.*.*.*...*.*..\n",
            ".*...*.*...*.*.*.*.*.*.*.....*.*.*.*.*.\n",
            "*.*.*.*.*.....*.*.*.*.*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*...*.*.*.*.....*.*.*.*...*.\n",
            "*.*...*.*.*.*.*.*.*.*...*.*.*...*.*.*.*\n",
            ".*...*.*.*.*.*.*.*.*...*.*.*.*.*.*.*.*.\n",
            "*.*.*.*.*.*...*.....*.*...*...*.*.*.*.*\n",
            ".*...*.*.*.*.*...*.*.*.*.*...*.*...*.*.\n",
            "*.*.*.*.*...*.*.*.*.*.*.*.*...*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*...*.*.*.*.*.*.*.*.*.*.\n",
            "....*.*.*.*...*.*.*.*.*.*.*...*.*.*...*\n",
            ".*.*.*...*.*.*.*.*...*.*.*.*.*.*.*.*...\n",
            "*.*.*.*.*.*.*.....*...*...*.*.*.*.*.*.*\n",
            ".*.*...*.....*.*.*.*.*.*.*...*.*.*.*.*.\n",
            "*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*\n",
            ".*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.*.\n",
            "\n",
            "RRRRLLRRLLLLLLLRLLRL\n",
            "RRRRRRRLRRLRRLRRRLRR\n",
            "RRRLLRRRRRLRRRRRLRRR\n",
            "LLLLRRLLRRLLLLLRRLLL\n",
            "LRRRRLRRLRLLRLLRRLRR\n",
            "RRRRRRRRLRRRRLLRRRLR\n",
        );
        let expected = (39, 122);
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
