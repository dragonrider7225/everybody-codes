use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead, BufReader},
    mem,
    ops::{Add, Index, IndexMut},
    time::Instant,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Entity {
    Dragon,
    Sheep,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Tile {
    Empty,
    Entity(Entity),
    Hideout {
        sheep: Option<Entity>,
        dragon: Option<Entity>,
    },
}

impl TryFrom<char> for Tile {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Empty),
            'D' => Ok(Self::Entity(Entity::Dragon)),
            'S' => Ok(Self::Entity(Entity::Sheep)),
            '#' => Ok(Self::Hideout {
                sheep: None,
                dragon: None,
            }),
            _ => Err(format!("Invalid tile {value:?}")),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Board(Vec<Vec<Tile>>);

impl Board {
    fn read(input: &mut dyn BufRead) -> io::Result<Board> {
        input
            .lines()
            .map(|line| {
                line.and_then(|line| {
                    line.chars()
                        .map(|c| {
                            Tile::try_from(c)
                                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                        })
                        .collect::<io::Result<Vec<_>>>()
                })
            })
            .collect::<io::Result<Vec<_>>>()
            .map(Board)
    }

    fn dragon_position(&self) -> Option<Position> {
        self.0
            .iter()
            .enumerate()
            .flat_map(|(row_idx, row)| {
                row.iter().enumerate().filter_map(move |(col_idx, tile)| {
                    Some(Position {
                        row: row_idx,
                        col: col_idx,
                    })
                    .filter(|_| {
                        matches!(
                            tile,
                            Tile::Entity(Entity::Dragon)
                                | Tile::Hideout {
                                    dragon: Some(_),
                                    ..
                                }
                        )
                    })
                })
            })
            .next()
    }

    /// Returns the position of the tile in the hypothetical row and column after the last of each.
    fn size(&self) -> Position {
        Position {
            row: self.height(),
            col: self.width(),
        }
    }

    fn width(&self) -> usize {
        self.0[0].len()
    }

    fn height(&self) -> usize {
        self.0.len()
    }
}

impl Index<Position> for Board {
    type Output = Tile;

    fn index(&self, index: Position) -> &Self::Output {
        self.index(&index)
    }
}

impl Index<&'_ Position> for Board {
    type Output = Tile;

    fn index(&self, index: &'_ Position) -> &Self::Output {
        &self.0[index.row][index.col]
    }
}

impl IndexMut<Position> for Board {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        self.index_mut(&index)
    }
}

impl IndexMut<&'_ Position> for Board {
    fn index_mut(&mut self, index: &'_ Position) -> &mut Self::Output {
        &mut self.0[index.row][index.col]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    fn checked_add(self, rhs: DeltaPosition) -> Option<Self> {
        let row = self.row.checked_add_signed(rhs.row)?;
        let col = self.col.checked_add_signed(rhs.col)?;
        Some(Self { row, col })
    }
}

impl Add<DeltaPosition> for Position {
    type Output = Self;

    fn add(self, rhs: DeltaPosition) -> Self::Output {
        self.checked_add(rhs).unwrap()
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.row.cmp(&other.row), self.col.cmp(&other.col)) {
            (Ordering::Less, Ordering::Less) => Some(Ordering::Less),
            (Ordering::Equal, Ordering::Equal) => Some(Ordering::Equal),
            (Ordering::Greater, Ordering::Greater) => Some(Ordering::Greater),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DeltaPosition {
    pub row: isize,
    pub col: isize,
}

fn dragon_neighbors(pos: Position, board_size: Position) -> impl IntoIterator<Item = Position> {
    [
        DeltaPosition { row: -2, col: -1 },
        DeltaPosition { row: -2, col: 1 },
        DeltaPosition { row: -1, col: -2 },
        DeltaPosition { row: -1, col: 2 },
        DeltaPosition { row: 1, col: -2 },
        DeltaPosition { row: 1, col: 2 },
        DeltaPosition { row: 2, col: -1 },
        DeltaPosition { row: 2, col: 1 },
    ]
    .into_iter()
    .filter_map(move |delta| pos.checked_add(delta))
    .filter(move |neighbor| neighbor < &board_size)
}

fn part1(input: &mut dyn BufRead, num_steps: usize) -> io::Result<usize> {
    let board = Board::read(input)?;
    let board_size = board.size();
    let mut frontier = board
        .dragon_position()
        .into_iter()
        .map(|pos| (pos, 0))
        .collect::<Vec<_>>();
    let mut visited = frontier.iter().copied().collect::<HashMap<_, _>>();
    while let Some((next, depth)) = frontier.pop() {
        let frontier_len = frontier.len();
        if depth < num_steps {
            frontier.extend(
                dragon_neighbors(next, board_size)
                    .into_iter()
                    .map(|neighbor| (neighbor, depth + 1))
                    .filter(|(neighbor, depth)| {
                        visited
                            .get(neighbor)
                            .is_none_or(|prev_depth| prev_depth > depth)
                    }),
            );
            visited.extend(frontier[frontier_len..].iter().copied());
        }
    }
    Ok(visited
        .keys()
        .filter(|&pos| matches!(board[pos], Tile::Entity(Entity::Sheep)))
        .count())
}

fn part2(input: &mut dyn BufRead, num_rounds: usize) -> io::Result<usize> {
    let num_rounds = num_rounds as isize;
    let board = Board::read(input)?;
    let board_size = board.size();
    let (dragon_positions, sheep_positions, hideout_positions) = board
        .0
        .iter()
        .enumerate()
        .flat_map(|(row_idx, row)| {
            row.iter().copied().enumerate().map(move |(col_idx, tile)| {
                (
                    Position {
                        row: row_idx,
                        col: col_idx,
                    },
                    tile,
                )
            })
        })
        .filter(|(_, tile)| !matches!(tile, Tile::Empty))
        .fold(
            (HashSet::new(), HashSet::new(), HashSet::new()),
            |(mut dragons, mut sheep, mut hideouts), (position, tile)| {
                match tile {
                    Tile::Empty => false,
                    Tile::Entity(Entity::Dragon) => dragons.insert(position),
                    Tile::Entity(Entity::Sheep) => sheep.insert(position),
                    Tile::Hideout {
                        sheep: s,
                        dragon: d,
                    } => {
                        s.map(|_| sheep.insert(position));
                        d.map(|_| dragons.insert(position));
                        hideouts.insert(position)
                    }
                };
                (dragons, sheep, hideouts)
            },
        );
    let num_eaten = (0..num_rounds)
        .fold((0, dragon_positions, sheep_positions), |acc, _| {
            let mut sheep_positions = acc.2;
            let dragon_positions = acc
                .1
                .into_iter()
                .flat_map(|pos| dragon_neighbors(pos, board_size))
                .collect::<HashSet<_>>();
            let eaten_sheep = dragon_positions
                .iter()
                .copied()
                .filter(|pos| sheep_positions.contains(pos) && !hideout_positions.contains(pos))
                .collect::<Vec<_>>();
            let mut num_sheep_eaten = eaten_sheep.len();
            for pos in eaten_sheep {
                sheep_positions.remove(&pos);
            }
            let mut sheep_positions = sheep_positions
                .into_iter()
                .filter_map(|pos| pos.checked_add(DeltaPosition { row: 1, col: 0 }))
                .filter(|pos| pos < &board_size)
                .collect::<HashSet<_>>();
            let eaten_sheep = sheep_positions
                .iter()
                .copied()
                .filter(|pos| dragon_positions.contains(pos) && !hideout_positions.contains(pos))
                .collect::<Vec<_>>();
            num_sheep_eaten += eaten_sheep.len();
            for pos in eaten_sheep {
                sheep_positions.remove(&pos);
            }
            (acc.0 + num_sheep_eaten, dragon_positions, sheep_positions)
        })
        .0;
    Ok(num_eaten)
}

type SeqCache<'h> = HashMap<SparseBoard<'h>, usize>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct SparseBoard<'h> {
    board_size: Position,
    dragon_position: Position,
    /// The position of each column's sheep. `sheep_positions.len() == board_size.col`.
    sheep_positions: Vec<usize>,
    /// The positions of the hideouts in each column.
    hideout_positions: &'h [Vec<usize>],
}

impl<'h> SparseBoard<'h> {
    fn has_hideout(&self, position: Position) -> bool {
        self.hideout_positions[position.col].contains(&position.row)
    }

    fn num_sheep(&self) -> usize {
        self.sheep_positions
            .iter()
            .filter(|&&row| row < self.board_size.row)
            .count()
    }

    fn dragon_move(
        &mut self,
        depth: usize,
        dragon_cache: &mut SeqCache<'h>,
        sheep_cache: &mut SeqCache<'h>,
    ) -> usize {
        if let Some(&ret) = dragon_cache.get(self) {
            return ret;
        }
        let ret = dragon_neighbors(self.dragon_position, self.board_size)
            .into_iter()
            .map(|new_pos| {
                if self.sheep_positions[new_pos.col] == new_pos.row
                    && self.num_sheep() == 1
                    && !self.has_hideout(new_pos)
                {
                    return 1;
                }
                let old_dragon = mem::replace(&mut self.dragon_position, new_pos);
                let old_sheep = self.sheep_positions[new_pos.col];
                if !self.has_hideout(new_pos) && self.sheep_positions[new_pos.col] == new_pos.row {
                    self.sheep_positions[new_pos.col] = self.board_size.row;
                }
                let ret = self.sheep_move(depth + 1, sheep_cache, dragon_cache);
                self.dragon_position = old_dragon;
                self.sheep_positions[new_pos.col] = old_sheep;
                ret
            })
            .sum();
        dragon_cache.insert(self.clone(), ret);
        ret
    }

    fn sheep_move(
        &mut self,
        depth: usize,
        sheep_cache: &mut SeqCache<'h>,
        dragon_cache: &mut SeqCache<'h>,
    ) -> usize {
        if let Some(&ret) = sheep_cache.get(self) {
            return ret;
        }
        let new_sheep = self
            .sheep_positions
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, row)| row < self.board_size.row)
            .map(|(col, row)| Position { row: row + 1, col })
            .filter(|&new_pos| new_pos != self.dragon_position || self.has_hideout(new_pos))
            .collect::<Vec<_>>();
        let ret = if new_sheep.is_empty() {
            self.dragon_move(depth + 1, dragon_cache, sheep_cache)
        } else {
            new_sheep
                .into_iter()
                .filter_map(|new_pos| {
                    if new_pos.partial_cmp(&self.board_size) != Some(Ordering::Less) {
                        return None;
                    }
                    let old_sheep =
                        mem::replace(&mut self.sheep_positions[new_pos.col], new_pos.row);
                    let ret = self.dragon_move(depth + 1, dragon_cache, sheep_cache);
                    self.sheep_positions[new_pos.col] = old_sheep;
                    Some(ret)
                })
                .sum()
        };
        sheep_cache.insert(self.clone(), ret);
        ret
    }
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let board = Board::read(input)?;
    let board_size = board.size();
    let (dragon_position, sheep_positions, hideout_positions) = board
        .0
        .iter()
        .enumerate()
        .flat_map(|(row_idx, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, tile)| !matches!(tile, Tile::Empty))
                .map(move |(col_idx, tile)| {
                    (
                        Position {
                            row: row_idx,
                            col: col_idx,
                        },
                        tile,
                    )
                })
        })
        .fold(
            (
                None,
                vec![board_size.row; board_size.col],
                vec![vec![]; board_size.col],
            ),
            |(mut dragon_position, mut sheep_positions, mut hideout_positions), (pos, tile)| {
                match tile {
                    Tile::Empty => unreachable!(),
                    Tile::Entity(Entity::Sheep) => {
                        sheep_positions[pos.col] = pos.row;
                    }
                    Tile::Entity(Entity::Dragon) => {
                        if dragon_position.is_some() {
                            panic!("Board must contain exactly one dragon");
                        }
                        dragon_position = Some(pos);
                    }
                    Tile::Hideout { sheep, dragon } => {
                        sheep.inspect(|_| sheep_positions[pos.col] = pos.row);
                        dragon.inspect(|_| {
                            if dragon.is_some() {
                                panic!("Board must contain exactly one dragon");
                            } else {
                                dragon_position = Some(pos)
                            }
                        });
                        hideout_positions[pos.col].push(pos.row);
                    }
                }
                (dragon_position, sheep_positions, hideout_positions)
            },
        );
    let dragon_position = dragon_position
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Board must contain dragon"))?;
    let mut board = SparseBoard {
        board_size,
        dragon_position,
        sheep_positions,
        hideout_positions: &hideout_positions,
    };
    let start = Instant::now();
    let ret = board.sheep_move(0, &mut SeqCache::new(), &mut SeqCache::new());
    eprintln!("Computation took {:?}", Instant::now() - start);
    Ok(ret)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 10 Part 1");
        println!(
            "{}",
            part1(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_10-1.txt")?),
                4
            )?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 10 Part 2");
        println!(
            "{}",
            part2(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_10-2.txt")?),
                20
            )?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 10 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_10-3.txt"
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
            "...SSS.......\n",
            ".S......S.SS.\n",
            "..S....S...S.\n",
            "..........SS.\n",
            "..SSSS...S...\n",
            ".....SS..S..S\n",
            "SS....D.S....\n",
            "S.S..S..S....\n",
            "....S.......S\n",
            ".SSS..SS.....\n",
            ".........S...\n",
            ".......S....S\n",
            "SS.....S..S..\n",
        );
        let expected = 27;
        let actual = part1(&mut Cursor::new(TEST_DATA), 3)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "...SSS##.....\n",
            ".S#.##..S#SS.\n",
            "..S.##.S#..S.\n",
            ".#..#S##..SS.\n",
            "..SSSS.#.S.#.\n",
            ".##..SS.#S.#S\n",
            "SS##.#D.S.#..\n",
            "S.S..S..S###.\n",
            ".##.S#.#....S\n",
            ".SSS.#SS..##.\n",
            "..#.##...S##.\n",
            ".#...#.S#...S\n",
            "SS...#.S.#S..\n",
        );
        let expected = 27;
        let actual = part2(&mut Cursor::new(TEST_DATA), 3)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_1() -> io::Result<()> {
        const TEST_DATA_1: &str = "SSS\n..#\n#.#\n#D.\n";
        let expected = 15;
        let actual = part3(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_2() -> io::Result<()> {
        const TEST_DATA_2: &str = "SSS\n..#\n..#\n.##\n.D#\n";
        let expected = 8;
        let actual = part3(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_3() -> io::Result<()> {
        const TEST_DATA_3: &str = "..S..\n.....\n..#..\n.....\n..D..\n";
        let expected = 44;
        let actual = part3(&mut Cursor::new(TEST_DATA_3))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_4() -> io::Result<()> {
        const TEST_DATA_4: &str = ".SS.S\n#...#\n...#.\n##..#\n.####\n##D.#\n";
        let expected = 4406;
        let actual = part3(&mut Cursor::new(TEST_DATA_4))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_5() -> io::Result<()> {
        const TEST_DATA_5: &str = "SSS.S\n.....\n#.#.#\n.#.#.\n#.D.#\n";
        let expected = 13_033_988_838;
        let actual = part3(&mut Cursor::new(TEST_DATA_5))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
