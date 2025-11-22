use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead, BufReader, Write},
    iter,
    ops::{Add, AddAssign, Div, DivAssign, Mul, Neg, Sub, SubAssign},
};

use dependencies::priority_queue::PriorityQueue;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    pub fn manhattan_distance(&self, other: &Self) -> usize {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    pub fn neighbors(&self) -> impl IntoIterator<Item = Position> {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
        .into_iter()
        .map(|direction| *self + 1 * direction)
    }

    /// Computes the component-wise minimum of `self` and `other`.
    pub fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    /// Computes the component-wise maximum of `self` and `other`.
    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

impl Add for Position {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign for Position {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<Position> for isize {
    type Output = Position;

    fn mul(self, mut rhs: Position) -> Self::Output {
        rhs.x *= self;
        rhs.y *= self;
        rhs
    }
}

impl Neg for Position {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.x = -self.x;
        self.y = -self.y;
        self
    }
}

impl Div<isize> for Position {
    type Output = Self;

    fn div(mut self, rhs: isize) -> Self::Output {
        self /= rhs;
        self
    }
}

impl DivAssign<isize> for Position {
    fn div_assign(&mut self, rhs: isize) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Instruction {
    Left(usize),
    Right(usize),
}

impl Instruction {
    fn apply(
        self,
        start: Position,
        facing: Direction,
    ) -> (Direction, impl IntoIterator<Item = Position> + Clone) {
        let (distance, facing) = match self {
            Self::Left(distance) => (distance, facing.rotate_counterclockwise()),
            Self::Right(distance) => (distance, facing.rotate_clockwise()),
        };
        (
            facing,
            iter::successors(Some(start), move |&curr| Some(curr + 1 * facing))
                .skip(1)
                .take(distance),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn rotate_clockwise(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    fn rotate_counterclockwise(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        }
    }
}

impl Mul<Direction> for usize {
    type Output = Position;

    fn mul(self, rhs: Direction) -> Self::Output {
        let signed_self = self
            .try_into()
            .unwrap_or_else(|e| panic!("{e:?}: Overflow when converting {self} to signed"));
        match rhs {
            Direction::North => Position {
                x: 0,
                y: signed_self,
            },
            Direction::East => Position {
                x: signed_self,
                y: 0,
            },
            Direction::South => Position {
                x: 0,
                y: -signed_self,
            },
            Direction::West => Position {
                x: -signed_self,
                y: 0,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq)]
struct Weight {
    steps_taken: usize,
    steps_remaining: usize,
}

impl From<Weight> for usize {
    fn from(value: Weight) -> Self {
        value.steps_taken.saturating_add(value.steps_remaining)
    }
}

impl Ord for Weight {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        usize::from(*self).cmp(&usize::from(*other))
    }
}

impl PartialEq for Weight {
    fn eq(&self, other: &Self) -> bool {
        usize::from(*self) == usize::from(*other)
    }
}

impl PartialOrd for Weight {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "write_images")]
fn write_image(
    mut out: impl Write,
    mut in_wall: impl FnMut(Position) -> bool,
    start: Position,
    goal: Position,
    min: Position,
    max: Position,
) -> io::Result<()> {
    const RED: [u8; 3] = [0xFF, 0x00, 0x00];
    const GOLD: [u8; 3] = [0xFF, 0xD7, 0x00];
    const BROWN: [u8; 3] = [0x96, 0x4B, 0x00];
    const BLACK: [u8; 3] = [0x00, 0x00, 0x00];
    const WHITE: [u8; 3] = [0xFF, 0xFF, 0xFF];
    writeln!(out, "P6")?;
    writeln!(out, "{} {}", max.x - min.x + 1, max.y - min.y + 1)?;
    writeln!(out, "255")?;
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            let here = Position { x, y };
            let color = if here == start {
                RED
            } else if here == goal {
                GOLD
            } else if in_wall(here) {
                BROWN
            } else if (here.x + here.y) % 2 == 0 {
                BLACK
            } else {
                WHITE
            };
            out.write_all(&color)?;
        }
    }
    Ok(())
}

#[cfg(not(feature = "write_images"))]
fn write_image(
    _out: impl Write,
    _in_wall: impl FnMut(Position) -> bool,
    _start: Position,
    _goal: Position,
    _min: Position,
    _max: Position,
) -> io::Result<()> {
    Ok(())
}

fn read_instructions(input: &mut dyn BufRead) -> io::Result<Vec<Instruction>> {
    input
        .lines()
        .next()
        .unwrap_or_else(|| {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Can't find notes",
            ))
        })?
        .split(',')
        .map(|instruction| {
            let (direction, distance) = instruction.split_at(1);
            let distance = distance.parse::<usize>().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{e:?}: {distance:?} is not a number"),
                )
            })?;
            match direction {
                "L" => Ok(Instruction::Left(distance)),
                "R" => Ok(Instruction::Right(distance)),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Can't parse notes: {direction:?} is not a direction",
                )),
            }
        })
        .collect::<io::Result<Vec<_>>>()
}

fn flood_fill<F, Fx, Fy>(
    mut is_wall: F,
    origin: Position,
    min: Position,
    mut decompress_x: Fx,
    mut decompress_y: Fy,
    goal: Position,
) -> Option<usize>
where
    F: FnMut(&Position) -> bool,
    Fx: FnMut(usize) -> Option<isize>,
    Fy: FnMut(usize) -> Option<isize>,
{
    let mut visited = HashSet::new();
    let mut frontier = PriorityQueue::new();
    frontier.push(
        origin,
        Reverse(Weight {
            steps_taken: 0,
            steps_remaining: goal.manhattan_distance(&origin),
        }),
    );
    while let Some((next, Reverse(priority))) = frontier.pop() {
        if next == goal {
            assert_eq!(
                priority.steps_remaining, 0,
                "{next:?}=={goal:?} with non-zero manhattan distance",
            );
            return Some(priority.steps_taken);
        }
        visited.insert(next);
        next.neighbors()
            .into_iter()
            .filter(|neighbor| !visited.contains(neighbor) && !is_wall(neighbor))
            .for_each(|neighbor| {
                let uncompressed_distance = if next.y == neighbor.y {
                    let Some(next_x) = decompress_x(next.x.abs_diff(min.x) + 1) else {
                        return;
                    };
                    let Some(neighbor_x) = decompress_x(neighbor.x.abs_diff(min.x) + 1) else {
                        return;
                    };
                    next_x.abs_diff(neighbor_x)
                } else {
                    let Some(next_y) = decompress_y(next.y.abs_diff(min.y) + 1) else {
                        return;
                    };
                    let Some(neighbor_y) = decompress_y(neighbor.y.abs_diff(min.y) + 1) else {
                        return;
                    };
                    next_y.abs_diff(neighbor_y)
                };
                frontier.push_increase(
                    neighbor,
                    Reverse(Weight {
                        steps_taken: priority.steps_taken + uncompressed_distance,
                        steps_remaining: neighbor.manhattan_distance(&goal),
                    }),
                );
            });
    }
    None
}

fn solve_compressed<I, Fx, Fy>(
    instructions: I,
    decompress_x: Fx,
    decompress_y: Fy,
    image_out: impl Write,
) -> Option<usize>
where
    I: IntoIterator<Item = Instruction>,
    Fx: FnMut(usize) -> Option<isize>,
    Fy: FnMut(usize) -> Option<isize>,
{
    let mut facing = Direction::North;
    let mut wall = HashSet::new();
    let wall_start = Position { x: 0, y: 0 };
    let mut goal = wall_start;
    let mut min = wall_start;
    let mut max = wall_start;
    for instruction in instructions {
        // Loop invariant: `wall` consists of every `Position` that has ever been replaced as
        // `goal`.
        let (new_facing, new_walls) = instruction.apply(goal, facing);
        facing = new_facing;
        for next_wall in new_walls {
            min = min.min(next_wall);
            max = max.max(next_wall);
            wall.insert(std::mem::replace(&mut goal, next_wall));
        }
    }
    wall.remove(&goal);
    eprintln!("Compressed grid is {}x{}", max.x - min.x, max.y - min.y);
    let _ = write_image(image_out, |p| wall.contains(&p), wall_start, goal, min, max);
    flood_fill(
        |p| wall.contains(p),
        wall_start,
        min,
        decompress_x,
        decompress_y,
        goal,
    )
}

fn compress_coordinates(real: &mut Vec<isize>) -> HashMap<isize, usize> {
    real.sort();
    real.dedup();
    real.iter()
        .copied()
        .enumerate()
        .map(|(idx, value)| (value, idx))
        .collect()
}

fn compress_instructions<'v, I>(
    vertices: &'v [Position],
    instructions: I,
    mut compress_x: impl FnMut(isize) -> usize + 'v,
    mut compress_y: impl FnMut(isize) -> usize + 'v,
) -> impl IntoIterator<Item = Instruction>
where
    I: IntoIterator<Item = Instruction>,
{
    vertices
        .windows(2)
        .zip(instructions)
        .map(move |(edge, instruction)| {
            let distance = if edge[0].y == edge[1].y {
                compress_x(edge[1].x).abs_diff(compress_x(edge[0].x))
            } else {
                compress_y(edge[1].y).abs_diff(compress_y(edge[0].y))
            };
            match instruction {
                Instruction::Left(_) => Instruction::Left(distance),
                Instruction::Right(_) => Instruction::Right(distance),
            }
        })
}

fn part1(input: &mut dyn BufRead, image_out: impl Write) -> io::Result<usize> {
    let instructions = read_instructions(input)?;
    let stair_position = Position { x: 0, y: 0 };
    let mut vertices = vec![stair_position];
    let mut xs = vec![0];
    let mut ys = vec![0];
    instructions.iter().copied().fold(
        (Direction::North, stair_position),
        |(facing, position), instruction| {
            let (distance, facing) = match instruction {
                Instruction::Left(distance) => (distance, facing.rotate_counterclockwise()),
                Instruction::Right(distance) => (distance, facing.rotate_clockwise()),
            };
            let new_position = position + distance * facing;
            vertices.push(new_position);
            if new_position.y == position.y {
                xs.extend_from_slice(&[new_position.x, (new_position + 1 * facing).x]);
                ys.extend_from_slice(&[new_position.y - 1, new_position.y + 1]);
            } else {
                xs.extend_from_slice(&[new_position.x - 1, new_position.x + 1]);
                ys.extend_from_slice(&[new_position.y, (new_position + 1 * facing).y]);
            }
            (facing, new_position)
        },
    );
    assert_eq!(vertices.len(), instructions.len() + 1);
    let compressed_xs = compress_coordinates(&mut xs);
    let compressed_ys = compress_coordinates(&mut ys);
    eprintln!(
        "Real grid is {}x{}",
        xs.last().unwrap() - xs[0] - 1,
        ys.last().unwrap() - ys[0] - 1,
    );
    let instructions = compress_instructions(
        &vertices,
        instructions,
        |x| compressed_xs[&x],
        |y| compressed_ys[&y],
    );
    solve_compressed(
        instructions,
        |x| xs.get(x).copied(),
        |y| ys.get(y).copied(),
        image_out,
    )
    .ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Notes describe loop that separates start from end",
        )
    })
}

fn part2(input: &mut dyn BufRead, image_out: impl Write) -> io::Result<usize> {
    part1(input, image_out)
}

fn part3(input: &mut dyn BufRead, image_out: impl Write) -> io::Result<usize> {
    part1(input, image_out)
}

pub(super) fn run() -> io::Result<()> {
    {
        #[cfg(not(feature = "write_images"))]
        let image = io::sink();
        #[cfg(feature = "write_images")]
        let image = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open("SoDaD_15_1.ppm")?;
        println!("Song of Ducks and Dragons Quest 15 Part 1");
        println!(
            "{}",
            part1(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_15-1.txt")?),
                image,
            )?
        );
    }
    {
        #[cfg(not(feature = "write_images"))]
        let image = io::sink();
        #[cfg(feature = "write_images")]
        let image = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open("SoDaD_15_2.ppm")?;
        println!("Song of Ducks and Dragons Quest 15 Part 2");
        println!(
            "{}",
            part2(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_15-2.txt")?),
                image,
            )?
        );
    }
    {
        #[cfg(not(feature = "write_images"))]
        let image = io::sink();
        #[cfg(feature = "write_images")]
        let image = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open("SoDaD_15_3.ppm")?;
        println!("Song of Ducks and Dragons Quest 15 Part 3");
        println!(
            "{}",
            part3(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_15-3.txt")?),
                image
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
    fn test_position_min() {
        assert_eq!(
            Position { x: 3, y: 0 }.min(Position { x: 0, y: 3 }),
            Position { x: 0, y: 0 }
        );
    }

    #[test]
    fn test_position_max() {
        assert_eq!(
            Position { x: 3, y: 0 }.max(Position { x: 0, y: 3 }),
            Position { x: 3, y: 3 }
        );
    }

    #[test]
    fn test_part1() -> io::Result<()> {
        const TEST_DATA_1: &str = "R3,R4,L3,L4,R3,R6,R9\n";
        let expected = 6;
        let actual = part1(&mut Cursor::new(TEST_DATA_1), io::sink())?;
        assert_eq!(expected, actual);
        const TEST_DATA_2: &str = "L6,L3,L6,R3,L6,L3,L3,R6,L6,R6,L6,L6,R3,L3,L3,R3,R3,L6,L6,L3\n";
        let expected = 16;
        let actual = part1(&mut Cursor::new(TEST_DATA_2), io::sink())?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
