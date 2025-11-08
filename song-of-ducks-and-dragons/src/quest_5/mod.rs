use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    iter,
};

fn str_cmp_as_num(left: &str, right: &str) -> std::cmp::Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

type Point = u32;

#[derive(Clone, Copy, Debug, Eq)]
struct Segment {
    center: Point,
    left: Option<Point>,
    right: Option<Point>,
}

impl Segment {
    pub fn value(&self) -> String {
        self.left
            .into_iter()
            .chain(iter::once(self.center))
            .chain(self.right)
            .map(|point| point.to_string())
            .collect()
    }
}

impl From<Point> for Segment {
    fn from(center: Point) -> Self {
        Self {
            center,
            left: None,
            right: None,
        }
    }
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_value = self.value();
        let other_value = other.value();
        self_value
            .len()
            .cmp(&other_value.len())
            .then_with(|| self_value.cmp(&other_value))
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq)]
struct Sword(Vec<Segment>);

impl Sword {
    pub fn quality(&self) -> String {
        self.0
            .iter()
            .map(|point| point.center.to_string())
            .collect()
    }
}

impl FromIterator<Point> for Sword {
    fn from_iter<T: IntoIterator<Item = Point>>(iter: T) -> Self {
        Self(iter.into_iter().fold(vec![], |mut acc, point| {
            let mut placed = false;
            for segment in &mut acc {
                match segment {
                    Segment {
                        center,
                        left: left @ None,
                        ..
                    } if point < *center => {
                        *left = Some(point);
                        placed = true;
                        break;
                    }
                    Segment {
                        center,
                        right: right @ None,
                        ..
                    } if point > *center => {
                        *right = Some(point);
                        placed = true;
                        break;
                    }
                    _ => {}
                }
            }
            if !placed {
                acc.push(Segment::from(point));
            }
            acc
        }))
    }
}

impl Ord for Sword {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_quality = self.quality();
        let other_quality = other.quality();
        str_cmp_as_num(&self_quality, &other_quality).then_with(|| {
            self.0
                .iter()
                .map(Segment::value)
                .map(|value| value.parse::<u64>().unwrap())
                .map(Some)
                .chain(iter::repeat(None))
                .zip(
                    other
                        .0
                        .iter()
                        .map(Segment::value)
                        .map(|value| value.parse::<u64>().unwrap())
                        .map(Some)
                        .chain(iter::repeat(None)),
                )
                .take_while(|(a, b)| !(a.is_none() && b.is_none()))
                .fold(std::cmp::Ordering::Equal, |acc, (left, right)| {
                    acc.then_with(|| left.cmp(&right))
                })
        })
    }
}

impl PartialEq for Sword {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Sword {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<String> {
    let input = input.lines().next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Couldn't read fishbone data")
    })??;
    let Some((_, points)) = input.split_once(':') else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Fishbone data must contain a colon: {input:?}",
        ));
    };
    let points = points
        .split(',')
        .map(|point| {
            point.parse::<Point>().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{e:?}: Cannot parse {point:?} as Point"),
                )
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    Ok(Sword::from_iter(points).quality())
}

fn part2(input: &mut dyn BufRead) -> io::Result<u64> {
    let mut qualities = input
        .lines()
        .map(|line| {
            let line = line?;
            let Some((_, points)) = line.split_once(':') else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Fishbone data must contain a colon: {input:?}",
                ));
            };
            points
                .split(',')
                .map(|point| {
                    point.parse::<Point>().map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("{e:?}: Cannot parse {point:?} as Point"),
                        )
                    })
                })
                .collect::<io::Result<Vec<_>>>()
        })
        .map(|points| {
            points.and_then(|points| {
                Sword::from_iter(points)
                    .quality()
                    .parse::<u64>()
                    .map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("{e:?}: Couldn't parse quality string as u64"),
                        )
                    })
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    qualities.sort();
    Ok(qualities.last().unwrap() - qualities.first().unwrap())
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut swords = input
        .lines()
        .map(|line| {
            let line = line?;
            let Some((id, points)) = line.split_once(':') else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Sword data must contain a colon",
                ));
            };
            let sword = points
                .split(',')
                .map(|point| {
                    point.parse::<Point>().map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("{e:?}: Couldn't parse {point:?} as a Point"),
                        )
                    })
                })
                .collect::<io::Result<Sword>>()?;
            Ok((
                sword,
                id.parse::<usize>().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("{e:?}: Can't parse {id:?} as usize"),
                    )
                })?,
            ))
        })
        .collect::<io::Result<Vec<_>>>()?;
    swords.sort_by(|left, right| right.cmp(left));
    Ok(swords
        .into_iter()
        .map(|(_, id)| id)
        .enumerate()
        .map(|(idx, id)| (idx + 1) * id)
        .sum())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 5 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_05-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 5 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_05-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 5 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_05-3.txt"
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
        const TEST_DATA: &str = "58:5,3,7,8,9,10,4,5,7,8,8\n";
        let expected = "581078";
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "1:2,4,1,1,8,2,7,9,8,6\n",
            "2:7,9,9,3,8,3,8,8,6,8\n",
            "3:4,7,6,9,1,8,3,7,2,2\n",
            "4:6,4,2,1,7,4,5,5,5,8\n",
            "5:2,9,3,8,3,9,5,2,1,4\n",
            "6:2,4,9,6,7,4,1,7,6,8\n",
            "7:2,3,7,6,2,2,4,1,4,2\n",
            "8:5,1,5,6,8,3,1,8,3,9\n",
            "9:5,7,7,3,7,2,3,8,6,7\n",
            "10:4,1,9,3,8,5,4,3,5,5\n",
        );
        let expected = 77053;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "1:7,1,9,1,6,9,8,3,7,2\n",
            "2:6,1,9,2,9,8,8,4,3,1\n",
            "3:7,1,9,1,6,9,8,3,8,3\n",
            "4:6,1,9,2,8,8,8,4,3,1\n",
            "5:7,1,9,1,6,9,8,3,7,3\n",
            "6:6,1,9,2,8,8,8,4,3,5\n",
            "7:3,7,2,2,7,4,4,6,3,1\n",
            "8:3,7,2,2,7,4,4,6,3,7\n",
            "9:3,7,2,2,7,4,1,6,3,7\n",
        );
        let expected = 260;
        let actual = part3(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        const TEST_DATA_2: &str = "1:7,1,9,1,6,9,8,3,7,2\n2:7,1,9,1,6,9,8,3,7,2\n";
        let expected = 4;
        let actual = part3(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
