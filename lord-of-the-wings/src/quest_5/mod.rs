use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    ops::{Add, Index, IndexMut},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Cell {
    Light,
    Dark,
    Land,
    Water,
    Sand,
    Tree,
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Light => write!(f, "0"),
            Self::Dark => write!(f, "1"),
            Self::Land => write!(f, "#"),
            Self::Water => write!(f, "."),
            Self::Sand => write!(f, "="),
            Self::Tree => write!(f, "@"),
        }
    }
}

impl TryFrom<char> for Cell {
    type Error = String;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '0' => Ok(Self::Light),
            '1' => Ok(Self::Dark),
            '#' => Ok(Self::Land),
            '.' => Ok(Self::Water),
            '=' => Ok(Self::Sand),
            '@' => Ok(Self::Tree),
            _ => Err(format!("Invalid cell {c:?}")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Map {
    cells: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
}

impl Map {
    fn convert_to_land_and_water(&mut self) {
        let mut land = HashSet::new();
        let mut to_add = HashSet::new();
        to_add.insert(Point {
            row: self.height / 2,
            column: self.width / 2,
        });
        let mut to_check = HashSet::new();
        while !to_add.is_empty() {
            for point in to_add.drain() {
                self[point] = Cell::Land;
                land.insert(point);
                to_check.extend(point.neighbors(self.width, self.height));
            }
            for point in to_check.drain() {
                if !land.contains(&point)
                    && point
                        .neighbors(self.width, self.height)
                        .filter(|&neighbor| matches!(self[neighbor], Cell::Dark | Cell::Land))
                        .count()
                        > 4
                {
                    to_add.insert(point);
                }
            }
        }
        self.cells
            .iter_mut()
            .flat_map(|row| row.iter_mut())
            .for_each(|cell| {
                if matches!(cell, Cell::Light | Cell::Dark) {
                    *cell = Cell::Water;
                }
            });
    }

    fn mark_sand(&mut self) {
        let mut to_add = HashSet::new();
        to_add.extend(
            (0..self.height)
                .flat_map(|row| (0..self.width).map(move |column| Point { row, column }))
                .filter(|&point| self[point] == Cell::Land)
                .filter(|point| {
                    let neighbors = point.neighbors(self.width, self.height).collect::<Vec<_>>();
                    neighbors.len() < 8
                        || neighbors
                            .into_iter()
                            .filter(|&neighbor| matches!(self[neighbor], Cell::Water))
                            .count()
                            > 0
                }),
        );
        let mut to_check = HashSet::new();
        while !to_add.is_empty() {
            for point in to_add.drain() {
                self[point] = Cell::Sand;
                to_check.extend(point.neighbors(self.width, self.height));
            }
            for point in to_check.drain().filter(|&point| self[point] == Cell::Land) {
                if point
                    .neighbors(self.width, self.height)
                    .filter(|&neighbor| matches!(self[neighbor], Cell::Sand))
                    .count()
                    > 4
                {
                    to_add.insert(point);
                }
            }
        }
    }

    fn can_be_tree(&self, point: Point) -> bool {
        self[point] == Cell::Land
            && point
                .tree_stamp_neighbors(self.width, self.height)
                .all(|neighbor| matches!(self[neighbor], Cell::Land | Cell::Tree))
    }

    fn mark_trees(&mut self) {
        let mut to_add = HashSet::new();
        to_add.extend(
            (0..self.height)
                .flat_map(|row| (0..self.width).map(move |column| Point { row, column }))
                .filter(|&point| self.can_be_tree(point)),
        );
        let mut to_check = HashSet::new();
        while !to_add.is_empty() {
            for point in to_add.drain() {
                self[point] = Cell::Tree;
                to_check.extend(point.tree_stamp_neighbors(self.width, self.height));
            }
            to_add.extend(to_check.drain().filter(|&point| self.can_be_tree(point)));
        }
        for _ in 0..2 {
            let extra_trees = (0..self.height)
                .flat_map(|row| (0..self.width).map(move |column| Point { row, column }))
                .filter(|&point| self[point] == Cell::Land)
                .filter(|&point| {
                    point
                        .neighbors(self.width, self.height)
                        .try_fold(0, |num_trees, neighbor| {
                            if self[neighbor] == Cell::Sand {
                                None
                            } else {
                                Some(num_trees + ((self[neighbor] == Cell::Tree) as usize))
                            }
                        })
                        .is_some_and(|num_trees| num_trees >= 4)
                })
                .collect::<Vec<_>>();
            for point in extra_trees {
                self[point] = Cell::Tree;
            }
        }
    }
}

impl Index<Point> for Map {
    type Output = Cell;

    fn index(&self, index: Point) -> &Self::Output {
        &self.cells[index.row][index.column]
    }
}

impl IndexMut<Point> for Map {
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        &mut self.cells[index.row][index.column]
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for row in &self.cells {
            for cell in row {
                write!(f, "{cell}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl FromStr for Map {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.lines()
            .map(|line| {
                line.chars()
                    .map(|c| {
                        Cell::try_from(c).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                    })
                    .collect::<io::Result<Vec<_>>>()
            })
            .collect::<io::Result<Vec<_>>>()
            .map(|cells| {
                let height = cells.len();
                let width = cells.first().map(Vec::len).unwrap_or(0);
                Self {
                    cells,
                    width,
                    height,
                }
            })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Point {
    row: usize,
    column: usize,
}

impl Add<Delta> for Point {
    type Output = Option<Self>;

    fn add(mut self, rhs: Delta) -> Self::Output {
        self.row = self.row.checked_add_signed(rhs.delta_row)?;
        self.column = self.column.checked_add_signed(rhs.delta_column)?;
        Some(self)
    }
}

impl Add<Delta> for &Point {
    type Output = Option<Point>;

    fn add(self, rhs: Delta) -> Self::Output {
        *self + rhs
    }
}

impl<'d> Add<&'d Delta> for Point {
    type Output = Option<Self>;

    fn add(self, rhs: &'d Delta) -> Self::Output {
        self + *rhs
    }
}

impl<'d> Add<&'d Delta> for &Point {
    type Output = Option<Point>;

    fn add(self, rhs: &'d Delta) -> Self::Output {
        *self + *rhs
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Delta {
    delta_row: isize,
    delta_column: isize,
}

impl<T> Add<T> for Delta
where
    T: Add<Self>,
{
    type Output = <T as Add<Self>>::Output;

    fn add(self, rhs: T) -> Self::Output {
        rhs + self
    }
}

impl<T> Add<T> for &Delta
where
    T: Add<Self>,
{
    type Output = <T as Add<Self>>::Output;

    fn add(self, rhs: T) -> Self::Output {
        rhs + self
    }
}

impl Point {
    pub fn neighbors(&self, width: usize, height: usize) -> impl Iterator<Item = Self> {
        #[rustfmt::skip]
        const DELTAS: [(isize, isize); 8] = [
            (-1, -1), (0, -1), (1, -1),
            (-1,  0),          (1,  0),
            (-1,  1), (0,  1), (1,  1),
        ];
        self.neighbors_with(&DELTAS, width, height)
    }

    pub fn tree_stamp_neighbors(&self, width: usize, height: usize) -> impl Iterator<Item = Self> {
        #[rustfmt::skip]
        const DELTAS: [(isize, isize); 20] = [
                      (-1, -2), (0, -2), (1, -2),
            (-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1),
            (-2,  0), (-1,  0),          (1,  0), (2,  0),
            (-2,  1), (-1,  1), (0,  1), (1,  1), (2,  1),
                      (-1,  2), (0,  2), (1,  2),
        ];
        self.neighbors_with(&DELTAS, width, height)
    }

    fn neighbors_with(
        &self,
        deltas: &[(isize, isize)],
        width: usize,
        height: usize,
    ) -> impl Iterator<Item = Point> {
        deltas
            .iter()
            .filter_map(move |&(delta_column, delta_row)| {
                self + Delta {
                    delta_row,
                    delta_column,
                }
            })
            .filter(move |neighbor| neighbor.row < height && neighbor.column < width)
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut map = {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        buf.parse::<Map>()?
    };
    map.convert_to_land_and_water();
    #[cfg(debug_assertions)]
    eprintln!("{map}");
    Ok(map
        .cells
        .into_iter()
        .flatten()
        .filter(|cell| matches!(cell, Cell::Land))
        .count())
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut map = {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        buf.parse::<Map>()?
    };
    map.convert_to_land_and_water();
    map.mark_sand();
    #[cfg(debug_assertions)]
    eprintln!("{map}");
    Ok(map
        .cells
        .into_iter()
        .flatten()
        .filter(|cell| matches!(cell, Cell::Sand))
        .count())
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut map = {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        buf.parse::<Map>()?
    };
    map.convert_to_land_and_water();
    map.mark_sand();
    map.mark_trees();
    #[cfg(debug_assertions)]
    eprintln!("{map}");
    Ok(map
        .cells
        .into_iter()
        .flatten()
        .filter(|cell| matches!(cell, Cell::Tree))
        .count())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Lord of the wings Quest 5 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "lord-of-the-wings_05-1.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 5 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "lord-of-the-wings_05-2.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 5 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "lord-of-the-wings_05-3.txt"
            )?))?
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    const TEST_DATA: &str = concat!(
        "010100100100001101000\n",
        "111101001000001110000\n",
        "101111011000101010011\n",
        "001000101010000100000\n",
        "000110100110011001100\n",
        "000001110111111000000\n",
        "101100010101011110000\n",
        "001101100000001010011\n",
        "001001001111010001001\n",
        "100011011110011010101\n",
        "001100101110010010000\n",
    );

    #[test]
    fn test_part1() -> io::Result<()> {
        let expected = 137;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        let expected = 69;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        let expected = 14;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
