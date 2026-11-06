use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    iter,
    str::FromStr,
};

#[derive(Clone, Copy, Debug)]
enum GridDirection {
    Southeast,
    Southwest,
    West,
    Northwest,
    Northeast,
    East,
}

impl Display for GridDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Southeast => write!(f, "Southeast"),
            Self::Southwest => write!(f, "Southwest"),
            Self::West => write!(f, "West"),
            Self::Northwest => write!(f, "Northwest"),
            Self::Northeast => write!(f, "Northeast"),
            Self::East => write!(f, "East"),
        }
    }
}

struct Grid {
    cells: Vec<Vec<u8>>,
}

impl Grid {
    pub fn read(input: impl BufRead) -> io::Result<Self> {
        Ok(Self {
            cells: input
                .lines()
                .map(|line| {
                    line.and_then(|line| {
                        line.split('.')
                            .filter(|tile| !tile.is_empty())
                            .map(|tile| {
                                tile.parse::<u8>().map_err(|_| {
                                    io::Error::new(
                                        io::ErrorKind::InvalidData,
                                        format!("Non-numeric tile {tile:?}"),
                                    )
                                })
                            })
                            .collect::<io::Result<Vec<_>>>()
                    })
                })
                .collect::<io::Result<Vec<_>>>()?,
        })
    }

    pub fn required_power(&self, direction: GridDirection) -> u32 {
        fn line_power((mut acc, mut shadow_height): (u32, u8), ent_height: u8) -> (u32, u8) {
            shadow_height = shadow_height.saturating_sub(1);
            // #[cfg(debug_assertions)]
            // eprintln!("Ent of height {ent_height} is in a shadow of height {shadow_height}");
            if ent_height > shadow_height {
                shadow_height = ent_height;
                acc += 1;
            }
            (acc, shadow_height)
        }

        let middle_row_idx = self.cells.len() / 2;
        match direction {
            GridDirection::Southeast => (0..middle_row_idx)
                .map(|num_rows_up| {
                    self.cells[(middle_row_idx - num_rows_up)..]
                        .iter()
                        .zip((0..num_rows_up).chain(iter::repeat(num_rows_up)))
                        .map(|(row, num_skip)| row[num_skip])
                        .fold((0, 0), line_power)
                        .0
                })
                .chain((0..self.cells[0].len()).map(|num_cols_right| {
                    self.cells[..(self.cells.len() - num_cols_right)]
                        .iter()
                        .zip((0..middle_row_idx).chain(iter::repeat(middle_row_idx)))
                        .map(|(row, num_skip)| row[num_skip + num_cols_right])
                        .fold((0, 0), line_power)
                        .0
                }))
                .sum(),
            GridDirection::Southwest => (0..self.cells[0].len())
                .map(|num_cols_right| {
                    self.cells[..=(middle_row_idx + num_cols_right)]
                        .iter()
                        .zip(
                            iter::repeat_n(num_cols_right, middle_row_idx + 1)
                                .chain((0..num_cols_right).rev()),
                        )
                        .map(|(row, col_idx)| row[col_idx])
                        .fold((0, 0), line_power)
                        .0
                })
                .chain((1..=middle_row_idx).map(|num_rows_down| {
                    self.cells[num_rows_down..]
                        .iter()
                        .zip(
                            iter::repeat_n(middle_row_idx, middle_row_idx - num_rows_down)
                                .chain((0..=middle_row_idx).rev()),
                        )
                        .map(|(row, col_idx)| row[col_idx + num_rows_down])
                        .fold((0, 0), line_power)
                        .0
                }))
                .sum(),
            GridDirection::West => self
                .cells
                .iter()
                .map(|row| row.iter().copied().rev().fold((0, 0), line_power).0)
                .sum(),
            GridDirection::Northwest => (0..middle_row_idx)
                .map(|num_rows_up| {
                    self.cells[(middle_row_idx - num_rows_up)..]
                        .iter()
                        .rev()
                        .zip(
                            iter::repeat_n(num_rows_up, middle_row_idx + 1)
                                .chain((0..num_rows_up).rev()),
                        )
                        .map(|(row, num_skip)| row[num_skip])
                        .fold((0, 0), line_power)
                        .0
                })
                .chain((0..self.cells[0].len()).map(|num_cols_right| {
                    self.cells[..(self.cells.len() - num_cols_right)]
                        .iter()
                        .rev()
                        .zip(
                            iter::repeat_n(middle_row_idx, middle_row_idx + 1 - num_cols_right)
                                .chain((0..middle_row_idx).rev()),
                        )
                        .map(|(row, num_skip)| row[num_skip + num_cols_right])
                        .fold((0, 0), line_power)
                        .0
                }))
                .sum(),
            GridDirection::Northeast => (0..self.cells[0].len())
                .map(|num_cols_right| {
                    self.cells[..=(middle_row_idx + num_cols_right)]
                        .iter()
                        .rev()
                        .zip(
                            (0..num_cols_right)
                                .chain(iter::repeat_n(num_cols_right, middle_row_idx + 1)),
                        )
                        .map(|(row, col_idx)| row[col_idx])
                        .fold((0, 0), line_power)
                        .0
                })
                .chain((1..=middle_row_idx).map(|num_rows_down| {
                    self.cells[num_rows_down..]
                        .iter()
                        .rev()
                        .zip((0..=middle_row_idx).chain(iter::repeat_n(
                            middle_row_idx,
                            middle_row_idx - num_rows_down,
                        )))
                        .map(|(row, col_idx)| row[col_idx + num_rows_down])
                        .fold((0, 0), line_power)
                        .0
                }))
                .sum(),
            GridDirection::East => self
                .cells
                .iter()
                .map(|row| row.iter().copied().fold((0, 0), line_power).0)
                .sum(),
        }
    }

    pub fn run_growth_round(&mut self, direction: GridDirection) -> (u32, u64) {
        #[derive(Default)]
        struct PowerLine {
            shadow_height: u8,
            new_warriors: u32,
            power: u64,
        }

        fn grow_line(mut acc: PowerLine, cell: &mut u8) -> PowerLine {
            acc.shadow_height = acc.shadow_height.saturating_sub(1);
            if *cell == 0 || *cell > 9 {
                *cell = cell.wrapping_add(1);
                return acc;
            }
            if *cell > acc.shadow_height {
                acc.shadow_height = *cell;
                *cell += 1;
                acc.power += 1;
            }
            if *cell == 10 {
                acc.new_warriors += 1;
                *cell = 1u8.wrapping_sub(3);
            }
            acc
        }

        let middle_row_idx = self.cells.len() / 2;
        match direction {
            GridDirection::Southeast => {
                let res = (0..middle_row_idx)
                    .map(|num_rows_up| {
                        self.cells[(middle_row_idx - num_rows_up)..]
                            .iter_mut()
                            .zip((0..num_rows_up).chain(iter::repeat(num_rows_up)))
                            .map(|(row, num_skip)| &mut row[num_skip])
                            .fold(PowerLine::default(), grow_line)
                    })
                    .fold((0, 0), |(warriors, power), power_line| {
                        (warriors + power_line.new_warriors, power + power_line.power)
                    });
                (0..self.cells[0].len())
                    .map(|num_cols_right| {
                        let first_unused_row = self.cells.len() - num_cols_right;
                        self.cells[..first_unused_row]
                            .iter_mut()
                            .zip((0..middle_row_idx).chain(iter::repeat(middle_row_idx)))
                            .map(|(row, num_skip)| &mut row[num_skip + num_cols_right])
                            .fold(PowerLine::default(), grow_line)
                    })
                    .fold(res, |(new_warriors, power), power_line| {
                        (
                            new_warriors + power_line.new_warriors,
                            power + power_line.power,
                        )
                    })
            }
            GridDirection::Southwest => {
                let res = (0..self.cells[0].len())
                    .map(|num_cols_right| {
                        self.cells[..=(middle_row_idx + num_cols_right)]
                            .iter_mut()
                            .zip(
                                iter::repeat_n(num_cols_right, middle_row_idx + 1)
                                    .chain((0..num_cols_right).rev()),
                            )
                            .map(|(row, col_idx)| &mut row[col_idx])
                            .fold(PowerLine::default(), grow_line)
                    })
                    .fold((0, 0), |(warriors, power), power_line| {
                        (warriors + power_line.new_warriors, power + power_line.power)
                    });
                (1..=middle_row_idx)
                    .map(|num_rows_down| {
                        self.cells[num_rows_down..]
                            .iter_mut()
                            .zip(
                                iter::repeat_n(middle_row_idx, middle_row_idx - num_rows_down)
                                    .chain((0..=middle_row_idx).rev()),
                            )
                            .map(|(row, col_idx)| &mut row[col_idx + num_rows_down])
                            .fold(PowerLine::default(), grow_line)
                    })
                    .fold(res, |(new_warriors, power), power_line| {
                        (
                            new_warriors + power_line.new_warriors,
                            power + power_line.power,
                        )
                    })
            }
            GridDirection::West => self
                .cells
                .iter_mut()
                .map(|row| row.iter_mut().rev().fold(PowerLine::default(), grow_line))
                .fold((0, 0), |(new_warriors, power), power_line| {
                    (
                        new_warriors + power_line.new_warriors,
                        power + power_line.power,
                    )
                }),
            GridDirection::Northwest => {
                let res = (0..middle_row_idx)
                    .map(|num_rows_up| {
                        self.cells[(middle_row_idx - num_rows_up)..]
                            .iter_mut()
                            .rev()
                            .zip(
                                iter::repeat_n(num_rows_up, middle_row_idx + 1)
                                    .chain((0..num_rows_up).rev()),
                            )
                            .map(|(row, num_skip)| &mut row[num_skip])
                            .fold(PowerLine::default(), grow_line)
                    })
                    .fold((0, 0), |(new_warriors, power), power_line| {
                        (
                            new_warriors + power_line.new_warriors,
                            power + power_line.power,
                        )
                    });
                (0..self.cells[0].len())
                    .map(|num_cols_right| {
                        let first_unused_row = self.cells.len() - num_cols_right;
                        self.cells[..first_unused_row]
                            .iter_mut()
                            .rev()
                            .zip(
                                iter::repeat_n(middle_row_idx, middle_row_idx + 1 - num_cols_right)
                                    .chain((0..middle_row_idx).rev()),
                            )
                            .map(|(row, num_skip)| &mut row[num_skip + num_cols_right])
                            .fold(PowerLine::default(), grow_line)
                    })
                    .fold(res, |(new_warriors, power), power_line| {
                        (
                            new_warriors + power_line.new_warriors,
                            power + power_line.power,
                        )
                    })
            }
            GridDirection::Northeast => {
                let res = (0..self.cells[0].len())
                    .map(|num_cols_right| {
                        self.cells[..=(middle_row_idx + num_cols_right)]
                            .iter_mut()
                            .rev()
                            .zip(
                                (0..num_cols_right)
                                    .chain(iter::repeat_n(num_cols_right, middle_row_idx + 1)),
                            )
                            .map(|(row, col_idx)| &mut row[col_idx])
                            .fold(PowerLine::default(), grow_line)
                    })
                    .fold((0, 0), |(new_warriors, power), power_line| {
                        (
                            new_warriors + power_line.new_warriors,
                            power + power_line.power,
                        )
                    });
                (1..=middle_row_idx)
                    .map(|num_rows_down| {
                        self.cells[num_rows_down..]
                            .iter_mut()
                            .rev()
                            .zip((0..=middle_row_idx).chain(iter::repeat_n(
                                middle_row_idx,
                                middle_row_idx - num_rows_down,
                            )))
                            .map(|(row, col_idx)| &mut row[col_idx + num_rows_down])
                            .fold(PowerLine::default(), grow_line)
                    })
                    .fold(res, |(new_warriors, power), power_line| {
                        (
                            new_warriors + power_line.new_warriors,
                            power + power_line.power,
                        )
                    })
            }
            GridDirection::East => self
                .cells
                .iter_mut()
                .map(|row| row.iter_mut().fold(PowerLine::default(), grow_line))
                .fold((0, 0), |(new_warriors, power), power_line| {
                    (
                        new_warriors + power_line.new_warriors,
                        power + power_line.power,
                    )
                }),
        }
    }

    #[cfg(debug_assertions)]
    pub fn write(&self, mut out: impl io::Write) -> io::Result<()> {
        let middle_row_idx = self.cells.len() / 2;
        for (row_idx, row) in self.cells[..middle_row_idx].iter().enumerate() {
            let num_zeroes = middle_row_idx - row_idx;
            (0..num_zeroes).try_for_each(|_| write!(out, "."))?;
            row.iter().try_for_each(|tile| write!(out, "{tile}."))?;
            (0..(num_zeroes - 1)).try_for_each(|_| write!(out, "."))?;
            writeln!(out)?;
        }
        write!(out, "{}", self.cells[middle_row_idx][0])?;
        for cell in &self.cells[middle_row_idx][1..] {
            write!(out, ".{cell}")?;
        }
        writeln!(out)?;
        for (row_idx, row) in self.cells[(middle_row_idx + 1)..].iter().enumerate() {
            let num_zeroes = row_idx + 1;
            (0..num_zeroes).try_for_each(|_| write!(out, "."))?;
            row.iter().try_for_each(|tile| write!(out, "{tile}."))?;
            (0..(num_zeroes - 1)).try_for_each(|_| write!(out, "."))?;
            writeln!(out)?;
        }
        Ok(())
    }
}

impl FromStr for Grid {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::read(std::io::Cursor::new(s))
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<u32> {
    let grid = Grid::read(input)?;
    #[cfg(debug_assertions)]
    grid.write(io::stderr().lock())?;
    Ok(grid.required_power(GridDirection::Southeast))
}

fn part2(input: &mut dyn BufRead) -> io::Result<u32> {
    let grid = Grid::read(input)?;
    #[cfg(debug_assertions)]
    grid.write(io::stderr().lock())?;
    Ok([
        GridDirection::Southeast,
        GridDirection::Southwest,
        GridDirection::West,
        GridDirection::Northwest,
        GridDirection::Northeast,
        GridDirection::East,
    ]
    .into_iter()
    .map(|direction| {
        let ret = grid.required_power(direction);
        #[cfg(debug_assertions)]
        eprintln!("Required power to the {direction} is {ret}");
        ret
    })
    .sum())
}

fn part3(input: &mut dyn BufRead) -> io::Result<u64> {
    let mut grid = Grid::read(input)?;
    #[cfg(debug_assertions)]
    grid.write(io::stderr().lock())?;
    [
        GridDirection::Southeast,
        GridDirection::Southwest,
        GridDirection::West,
        GridDirection::Northwest,
        GridDirection::Northeast,
        GridDirection::East,
    ]
    .into_iter()
    .cycle()
    .map(|direction| grid.run_growth_round(direction))
    .enumerate()
    .scan(
        (0, 0),
        |(warriors, power), (_round_num, (new_warriors, new_power))| {
            if *warriors >= 100_000 {
                return None;
            }
            *warriors += new_warriors;
            *power += new_power;
            #[cfg(debug_assertions)]
            {
                let round_num = _round_num + 1;
                match round_num {
                    1..101 | 1000 | 10_000 | 20_000 | 30_000 | 32_000.. => {
                        eprintln!(
                            "{round_num}: ({new_power}, {new_warriors}) ({warriors}, {power})"
                        )
                    }
                    _ => {}
                }
            }
            Some(*power)
        },
    )
    .enumerate()
    .last()
    .map(|(_round_num, res)| {
        #[cfg(debug_assertions)]
        {
            let round_num = _round_num + 1;
            eprintln!("{round_num} rounds");
        }
        res
    })
    .ok_or_else(|| unreachable!("1000000 warriors will eventually be generated"))
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Lord of the wings Quest 8 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "lord-of-the-wings_08-1.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 8 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "lord-of-the-wings_08-2.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 8 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "lord-of-the-wings_08-3.txt"
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
    fn test_grid_write() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "....1.9.6.1.7....\n",
            "...9.2.3.3.8.1...\n",
            "..7.8.3.5.1.4.3..\n",
            ".4.6.7.4.1.1.8.9.\n",
            "4.5.3.6.5.8.9.4.9\n",
            ".9.5.1.5.6.2.8.9.\n",
            "..7.7.6.4.6.2.4..\n",
            "...9.3.8.3.8.4...\n",
            "....1.6.6.2.8....\n",
        );
        let mut buf = vec![];
        TEST_DATA.parse::<Grid>()?.write(&mut buf)?;
        buf.split(|&b| b == b'\n')
            .zip(TEST_DATA.as_bytes().split(|&b| b == b'\n'))
            .for_each(|(actual, expected)| {
                assert_eq!(expected, actual);
            });
        Ok(())
    }

    #[test]
    fn test_part1() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "....1.9.6.1.7....\n",
            "...9.2.3.3.8.1...\n",
            "..7.8.3.5.1.4.3..\n",
            ".4.6.7.4.1.1.8.9.\n",
            "4.5.3.6.5.8.9.4.9\n",
            ".9.5.1.5.6.2.8.9.\n",
            "..7.7.6.4.6.2.4..\n",
            "...9.3.8.3.8.4...\n",
            "....1.6.6.2.8....\n",
        );
        let expected = 32;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "....1.9.6.1.7....\n",
            "...9.2.3.3.8.1...\n",
            "..7.8.3.5.1.4.3..\n",
            ".4.6.7.4.1.1.8.9.\n",
            "4.5.3.6.5.8.9.4.9\n",
            ".9.5.1.5.6.2.8.9.\n",
            "..7.7.6.4.6.2.4..\n",
            "...9.3.8.3.8.4...\n",
            "....1.6.6.2.8....\n",
        );
        let expected = 187;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "....1.9.6.1.7....\n",
            "...9.2.3.3.8.1...\n",
            "..7.8.3.5.1.4.3..\n",
            ".4.6.7.4.1.1.8.9.\n",
            "4.5.3.6.5.8.9.4.9\n",
            ".9.5.1.5.6.2.8.9.\n",
            "..7.7.6.4.6.2.4..\n",
            "...9.3.8.3.8.4...\n",
            "....1.6.6.2.8....\n",
        );
        let expected = 899_889;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
