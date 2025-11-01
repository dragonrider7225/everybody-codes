use std::{
    fmt::{self, Debug, Formatter},
    fs::File,
    io::{self, BufRead, BufReader, Cursor, Read},
    iter,
    ops::{Add, AddAssign, Sub, SubAssign},
};

#[derive(Clone, Copy, Eq, PartialEq)]
struct DevicePower(pub u32);

impl DevicePower {
    pub const fn new() -> Self {
        Self(10)
    }
}

impl Debug for DevicePower {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Default for DevicePower {
    fn default() -> Self {
        Self::new()
    }
}

impl Add<u32> for DevicePower {
    type Output = Self;

    fn add(mut self, rhs: u32) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<u32> for DevicePower {
    fn add_assign(&mut self, rhs: u32) {
        self.0 = self.0.saturating_add(rhs);
    }
}

impl Sub<u32> for DevicePower {
    type Output = Self;

    fn sub(mut self, rhs: u32) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign<u32> for DevicePower {
    fn sub_assign(&mut self, rhs: u32) {
        self.0 = self.0.saturating_sub(rhs);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Instruction {
    /// Increases the device's power by one.
    Increment,
    /// Decreases the device's power by one if possible.
    Decrement,
    /// Leaves the device's power unmodified.
    Maintain,
}

impl Instruction {
    pub fn apply(self, value: DevicePower) -> DevicePower {
        match self {
            Self::Increment => value + 1,
            Self::Decrement => value - 1,
            Self::Maintain => value,
        }
    }
}

impl TryFrom<u8> for Instruction {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'+' => Ok(Self::Increment),
            b'-' => Ok(Self::Decrement),
            b'=' => Ok(Self::Maintain),
            _ => Err(format!(
                "Invalid instruction: {:?}",
                char::from_u32(value as _).unwrap()
            )),
        }
    }
}

/// Reads all lines from `input` through the first empty line. Each non-empty line read is parsed
/// as `"$id:$($instr),*"` -> `($id, vec![$($instr),*])`.
fn read_plans(input: &mut dyn BufRead) -> io::Result<Vec<(String, Vec<Instruction>)>> {
    input
        .lines()
        .take_while(|line| !line.as_ref().is_ok_and(String::is_empty))
        .map(|line| {
            let line = line?;
            let (id, instructions) = line.split_once(':').ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Each plan line must contain exactly one colon",
                )
            })?;
            let instructions = instructions
                .bytes()
                .filter(|&b| b != b',')
                .map(Instruction::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "{e:?}: Each plan must consist of instructions separated by commas"
                        ),
                    )
                })?;
            Ok((id.to_string(), instructions))
        })
        .collect::<io::Result<Vec<_>>>()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PathSegment {
    /// The first segment of any path. Does not modify the current instruction.
    Start,
    /// Replaces the current instruction of the current cycle of the plan with an Increment
    /// instruction.
    Increment,
    /// Replaces the current instruction of the current cycle of the plan with a Decrement
    /// instruction.
    Decrement,
    /// Does not modify the current instruction.
    Maintain,
}

impl PathSegment {
    pub const fn apply(self, instruction: Instruction) -> Instruction {
        match self {
            Self::Increment => Instruction::Increment,
            Self::Decrement => Instruction::Decrement,
            Self::Start | Self::Maintain => instruction,
        }
    }

    pub const fn is_not_start(&self) -> bool {
        !matches!(self, Self::Start)
    }
}

impl TryFrom<u8> for PathSegment {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'S' => Ok(Self::Start),
            b'+' => Ok(Self::Increment),
            b'-' => Ok(Self::Decrement),
            b'=' => Ok(Self::Maintain),
            _ => Err(format!(
                "Invalid path segment: {:?}",
                char::from_u32(value as _).unwrap()
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tile {
    PathSegment(PathSegment),
    Impassable,
}

impl From<u8> for Tile {
    fn from(value: u8) -> Self {
        match PathSegment::try_from(value) {
            Ok(segment) => Self::PathSegment(segment),
            Err(_) => Self::Impassable,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Position {
    x: usize,
    y: usize,
}

impl Add<(Delta, Bounds)> for Position {
    type Output = Option<Self>;

    fn add(self, (delta, size): (Delta, Bounds)) -> Self::Output {
        let x = self
            .x
            .checked_add_signed(delta.x)
            .filter(|&x| x < size.width)?;
        let y = self
            .y
            .checked_add_signed(delta.y)
            .filter(|&y| y < size.height)?;
        Some(Self { x, y })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Delta {
    x: isize,
    y: isize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Bounds {
    width: usize,
    height: usize,
}

fn read_track(input: &mut dyn BufRead) -> io::Result<Vec<PathSegment>> {
    let mut lines = input
        .lines()
        .take_while(|line| !line.as_ref().is_ok_and(String::is_empty))
        .map(|line| Ok(line?.bytes().map(Tile::from).collect::<Vec<_>>()))
        .collect::<io::Result<Vec<_>>>()?;
    let width = lines.iter().map(|line| line.len()).max().unwrap();
    lines.iter_mut().for_each(|line| {
        if line.len() < width {
            line.extend(iter::repeat_n(Tile::Impassable, width - line.len()));
        }
    });
    let bounds = Bounds {
        width,
        height: lines.len(),
    };
    assert_eq!(lines[0][0], Tile::PathSegment(PathSegment::Start));
    let neighbor = |delta: Delta| move |center: Position| center + (delta, bounds);
    let neighbors = [
        neighbor(Delta { x: -1, y: 0 }),
        neighbor(Delta { x: 0, y: -1 }),
        neighbor(Delta { x: 1, y: 0 }),
        neighbor(Delta { x: 0, y: 1 }),
    ];
    let mut last_pos = Position { x: 1, y: 0 };
    let mut other_neighbor = Position { x: 0, y: 0 };
    let mut segments = vec![match lines[last_pos.y][last_pos.x] {
        Tile::Impassable => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Path must include (0, 0)",
            ))
        }
        Tile::PathSegment(segment) => segment,
    }];
    while (Position { x: 0, y: 0 }) != last_pos {
        let (neighbor, segment) = neighbors
            .iter()
            .filter_map(|neighbor| neighbor(last_pos))
            .filter(|&neighbor| neighbor != other_neighbor)
            .filter_map(|neighbor| match lines[neighbor.y][neighbor.x] {
                Tile::Impassable => None,
                Tile::PathSegment(segment) => Some((neighbor, segment)),
            })
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Path contains dead end"))?;
        other_neighbor = std::mem::replace(&mut last_pos, neighbor);
        segments.push(segment);
    }
    // lines.reverse();
    // let start = lines
    //     .pop()
    //     .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing track"))?;
    // let (start, end_rev) = lines.into_iter().rev().try_fold(
    //     (start, vec![]),
    //     |(mut start, mut end_rev), mut line| -> io::Result<_> {
    //         start.push(line.pop().ok_or_else(|| {
    //             io::Error::new(
    //                 io::ErrorKind::InvalidData,
    //                 "Found track line containing only whitespace",
    //             )
    //         })?);
    //         end_rev.extend(line);
    //         Ok((start, end_rev))
    //     },
    // )?;
    // Ok(start
    Ok(segments
        .into_iter()
        // .chain(end_rev.into_iter().rev())
        .cycle()
        .skip_while(PathSegment::is_not_start)
        .skip(1)
        .take_while(PathSegment::is_not_start)
        .chain(iter::once(PathSegment::Start))
        .collect())
}

fn run_instructions(instructions: impl IntoIterator<Item = Instruction>) -> u64 {
    instructions
        .into_iter()
        .fold(
            (0, DevicePower::new()),
            |(essence, power): (u64, _), instruction| {
                let new_power = instruction.apply(power);
                (essence.saturating_add(new_power.0 as _), new_power)
            },
        )
        .0
}

fn run_track(
    track: impl IntoIterator<Item = PathSegment> + Clone,
    laps: usize,
    instructions: impl IntoIterator<Item = Instruction, IntoIter: Clone>,
) -> u64 {
    run_instructions(
        iter::repeat_n(track, laps)
            .flatten()
            .zip(instructions.into_iter().cycle())
            .map(|(segment, instruction)| segment.apply(instruction)),
    )
}

fn part1(input: &mut dyn BufRead) -> io::Result<String> {
    let plans = read_plans(input)?;
    let mut collected_essence = plans
        .into_iter()
        .map(|(id, instructions)| {
            (
                id,
                run_instructions(instructions.into_iter().cycle().take(10)),
            )
        })
        .collect::<Vec<_>>();
    collected_essence.sort_by_key(|&(_, essence)| std::cmp::Reverse(essence));
    Ok(collected_essence.into_iter().map(|(id, _)| id).collect())
}

fn combine_plans_and_track(plans: impl BufRead, track: impl BufRead) -> impl BufRead {
    plans.chain(Cursor::new("\n")).chain(track)
}

fn part2(input: &mut dyn BufRead) -> io::Result<String> {
    let plans = read_plans(input)?;
    let track = read_track(input)?;
    let mut essences = plans
        .into_iter()
        .map(|(id, instructions)| (id, run_track(track.iter().copied(), 10, instructions)))
        .collect::<Vec<_>>();
    essences.sort_by_key(|&(_, essence)| std::cmp::Reverse(essence));
    Ok(essences.into_iter().map(|(id, _)| id).collect())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AllPlans {
    maintain_positions: [usize; 3],
    increment_positions: u8,
}

impl AllPlans {
    pub const fn new() -> Self {
        Self {
            maintain_positions: [0, 1, 2],
            increment_positions: 0x1F,
        }
    }
}

impl Default for AllPlans {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for AllPlans {
    type Item = [Instruction; 11];

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_increment_positions = self.increment_positions.checked_add(1)?;
        while next_increment_positions.count_ones() != 5 {
            next_increment_positions = match next_increment_positions.checked_add(1) {
                None => break,
                Some(n) => n,
            };
        }
        let mut ret = [Instruction::Decrement; 11];
        self.maintain_positions
            .iter()
            .copied()
            .for_each(|pos| ret[pos] = Instruction::Maintain);
        (0..(u8::BITS as _)).fold(0, |mut acc, i| {
            let is_increment = (self.increment_positions & (1 << i)) != 0;
            while acc < 3 && self.maintain_positions[acc] == i + acc {
                acc += 1;
            }
            if is_increment {
                ret[i + acc] = Instruction::Increment;
            }
            acc
        });
        if next_increment_positions == 0xFF {
            next_increment_positions = 0x1F;
            match self.maintain_positions {
                [8, 9, 10] => next_increment_positions = 0xFF,
                [a, 9, 10] => {
                    self.maintain_positions[0] = a + 1;
                    self.maintain_positions[1] = a + 2;
                    self.maintain_positions[2] = a + 3;
                }
                [_, b, 10] => {
                    self.maintain_positions[1] = b + 1;
                    self.maintain_positions[2] = b + 2;
                }
                [.., c] => self.maintain_positions[2] = c + 1,
            }
        }
        self.increment_positions = next_increment_positions;
        Some(ret)
    }
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let rival_plan = read_plans(input)?.swap_remove(0).1;
    let track = read_track(input)?;
    let rival_start = std::time::Instant::now();
    let rival_essence = run_track(track.iter().copied(), 2024, rival_plan);
    let rival_end = std::time::Instant::now();
    let rival_duration = rival_end - rival_start;
    let num_plans = AllPlans::new().count();
    let real_start = std::time::Instant::now();
    let mut successful = 0;
    for (tested, essence) in AllPlans::new()
        .map(|plan| run_track(track.iter().copied(), 2024, plan))
        .enumerate()
    {
        if tested % 1000 == 0 {
            eprintln!(
                "Time elapsed: {:?}. Estimated time remaining: {:?}",
                std::time::Instant::now() - real_start,
                rival_duration * (num_plans - tested) as u32,
            );
        }
        if essence > rival_essence {
            successful += 1;
        }
    }
    Ok(successful)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Algorithmia Quest 7 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open("algorithmia_07-1.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 7 Part 2");
        println!(
            "{}",
            part2(&mut combine_plans_and_track(
                BufReader::new(File::open("algorithmia_07-2.txt")?),
                BufReader::new(File::open("algorithmia_07-2-track.txt")?)
            ))?
        );
    }
    {
        println!("Algorithmia Quest 7 Part 3");
        println!(
            "{}",
            part3(
                &mut BufReader::new(File::open("algorithmia_07-3.txt")?)
                    .chain(Cursor::new("\n"))
                    .chain(BufReader::new(File::open("algorithmia_07-3-track.txt")?))
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
        const TEST_DATA: &str = "A:+,-,=,=\nB:+,=,-,+\nC:=,-,+,+\nD:=,=,=,+\n";
        let expected = "BDCA";
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = "A:+,-,=,=\nB:+,=,-,+\nC:=,-,+,+\nD:=,=,=,+\n";
        const TEST_TRACK: &str = "S+===\n-   +\n=+=-+\n";
        let expected = "DCBA";
        let actual = part2(&mut combine_plans_and_track(
            Cursor::new(TEST_DATA),
            Cursor::new(TEST_TRACK),
        ))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_all_plans() {
        let expected = 9240;
        let actual = AllPlans::new().count();
        assert_eq!(expected, actual);
    }
}
