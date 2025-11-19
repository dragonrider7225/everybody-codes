use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    time::Instant,
};

fn neighbors(
    start: (usize, usize),
    size: (usize, usize),
) -> impl IntoIterator<Item = (usize, usize)> {
    [(0, -1), (0, 1), (-1, 0), (1, 0)]
        .into_iter()
        .filter_map(move |delta| {
            start
                .0
                .checked_add_signed(delta.0)
                .and_then(|neighbor_col| Some((neighbor_col, start.1.checked_add_signed(delta.1)?)))
        })
        .filter(move |&neighbor| neighbor.0 < size.0 && neighbor.1 < size.1)
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let grid = input
        .lines()
        .map(|line| line.map(|line| line.bytes().map(|b| b - b'0').collect::<Vec<_>>()))
        .collect::<io::Result<Vec<_>>>()?;
    let height = grid.len();
    let width = grid[0].len();
    let mut visited = HashSet::new();
    let mut frontier = HashSet::<(usize, usize)>::from_iter([(0, 0)]);
    while let Some(next) = frontier.extract_if(|_| true).next() {
        visited.insert(next);
        frontier.extend(
            neighbors(next, (width, height))
                .into_iter()
                .filter(|neighbor| !visited.contains(neighbor))
                .filter(|&neighbor| grid[neighbor.1][neighbor.0] <= grid[next.1][next.0]),
        );
    }
    Ok(visited.len())
}

fn print_colored_grid(
    mut out: impl Write,
    grid: &[Vec<u8>],
    colored: impl Fn((usize, usize)) -> bool,
) {
    const RED: &str = "\x1B[0;31m";
    const NO_COLOR: &str = "\x1B[0m";
    grid.iter()
        .enumerate()
        .map(|(row_idx, row)| {
            row.iter().map(|&b| (b + b'0') as char).enumerate().fold(
                (false, String::new()),
                |(mut color_active, mut acc), (col_idx, c)| {
                    #[expect(clippy::collapsible_else_if, reason = "Parallel structure")]
                    if colored((col_idx, row_idx)) {
                        if !color_active {
                            color_active = true;
                            acc.push_str(RED);
                        }
                    } else {
                        if color_active {
                            color_active = false;
                            acc.push_str(NO_COLOR);
                        }
                    }
                    acc.push(c);
                    (color_active, acc)
                },
            )
        })
        .for_each(|(color_active, row)| {
            let _ = writeln!(out, "{row}{}", if color_active { NO_COLOR } else { "" });
        });
}

fn part2(input: &mut dyn BufRead, log: impl Write) -> io::Result<usize> {
    let grid = input
        .lines()
        .map(|line| line.map(|line| line.bytes().map(|b| b - b'0').collect::<Vec<_>>()))
        .collect::<io::Result<Vec<_>>>()?;
    let height = grid.len();
    let width = grid[0].len();
    let mut visited = HashSet::new();
    let mut frontier = HashSet::<(usize, usize)>::from_iter([(0, 0), (width - 1, height - 1)]);
    while let Some(next) = frontier.extract_if(|_| true).next() {
        visited.insert(next);
        frontier.extend(
            neighbors(next, (width, height))
                .into_iter()
                .filter(|neighbor| !visited.contains(neighbor))
                .filter(|&neighbor| grid[neighbor.1][neighbor.0] <= grid[next.1][next.0]),
        );
    }
    print_colored_grid(log, &grid, |coords| visited.contains(&coords));
    Ok(visited.len())
}

fn part3(input: &mut dyn BufRead, mut log: impl Write) -> io::Result<usize> {
    let grid = input
        .lines()
        .map(|line| line.map(|line| line.bytes().map(|b| b - b'0').collect::<Vec<_>>()))
        .collect::<io::Result<Vec<_>>>()?;
    let height = grid.len();
    let width = grid[0].len();
    let mut total_visited = HashSet::new();
    let start = Instant::now();
    for i in 1..=3 {
        let start = Instant::now();
        let mut best_visited = HashSet::new();
        for row_idx in 0..height {
            for col_idx in 0..width {
                let mut last_visited = HashSet::new();
                let mut frontier = HashSet::new();
                if total_visited.contains(&(col_idx, row_idx)) {
                    continue;
                }
                if neighbors((col_idx, row_idx), (width, height))
                    .into_iter()
                    .any(|neighbor| grid[neighbor.1][neighbor.0] > grid[row_idx][col_idx])
                {
                    continue;
                }
                frontier.insert((col_idx, row_idx));
                while let Some(next) = frontier.extract_if(|_| true).next() {
                    last_visited.insert(next);
                    frontier.extend(
                        neighbors(next, (width, height))
                            .into_iter()
                            .filter(|neighbor| !total_visited.contains(neighbor))
                            .filter(|neighbor| !last_visited.contains(neighbor))
                            .filter(|&neighbor| {
                                grid[neighbor.1][neighbor.0] <= grid[next.1][next.0]
                            }),
                    );
                }
                if last_visited.len() > best_visited.len() {
                    best_visited = last_visited;
                }
            }
        }
        eprintln!("Exploding {} barrels", best_visited.len());
        total_visited.extend(best_visited);
        eprintln!("Calculating barrel {i} took {:?}", Instant::now() - start);
        print_colored_grid(&mut log, &grid, |position| {
            total_visited.contains(&position)
        });
        let _ = writeln!(&mut log);
    }
    eprintln!("Total time to choose barrels: {:?}", Instant::now() - start);
    print_colored_grid(log, &grid, |coords| total_visited.contains(&coords));
    Ok(total_visited.len())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 12 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_12-1.txt"
            )?))?
        );
    }
    {
        const COLORED_GRID: &str = "SoDaD_12_2.grid";
        println!("Song of Ducks and Dragons Quest 12 Part 2");
        println!(
            "{}",
            part2(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_12-2.txt")?),
                File::options()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(COLORED_GRID)?,
            )?
        );
        eprintln!("Colorized grid written to {COLORED_GRID}");
    }
    {
        const COLORED_GRID: &str = "SoDaD_12_3.grid";
        println!("Song of Ducks and Dragons Quest 12 Part 3");
        println!(
            "{}",
            part3(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_12-3.txt")?),
                File::options()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(COLORED_GRID)?,
            )?
        );
        eprintln!("Colorized grid written to {COLORED_GRID}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_part1() -> io::Result<()> {
        const TEST_DATA: &str = "989611\n857782\n746543\n766789\n";
        let expected = 16;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "9589233445\n",
            "9679121695\n",
            "8469121876\n",
            "8352919876\n",
            "7342914327\n",
            "7234193437\n",
            "6789193538\n",
            "6781219648\n",
            "5691219769\n",
            "5443329859\n",
        );
        let expected = 58;
        let actual = part2(&mut Cursor::new(TEST_DATA), io::sink())?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA_1: &str = "5411\n3362\n5235\n3112\n";
        let expected = 14;
        let actual = part3(&mut Cursor::new(TEST_DATA_1), io::sink())?;
        assert_eq!(expected, actual);
        const TEST_DATA_2: &str = concat!(
            "41951111131882511179\n",
            "32112222211518122215\n",
            "31223333322115122219\n",
            "31234444432147511128\n",
            "91223333322176121892\n",
            "61112222211166431583\n",
            "14661111166111111746\n",
            "11111119142122222177\n",
            "41222118881233333219\n",
            "71222127839122222196\n",
            "56111126279711111517\n",
        );
        let expected = 136;
        let actual = part3(&mut Cursor::new(TEST_DATA_2), io::sink())?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
