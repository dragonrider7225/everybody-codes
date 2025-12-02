use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader, Write},
    iter,
};

use dependencies::priority_queue::PriorityQueue;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tile {
    Trampoline,
    Void,
    Start,
    End,
}

impl Tile {
    fn is_trampoline(&self) -> bool {
        match self {
            Self::Start | Self::End | Self::Trampoline => true,
            Self::Void => false,
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start => write!(f, "S"),
            Self::End => write!(f, "E"),
            Self::Trampoline => write!(f, "#"),
            Self::Void => write!(f, " "),
        }
    }
}

impl TryFrom<u8> for Tile {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'T' => Ok(Self::Trampoline),
            b'#' => Ok(Self::Void),
            b'S' => Ok(Self::Start),
            b'E' => Ok(Self::End),
            _ => Err(format!("Invalid tile: {:?}", value as char)),
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct Position {
    x: usize,
    y: usize,
}

impl Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Position {{ x: {}, y: {} }}", self.x, self.y)
    }
}

struct Map {
    tiles: Vec<Vec<Tile>>,
}

impl Map {
    fn read(input: &mut dyn BufRead) -> io::Result<Self> {
        input
            .lines()
            .map(|line| {
                line.and_then(|line| {
                    line.bytes()
                        .skip_while(|&b| b == b'.')
                        .take_while(|&b| b != b'.')
                        .map(Tile::try_from)
                        .try_fold((vec![], vec![]), |(mut above, mut below), tile| {
                            let tile = match tile {
                                Ok(tile) => tile,
                                Err(e) => return Err(e),
                            };
                            if above.len() > below.len() {
                                below.push(tile);
                            } else {
                                above.push(tile);
                            }
                            Ok((above, below))
                        })
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                })
            })
            .try_fold(vec![], |mut acc, lines| {
                let (above, below) = lines?;
                acc.push(above);
                if !below.is_empty() {
                    acc.push(below);
                }
                Ok(acc)
            })
            .map(|tiles| Self { tiles })
    }

    pub fn at(&self, position: Position) -> Option<&Tile> {
        self.tiles
            .get(position.y)
            .and_then(|row| row.get(position.x))
    }

    fn neighbors(&self, position: Position) -> impl Iterator<Item = Position> {
        let mut ret = vec![];
        let y_below = position.y + 1;
        if position.y.is_multiple_of(2) {
            // Up
            if position.y != 0 {
                ret.push(Position {
                    y: position.y - 1,
                    ..position
                });
            }
            if y_below < self.tiles.len() {
                // down-left
                if position.x != 0 {
                    ret.push(Position {
                        x: position.x - 1,
                        y: y_below,
                    });
                }
                // down
                if position.x < self.tiles[y_below].len() {
                    ret.push(Position {
                        y: y_below,
                        ..position
                    });
                }
            }
        } else {
            // Up, up-right, down
            ret.extend([
                Position {
                    y: position.y - 1,
                    ..position
                },
                Position {
                    x: position.x + 1,
                    y: position.y - 1,
                },
                Position {
                    y: position.y + 1,
                    ..position
                },
            ]);
        }
        ret.into_iter()
    }

    fn rotate(&self, position: Position) -> Position {
        let x = position.y / 2;
        let y = 2 * (self.tiles[position.y].len() - 1 - position.x)
            + usize::from(!position.y.is_multiple_of(2));
        Position { x, y }
    }

    fn neighbors_with_rotation(&self, position: Position) -> impl Iterator<Item = Position> {
        iter::once(self.rotate(position)).chain(
            self.neighbors(position)
                .map(|neighbor| self.rotate(neighbor)),
        )
    }

    #[cfg(feature = "write_images")]
    fn write_rotations(&self, mut out: impl Write) -> io::Result<()> {
        writeln!(out, "P6")?;
        writeln!(out, "{} {}", self.tiles[0].len() * 2 - 1, self.tiles.len())?;
        writeln!(out, "2")?;
        let mut padding = 0;
        for (y, row) in self.tiles.iter().enumerate() {
            for _ in 0..padding {
                out.write_all(&[0, 0, 0])?;
            }
            for p in iter::successors(Some(Position { x: 0, y }), |&p| Some(self.rotate(p))).take(3)
            {
                match self.at(p) {
                    Some(Tile::End | Tile::Start) => out.write_all(&[2])?,
                    Some(Tile::Trampoline) => out.write_all(&[1])?,
                    Some(Tile::Void) => out.write_all(&[0])?,
                    None => unreachable!(),
                }
            }
            for x in 1..row.len() {
                out.write_all(&[0, 0, 0])?;
                for p in
                    iter::successors(Some(Position { x, y }), |&p| Some(self.rotate(p))).take(3)
                {
                    match self.at(p) {
                        Some(Tile::End | Tile::Start) => out.write_all(&[2])?,
                        Some(Tile::Trampoline) => out.write_all(&[1])?,
                        Some(Tile::Void) => out.write_all(&[0])?,
                        None => unreachable!(),
                    }
                }
            }
            for _ in 0..padding {
                out.write_all(&[0, 0, 0])?;
            }
            padding += usize::from(y.is_multiple_of(2));
        }
        Ok(())
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut padding = 0;
        for (y, row) in self.tiles.iter().enumerate() {
            write!(f, "{:padding$}{}", "", row[0])?;
            for tile in &row[1..] {
                write!(f, " {tile}")?;
            }
            writeln!(f)?;
            padding += usize::from(y.is_multiple_of(2));
        }
        Ok(())
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let map = Map::read(input)?;
    Ok(map
        .tiles
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.iter()
                .copied()
                .enumerate()
                .filter(|(_, tile)| tile.is_trampoline())
                .map(move |(x, _)| Position { x, y })
                .flat_map(|position| map.neighbors(position))
                .filter(|&position| map.at(position).is_some_and(Tile::is_trampoline))
                .count()
        })
        .sum::<usize>()
        / 2)
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let map = Map::read(input)?;
    let (start, end) = match map
        .tiles
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter()
                .copied()
                .enumerate()
                .map(move |(x, tile)| (Position { x, y }, tile))
        })
        .fold((None, None), |(start, end), (position, tile)| match tile {
            Tile::Start => (Some(position), end),
            Tile::End => (start, Some(position)),
            _ => (start, end),
        }) {
        (None, None) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Room contains neither start nor end",
            ))
        }
        (Some(_), None) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Room does not contain end",
            ))
        }
        (None, Some(_)) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Room does not contain start",
            ))
        }
        (Some(start), Some(end)) => (start, end),
    };
    let mut frontier = PriorityQueue::new();
    let mut sources = HashMap::new();
    frontier.push(start, Reverse(0));
    let mut visited = HashSet::new();
    while let Some((next, Reverse(num_jumps))) = frontier.pop() {
        visited.insert(next);
        if next == end {
            return Ok(num_jumps);
        }
        map.neighbors(next)
            .filter(|neighbor| !visited.contains(neighbor))
            .filter(|&neighbor| map.at(neighbor).is_some_and(Tile::is_trampoline))
            .for_each(
                |neighbor| match frontier.push_decrease(neighbor, Reverse(num_jumps + 1)) {
                    Some(Reverse(j)) if j == num_jumps + 1 => {}
                    _ => {
                        sources
                            .entry(neighbor)
                            .and_modify(|e| *e = next)
                            .or_insert(next);
                    }
                },
            );
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "No path from start to end",
    ))
}

fn part3(input: &mut dyn BufRead, out: impl Write) -> io::Result<usize> {
    let map = Map::read(input)?;
    #[cfg(feature = "write_images")]
    map.write_rotations(out)?;
    #[cfg(not(feature = "write_images"))]
    let _ = out;
    let (start, end) = match map
        .tiles
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter()
                .copied()
                .enumerate()
                .map(move |(x, tile)| (Position { x, y }, tile))
        })
        .fold((None, None), |(start, end), (position, tile)| match tile {
            Tile::Start => (Some(position), end),
            Tile::End => (start, Some(position)),
            _ => (start, end),
        }) {
        (None, None) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Room contains neither start nor end",
            ))
        }
        (Some(_), None) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Room does not contain end",
            ))
        }
        (None, Some(_)) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Room does not contain start",
            ))
        }
        (Some(start), Some(end)) => (start, end),
    };
    let mut frontier = PriorityQueue::new();
    let mut sources = HashMap::new();
    frontier.push((start, 0), Reverse(0));
    let mut visited = HashSet::new();
    while let Some(((next, next_rot), Reverse(num_jumps))) = frontier.pop() {
        visited.insert((next, next_rot));
        if next == end {
            assert!(
                sources.iter().all(|(&(to, _), &(from, _))| map
                    .at(to)
                    .is_some_and(Tile::is_trampoline)
                    && map.at(from).is_some_and(Tile::is_trampoline)),
                "Made jump to/from non-trampoline tile"
            );
            let mut path = iter::successors(Some((end, next_rot)), |position| {
                sources.get(position).copied()
            })
            .map(|(p, rot)| match rot {
                0 => p,
                1 => map.rotate(map.rotate(p)),
                2 => map.rotate(p),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
            path.reverse();
            assert_eq!(path.len(), num_jumps + 1);
            return Ok(num_jumps);
        }
        map.neighbors_with_rotation(next)
            .map(|neighbor| (neighbor, (next_rot + 1) % 3))
            .filter(|neighbor| !visited.contains(neighbor))
            .filter(|&(neighbor, _)| map.at(neighbor).is_some_and(Tile::is_trampoline))
            .for_each(|neighbor| {
                match frontier
                    .push_decrease((neighbor.0, (next_rot + 1) % 3), Reverse(num_jumps + 1))
                {
                    Some(Reverse(j)) if j == num_jumps + 1 => {}
                    _ => {
                        sources
                            .entry(neighbor)
                            .and_modify(|e| *e = (next, next_rot))
                            .or_insert((next, next_rot));
                    }
                }
            });
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "No path from start to end",
    ))
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 20 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_20-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 20 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_20-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 20 Part 3");
        #[cfg(feature = "write_images")]
        let out = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open("SoDaD_20_3.ppm")?;
        #[cfg(not(feature = "write_images"))]
        let out = io::sink();
        println!(
            "{}",
            part3(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_20-3.txt")?),
                out,
            )?
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
        #[rustfmt::skip]
        const TEST_DATA_1: &str = concat!(
            "T#TTT###T##\n",
            ".##TT#TT##.\n",
            "..T###T#T..\n",
            "...##TT#...\n",
            "....T##....\n",
            ".....#.....\n",
        );
        let expected = 7;
        let actual = part1(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        #[rustfmt::skip]
        const TEST_DATA_2: &str = concat!(
            "T#T#T#T#T#T\n",
            ".T#T#T#T#T.\n",
            "..T#T#T#T..\n",
            "...T#T#T...\n",
            "....T#T....\n",
            ".....T.....\n",
        );
        let expected = 0;
        let actual = part1(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        #[rustfmt::skip]
        const TEST_DATA_3: &str = concat!(
            "T#T#T#T#T#T\n",
            ".#T#T#T#T#.\n",
            "..#T###T#..\n",
            "...##T##...\n",
            "....#T#....\n",
            ".....#.....\n",
        );
        let expected = 0;
        let actual = part1(&mut Cursor::new(TEST_DATA_3))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "TTTTTTTTTTTTTTTTT\n",
            ".TTTT#T#T#TTTTTT.\n",
            "..TT#TTTETT#TTT..\n",
            "...TT#T#TTT#TT...\n",
            "....TTT#T#TTT....\n",
            ".....TTTTTT#.....\n",
            "......TT#TT......\n",
            ".......#TT.......\n",
            "........S........\n",
        );
        let expected = 32;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "T####T#TTT##T##T#T#\n",
            ".T#####TTTT##TTT##.\n",
            "..TTTT#T###TTTT#T..\n",
            "...T#TTT#ETTTT##...\n",
            "....#TT##T#T##T....\n",
            ".....#TT####T#.....\n",
            "......T#TT#T#......\n",
            ".......T#TTT.......\n",
            "........TT#........\n",
            ".........S.........\n",
        );
        let expected = 23;
        eprintln!("{:?}", std::env::current_dir()?);
        let out = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open("SoDaD_20_3_test.ppm")?;
        let actual = part3(&mut Cursor::new(TEST_DATA), out)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
