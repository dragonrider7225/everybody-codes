use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    iter,
    str::FromStr,
    time::Instant,
};

#[derive(Clone, Copy, Eq, PartialEq)]
struct Wall {
    x: usize,
    y: usize,
    height: usize,
}

impl Debug for Wall {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { x, y, height } = self;
        write!(f, "Wall {{ x: {x}, y: {y}, height: {height} }}")
    }
}

impl FromStr for Wall {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(',');
        let x = parts.next().ok_or_else(|| {
            format!("{s:?} is not a valid wall: a wall must contain exactly two ','")
        })?;
        let y = parts.next().ok_or_else(|| {
            format!("{s:?} is not a valid wall: a wall must contain exactly two ','")
        })?;
        let height = parts.next().ok_or_else(|| {
            format!("{s:?} is not a valid wall: a wall must contain exactly two ','")
        })?;
        if parts.next().is_some() {
            return Err(format!(
                "{s:?} is not a valid wall: a wall must contain exactly two ','"
            ));
        }
        let x = x
            .parse::<usize>()
            .map_err(|e| format!("{e:?}: {x:?} must be a number"))?;
        let y = y
            .parse::<usize>()
            .map_err(|e| format!("{e:?}: {y:?} must be a number"))?;
        let height = height
            .parse::<usize>()
            .map_err(|e| format!("{e:?}: {height:?} must be a number"))?;
        Ok(Self { x, y, height })
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let gaps = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Wall>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    (1..=gaps.last().expect("Missing notes").x)
        .fold(HashMap::from([(0usize, 0usize)]), |frontier, x| {
            frontier
                .into_iter()
                .flat_map(|(y, num_flaps)| {
                    iter::once((y + 1, num_flaps + 1))
                        .chain(y.checked_sub(1).map(|y| (y, num_flaps)))
                        .filter(|&(y, _)| {
                            gaps.iter()
                                .find(|wall| wall.x == x)
                                .is_none_or(|wall| y >= wall.y && y < wall.y + wall.height)
                        })
                })
                .fold(HashMap::new(), |mut acc, (y, num_flaps)| {
                    acc.entry(y)
                        .and_modify(|last_flaps| *last_flaps = num_flaps.min(*last_flaps))
                        .or_insert(num_flaps);
                    acc
                })
        })
        .values()
        .copied()
        .min()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Cannot reach last gap"))
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut gaps = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Wall>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    gaps.sort_by(|left, right| left.x.cmp(&right.x).then(left.y.cmp(&right.y)));
    let mut next_gap = 0;
    let start = Instant::now();
    let mut x = 0usize;
    let mut frontier = HashMap::from([(0usize, 0usize)]);
    loop {
        let gap_range = next_gap
            ..=(gaps[next_gap..]
                .iter()
                .rposition(|wall| wall.x == gaps[next_gap].x)
                .unwrap()
                + next_gap);
        let distance = (gaps[next_gap].x - x) as isize;
        next_gap = *gap_range.end() + 1;
        frontier = frontier
            .into_iter()
            .flat_map(|(y, num_flaps)| {
                gaps[gap_range.clone()].iter().flat_map(move |gap| {
                    let min_flaps = match (gap.y as isize) - (y as isize) {
                        min_flaps if min_flaps.rem_euclid(2) == distance % 2 => min_flaps,
                        min_flaps => min_flaps + 1,
                    }
                    .max(-distance);
                    let max_flaps = match ((gap.y + gap.height - 1) as isize) - (y as isize) {
                        max_flaps if max_flaps.rem_euclid(2) == distance % 2 => max_flaps,
                        max_flaps => max_flaps - 1,
                    }
                    .min(distance);
                    let flaps = [min_flaps, max_flaps].into_iter();
                    #[expect(clippy::iter_skip_zero, reason = "Type checking")]
                    let flaps = match min_flaps.cmp(&max_flaps) {
                        Ordering::Less => flaps.skip(0),
                        Ordering::Equal => flaps.skip(1),
                        Ordering::Greater => flaps.skip(2),
                    };
                    flaps.map(move |bonus_flaps| {
                        assert_eq!(
                            bonus_flaps.rem_euclid(2),
                            distance % 2,
                            "{bonus_flaps} is incorrect"
                        );
                        let new_flaps = bonus_flaps.max(0) + (distance - bonus_flaps.abs()) / 2;
                        (
                            y.strict_add_signed(bonus_flaps),
                            num_flaps.strict_add_signed(new_flaps),
                        )
                    })
                })
            })
            .fold(HashMap::new(), |mut acc, (y, num_flaps)| {
                acc.entry(y)
                    .and_modify(|last_flaps| *last_flaps = num_flaps.min(*last_flaps))
                    .or_insert(num_flaps);
                acc
            });
        if frontier.is_empty() {
            break;
        }
        if next_gap < gaps.len() {
            x = gaps[next_gap - 1].x;
        } else {
            break;
        }
    }
    let ret = frontier
        .values()
        .copied()
        .min()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Cannot reach last gap"));
    eprintln!("Calculation took {:?}", Instant::now() - start);
    ret
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    part2(input)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 19 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_19-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 19 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_19-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 19 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_19-3.txt"
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
        const TEST_DATA: &str = "7,7,2\n12,0,4\n15,5,3\n24,1,6\n28,5,5\n40,8,2\n";
        let expected = 24;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        #[rustfmt::skip]
        const TEST_DATA: &str = concat!(
            "7,7,2\n",
            "7,1,3\n",
            "12,0,4\n",
            "15,5,3\n",
            "24,1,6\n",
            "28,5,5\n",
            "40,3,3\n",
            "40,8,2\n",
        );
        let expected = 22;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
