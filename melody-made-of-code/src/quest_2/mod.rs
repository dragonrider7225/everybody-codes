use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    iter,
    ops::{Add, AddAssign},
    time::Instant,
    vec,
};

#[cfg(test)]
fn write_grid(
    mut out: impl std::io::Write,
    min: Position,
    max: Position,
    next: Position,
    is_path: impl Fn(&Position) -> bool,
    is_enclosed: impl Fn(&Position) -> bool,
    is_bone: impl Fn(&Position) -> bool,
) -> io::Result<()> {
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            let here = Position { x, y };
            let c = if next == here {
                '@'
            } else if is_path(&here) {
                '+'
            } else if is_enclosed(&here) {
                '*'
            } else if is_bone(&here) {
                '#'
            } else {
                '.'
            };
            write!(out, "{c}")?;
        }
        writeln!(out)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// The component-wise minimum of `self` and `rhs`.
    pub fn min(self, rhs: Self) -> Self {
        Self {
            x: self.x.min(rhs.x),
            y: self.y.min(rhs.y),
        }
    }

    /// The component-wise maximum of `self` and `rhs`.
    pub fn max(self, rhs: Self) -> Self {
        Self {
            x: self.x.max(rhs.x),
            y: self.y.max(rhs.y),
        }
    }
}

impl Add<Direction> for Position {
    type Output = Self;

    fn add(mut self, rhs: Direction) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<Direction> for Position {
    fn add_assign(&mut self, rhs: Direction) {
        match rhs {
            Direction::Up => self.y -= 1,
            Direction::Right => self.x += 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<{}, {}>", self.x, self.y)
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    const STEPS: [Direction; 4] = [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ];
    let (start, end) = input
        .lines()
        .enumerate()
        .filter_map(|(row, line)| {
            line.map(|line| {
                line.bytes().enumerate().fold(None, |pois, (col, b)| {
                    let (start, end) = pois.unwrap_or((None, None));
                    match b {
                        b'#' => Some((
                            start,
                            Some(Position {
                                y: row as _,
                                x: col as _,
                            }),
                        )),
                        b'@' => Some((
                            Some(Position {
                                y: row as _,
                                x: col as _,
                            }),
                            end,
                        )),
                        _ if start.is_some() || end.is_some() => Some((start, end)),
                        _ => None,
                    }
                })
            })
            .transpose()
        })
        .try_fold((None, None), |(start, end), line| {
            line.map(|(line_start, line_end)| (start.or(line_start), end.or(line_end)))
        })?;
    let start = start
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing start position"))?;
    let end =
        end.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing end position"))?;
    let mut position = start;
    let mut path = vec![position];
    for i in 0.. {
        let next = position + STEPS[i % STEPS.len()];
        if next == end {
            return Ok(path.len());
        }
        if !path.contains(&next) {
            path.push(next);
            position = next;
        }
    }
    panic!("usize too small")
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    const STEPS: [Direction; 4] = [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ];
    let (start, bone) = input
        .lines()
        .enumerate()
        .filter_map(|(row, line)| {
            line.map(|line| {
                line.bytes().enumerate().fold(None, |pois, (col, b)| {
                    let (start, end) = pois.unwrap_or((None, None));
                    match b {
                        b'#' => Some((
                            start,
                            Some(Position {
                                y: row as _,
                                x: col as _,
                            }),
                        )),
                        b'@' => Some((
                            Some(Position {
                                y: row as _,
                                x: col as _,
                            }),
                            end,
                        )),
                        _ if start.is_some() || end.is_some() => Some((start, end)),
                        _ => None,
                    }
                })
            })
            .transpose()
        })
        .try_fold((None, None), |(start, end), line| {
            line.map(|(line_start, line_end)| (start.or(line_start), end.or(line_end)))
        })?;
    let start = start
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing start position"))?;
    let bone =
        bone.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing end position"))?;
    let mut position = start;
    let mut path = vec![position];
    let mut enclosed = vec![];
    let mut min = position;
    let mut max = position;
    for i in 0.. {
        let next = position + STEPS[i % STEPS.len()];
        if next == bone || path.contains(&next) || enclosed.contains(&next) {
            continue;
        }
        path.push(next);
        position = next;
        min = min.min(next);
        max = max.max(next);
        'flood_fill: for flood_root in STEPS.iter().map(|&direction| next + direction) {
            let mut visited = vec![];
            let mut frontier = vec![flood_root];
            while let Some(next) = frontier.pop() {
                visited.push(next);
                let neighbors = iter::repeat_n(next, STEPS.len())
                    .zip(STEPS)
                    .map(|(neighbor, direction)| neighbor + direction)
                    .filter(|neighbor| !path.contains(neighbor) && *neighbor != bone)
                    .filter(|neighbor| !visited.contains(neighbor) && !frontier.contains(neighbor))
                    .collect::<Vec<_>>();
                if neighbors.iter().any(|neighbor| {
                    neighbor.x <= min.x
                        || max.x <= neighbor.x
                        || neighbor.y <= min.y
                        || max.y <= neighbor.y
                }) {
                    continue 'flood_fill;
                }
                frontier.extend(neighbors);
            }
            enclosed.extend(visited);
        }
        if STEPS
            .iter()
            .map(|&direction| bone + direction)
            .all(|bone_neighbor| path.contains(&bone_neighbor) || enclosed.contains(&bone_neighbor))
        {
            return Ok(path.len() - 1);
        }
    }
    panic!("usize too small")
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    const STEPS: [Direction; 4] = [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ];
    let (start, bones) = input
        .lines()
        .enumerate()
        .filter_map(|(row, line)| {
            line.map(|line| {
                line.bytes().enumerate().fold(None, |pois, (col, b)| {
                    let (start, mut bones) = pois.unwrap_or((None, vec![]));
                    match b {
                        b'#' => {
                            bones.push(Position {
                                y: row as _,
                                x: col as _,
                            });
                            Some((start, bones))
                        }
                        b'@' => Some((
                            Some(Position {
                                y: row as _,
                                x: col as _,
                            }),
                            bones,
                        )),
                        _ if start.is_some() || !bones.is_empty() => Some((start, bones)),
                        _ => None,
                    }
                })
            })
            .transpose()
        })
        .try_fold((None, vec![]), |(start, mut end), line| {
            line.map(|(line_start, line_end)| {
                end.extend(line_end);
                (start.or(line_start), end)
            })
        })?;
    let start = start
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing start position"))?;
    let mut position = start;
    let mut path = vec![position];
    let (mut min, mut max) = bones
        .iter()
        .copied()
        .fold((position, position), |(min, max), bone| {
            (min.min(bone), max.max(bone))
        });
    let mut enclosed = vec![];
    {
        let mut escapees = (min.y..=max.y)
            .flat_map(|y| [Position { y, x: min.x }, Position { y, x: max.x }])
            .chain(
                (min.x..=max.x).flat_map(|x| [Position { x, y: min.y }, Position { x, y: max.y }]),
            )
            .collect::<Vec<_>>();
        for y in min.y..max.y {
            'row: for x in min.x..max.x {
                let here = Position { x, y };
                if escapees.contains(&here) {
                    continue;
                }
                let mut visited = vec![];
                let mut frontier = vec![here];
                while let Some(next) = frontier.pop() {
                    visited.push(next);
                    let neighbors = STEPS
                        .iter()
                        .map(|&direction| next + direction)
                        .filter(|neighbor| !bones.contains(neighbor))
                        .filter(|neighbor| !visited.contains(neighbor))
                        .filter(|neighbor| !frontier.contains(neighbor))
                        .collect::<Vec<_>>();
                    if neighbors.iter().any(|neighbor| escapees.contains(neighbor)) {
                        escapees.extend(visited);
                        escapees.extend(frontier);
                        continue 'row;
                    }
                    frontier.extend(neighbors);
                }
                enclosed.extend(visited);
            }
        }
    }
    for i in 0.. {
        let next = position + STEPS[(i / 3) % STEPS.len()];
        if bones.contains(&next) || path.contains(&next) || enclosed.contains(&next) {
            continue;
        }
        min = min.min(next);
        max = max.max(next);
        path.push(next);
        position = next;
        'flood_fill: for flood_root in STEPS
            .iter()
            .map(|&direction| next + direction)
            .filter(|neighbor| !bones.contains(neighbor))
        {
            let mut visited = vec![];
            let mut frontier = vec![flood_root];
            while let Some(next) = frontier.pop() {
                visited.push(next);
                let neighbors = iter::repeat_n(next, STEPS.len())
                    .zip(STEPS)
                    .map(|(neighbor, direction)| neighbor + direction)
                    .filter(|neighbor| !path.contains(neighbor) && !bones.contains(neighbor))
                    .filter(|neighbor| !visited.contains(neighbor) && !frontier.contains(neighbor))
                    .collect::<Vec<_>>();
                if neighbors.iter().any(|neighbor| {
                    neighbor.x <= min.x
                        || max.x <= neighbor.x
                        || neighbor.y <= min.y
                        || max.y <= neighbor.y
                }) {
                    continue 'flood_fill;
                }
                frontier.extend(neighbors);
            }
            enclosed.extend(visited);
        }
        if bones.iter().all(|&bone| {
            STEPS
                .iter()
                .map(|&direction| bone + direction)
                .all(|bone_neighbor| {
                    path.contains(&bone_neighbor)
                        || enclosed.contains(&bone_neighbor)
                        || bones.contains(&bone_neighbor)
                })
        }) {
            return Ok(path.len() - 1);
        }
    }
    panic!("usize too small")
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Melody Made of Code Quest 2 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "melody-made-of-code_02-1.txt"
            )?))?
        );
    }
    {
        println!("Melody Made of Code Quest 2 Part 2");
        let start = Instant::now();
        let result = part2(&mut BufReader::new(File::open(
            "melody-made-of-code_02-2.txt",
        )?))?;
        eprintln!("Result computed in {:?}", Instant::now() - start);
        println!("{}", result);
    }
    {
        println!("Melody Made of Code Quest 2 Part 3");
        let start = Instant::now();
        let result = part3(&mut BufReader::new(File::open(
            "melody-made-of-code_02-3.txt",
        )?))?;
        eprintln!("Result computed in {:?}", Instant::now() - start);
        println!("{}", result);
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
            ".......\n",
            ".......\n",
            ".......\n",
            ".#.@...\n",
            ".......\n",
            ".......\n",
            ".......\n",
        );
        let expected = 12;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            ".......\n",
            ".......\n",
            ".......\n",
            ".#.@...\n",
            ".......\n",
            ".......\n",
            ".......\n",
        );
        let expected = 47;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_a() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            ".......\n",
            ".......\n",
            ".......\n",
            ".#.@...\n",
            ".......\n",
            ".......\n",
            ".......\n",
        );
        let expected = 87;
        let actual = part3(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_b() -> io::Result<()> {
        const TEST_DATA_2: &str = concat!(
            "#..#.......#...\n",
            "...#...........\n",
            "...#...........\n",
            "#######........\n",
            "...#....#######\n",
            "...#...@...#...\n",
            "...#.......#...\n",
            "...........#...\n",
            "...........#...\n",
            "#..........#...\n",
            "##......#######\n",
        );
        let expected = 239;
        let actual = part3(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_c() -> io::Result<()> {
        const TEST_DATA_3: &str = concat!(
            "................................................................\n",
            ".........................###.........###........................\n",
            "....................##...###########.#####......#.......###.....\n",
            ".........##.............############....####.............##.....\n",
            ".......######..............#############.###....................\n",
            ".........##................#############.###.......##...........\n",
            "...............##...........########....####....................\n",
            "...............................####.#######...........##........\n",
            "........................##################...........####.......\n",
            "....#.........#########################.....##......######......\n",
            "..............#.##......##....##..##.##...............##........\n",
            "..............................##....##..........##..............\n",
            "........####....#################..######...................##..\n",
            "........###.....###...####..###..##...##.########...............\n",
            ".................####....###..##.##.##..###....##.....##........\n",
            "....##...........#######.....##..##..##......#####..........#...\n",
            "...........##......#########......#....##.######..........#####.\n",
            "...........##........###########################....#.......#...\n",
            ".........######............##################.......#...........\n",
            "...........##.............#########.............................\n",
            "............#.........#############....................#........\n",
            ".....#...........##..####......###......##........#.............\n",
            ".............##................###..........#.....#.............\n",
            "..................##...........##...................##..........\n",
            "..........................###.####.####.........................\n",
            "................#.###########..###.############.#...............\n",
            ".....#####....###...............................###.............\n",
            ".....#####...#############......@......#############............\n",
            ".....#########.###################################.#............\n",
            "...###########..##.....###################.....##..##...........\n",
            "...######...#######.##...###.........##...##...###.##...........\n",
            ".....##.########........#####..###..####.......#.########.......\n",
            "............#########################################...........\n",
            "..............#####################################.............\n",
            "...............................###..............................\n",
            "................................................................\n",
        );
        let expected = 1539;
        let actual = part3(&mut Cursor::new(TEST_DATA_3))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
