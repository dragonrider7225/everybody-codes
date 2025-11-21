use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    iter,
    time::Instant,
};

#[cfg(feature = "write_images")]
use std::io::Write;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tile {
    Inactive,
    Active,
}

impl Tile {
    fn next_state(&self, num_active_diagonal_neighbors: usize) -> Self {
        let num_active = num_active_diagonal_neighbors + usize::from(*self);
        if num_active.is_multiple_of(2) {
            Self::Active
        } else {
            Self::Inactive
        }
    }
}

impl From<Tile> for usize {
    fn from(value: Tile) -> Self {
        match value {
            Tile::Inactive => 0,
            Tile::Active => 1,
        }
    }
}

impl TryFrom<char> for Tile {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Inactive),
            '#' => Ok(Self::Active),
            _ => Err(format!("Invalid tile: {value:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Position {
    col: usize,
    row: usize,
}

impl Position {
    pub fn checked_add(self, rhs: DeltaPosition) -> Option<Self> {
        Some(Self {
            col: self.col.checked_add_signed(rhs.col)?,
            row: self.row.checked_add_signed(rhs.row)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DeltaPosition {
    col: isize,
    row: isize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Grid(Vec<Vec<Tile>>);

impl Grid {
    pub fn read(input: &mut dyn BufRead) -> io::Result<Self> {
        input
            .lines()
            .take_while(|line| !line.as_ref().is_ok_and(|line| line.is_empty()))
            .map(|line| {
                line.and_then(|line| {
                    line.chars()
                        .map(Tile::try_from)
                        .map(|tile| {
                            tile.map_err(|e| {
                                io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("{e:?}: {line:?} is not a valid grid line"),
                                )
                            })
                        })
                        .collect::<io::Result<Vec<_>>>()
                })
            })
            .collect::<io::Result<Vec<_>>>()
            .map(Self)
    }

    fn diagonal_neighbors(&self, pos: Position) -> impl IntoIterator<Item = Position> {
        let width = self.0[0].len();
        let height = self.0.len();
        [
            DeltaPosition { row: -1, col: -1 },
            DeltaPosition { row: -1, col: 1 },
            DeltaPosition { row: 1, col: -1 },
            DeltaPosition { row: 1, col: 1 },
        ]
        .into_iter()
        .filter_map(move |delta| pos.checked_add(delta))
        .filter(move |pos| pos.row < height && pos.col < width)
    }

    pub fn next_step(&self) -> Self {
        Self(
            self.0
                .iter()
                .enumerate()
                .map(|(row_idx, row)| {
                    row.iter()
                        .enumerate()
                        .map(|(col_idx, tile)| {
                            let num_neighbors = self
                                .diagonal_neighbors(Position {
                                    col: col_idx,
                                    row: row_idx,
                                })
                                .into_iter()
                                .filter(|pos| matches!(self.0[pos.row][pos.col], Tile::Active))
                                .count();
                            tile.next_state(num_neighbors)
                        })
                        .collect()
                })
                .collect(),
        )
    }

    pub fn num_active(&self) -> usize {
        self.0
            .iter()
            .flatten()
            .filter(|tile| matches!(tile, Tile::Active))
            .count()
    }

    #[cfg(feature = "write_images")]
    pub fn write_image(&self, mut out: impl Write) -> io::Result<()> {
        let height = self.0.len();
        let width = self.0[0].len();
        writeln!(out, "P6 {width} {height} 1")?;
        self.0.iter().try_for_each(|row| {
            row.iter().try_for_each(|tile| match tile {
                Tile::Active => write!(out, "\x01\0\0"),
                Tile::Inactive => write!(out, "\0\0\0"),
            })
        })?;
        writeln!(out)
    }
}

#[cfg(test)]
macro_rules! tile {
    (#) => {
        Tile::Active
    };
    (1) => {
        Tile::Active
    };
    (.) => {
        Tile::Inactive
    };
    (0) => {
        Tile::Inactive
    };
}

#[cfg(test)]
macro_rules! grid {
    ($([$($tile:tt)*]),* $(,)?) => {
        Grid(vec![$(vec![$(tile!($tile)),*]),*])
    }
}

fn part1(input: &mut dyn BufRead, num_rounds: usize) -> io::Result<usize> {
    let grid = Grid::read(input)?;
    Ok(iter::successors(Some(grid), |grid| Some(grid.next_step()))
        .skip(1)
        .take(num_rounds)
        .map(|grid| grid.num_active())
        .sum())
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    part1(input, 2025)
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    /// This fixes the value of `CYCLE_LEN`. At a size of 34, the first step from an empty grid
    /// repeats after an additional 4095 steps, so we don't need to do any special-casing of early
    /// steps.
    const GRID_SIZE: usize = 34;
    /// This is fixed by `GRID_SIZE`.
    const CYCLE_LEN: usize = 4095;
    const NUM_ROUNDS: usize = 1_000_000_000;

    let target = Grid::read(input)?.0;
    let grid = Grid(vec![vec![Tile::Inactive; GRID_SIZE]; GRID_SIZE]);
    let start = Instant::now();
    let grid_anchor = Position {
        row: (GRID_SIZE - target.len()) / 2,
        col: (GRID_SIZE - target[0].len()) / 2,
    };
    let center_matches = |grid: &Grid| {
        grid.0[grid_anchor.row..]
            .iter()
            .zip(&target)
            .all(|(grid_row, target_row)| {
                grid_row[grid_anchor.col..]
                    .iter()
                    .zip(target_row)
                    .all(|(grid_tile, target_tile)| grid_tile == target_tile)
            })
    };
    #[cfg(not(feature = "write_images"))]
    let grid_inspection = |_: &(usize, Grid)| {};
    #[cfg(feature = "write_images")]
    let grid_inspection = |(num_steps, grid): &(usize, Grid)| {
        let Ok(out) = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("SoDaD_14_3_images/{num_steps}.ppm"))
        else {
            eprintln!("Couldn't open image file");
            return;
        };
        let _ = grid.write_image(out);
    };
    let tiles_per_cycle = iter::successors(Some(grid.next_step()), |grid| Some(grid.next_step()))
        .take((NUM_ROUNDS - 1).min(CYCLE_LEN))
        .enumerate()
        .inspect(grid_inspection)
        .map(|(_, grid)| grid)
        .fold(0, |num_tiles, step| {
            num_tiles
                + if center_matches(&step) {
                    step.num_active()
                } else {
                    0
                }
        });
    eprintln!(
        "Calculating cycle {}took {duration:?}",
        if cfg!(feature = "write_images") {
            "and writing images "
        } else {
            ""
        },
        duration = Instant::now() - start,
    );
    let num_cycles = NUM_ROUNDS / CYCLE_LEN;
    let remainder = NUM_ROUNDS % CYCLE_LEN;
    let ret = num_cycles * tiles_per_cycle
        + iter::successors(Some(grid), |grid| Some(grid.next_step()))
            .take(remainder)
            .filter(center_matches)
            .map(|grid| grid.num_active())
            .sum::<usize>();
    eprintln!("Computation took {:?}", Instant::now() - start);
    Ok(ret)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 14 Part 1");
        println!(
            "{}",
            part1(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_14-1.txt")?),
                10
            )?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 14 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_14-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 14 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_14-3.txt"
            )?))?
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, io::Cursor};

    use super::*;

    #[test]
    fn test_grid_neighbors() {
        let grid = Grid(vec![vec![Tile::Inactive; 4]; 4]);
        let expected = [
            Position { row: 0, col: 0 },
            Position { row: 2, col: 0 },
            Position { row: 0, col: 2 },
            Position { row: 2, col: 2 },
        ]
        .into_iter()
        .collect::<HashSet<_>>();
        let actual = grid
            .diagonal_neighbors(Position { col: 1, row: 1 })
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(expected, actual);
        let expected = [Position { row: 1, col: 0 }, Position { row: 1, col: 2 }]
            .into_iter()
            .collect::<HashSet<_>>();
        let actual = grid
            .diagonal_neighbors(Position { col: 1, row: 0 })
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(expected, actual);
        let expected = [Position { row: 0, col: 1 }, Position { row: 2, col: 1 }]
            .into_iter()
            .collect::<HashSet<_>>();
        let actual = grid
            .diagonal_neighbors(Position { col: 0, row: 1 })
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(expected, actual);
        let expected = [Position { row: 1, col: 1 }]
            .into_iter()
            .collect::<HashSet<_>>();
        let actual = grid
            .diagonal_neighbors(Position { col: 0, row: 0 })
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(expected, actual);
        let expected = [Position { row: 0, col: 2 }, Position { row: 2, col: 2 }]
            .into_iter()
            .collect::<HashSet<_>>();
        let actual = grid
            .diagonal_neighbors(Position { col: 3, row: 1 })
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(expected, actual);
        let expected = [Position { row: 2, col: 0 }, Position { row: 2, col: 2 }]
            .into_iter()
            .collect::<HashSet<_>>();
        let actual = grid
            .diagonal_neighbors(Position { col: 1, row: 3 })
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(expected, actual);
        let expected = [Position { row: 2, col: 2 }]
            .into_iter()
            .collect::<HashSet<_>>();
        let actual = grid
            .diagonal_neighbors(Position { col: 3, row: 3 })
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_grid_step() {
        let grid = grid![
            [. # . # # .],
            [# # . . # .],
            [. . # # . #],
            [. # . # # .],
            [. # # # . .],
            [# # # . # #],
        ];
        let expected = grid![
            [. # . # . .],
            [# # . # # .],
            [# . # . . .],
            [. . . . # #],
            [# . # # # #],
            [# # . . # .],
        ];
        let actual = grid.next_step();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_part1() -> io::Result<()> {
        const TEST_DATA: &str = ".#.##.\n##..#.\n..##.#\n.#.##.\n.###..\n###.##\n";
        let expected = 200;
        let actual = part1(&mut Cursor::new(TEST_DATA), 10)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "#......#\n",
            "..#..#..\n",
            ".##..##.\n",
            "...##...\n",
            "...##...\n",
            ".##..##.\n",
            "..#..#..\n",
            "#......#\n",
        );
        let expected = 278_388_552;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
