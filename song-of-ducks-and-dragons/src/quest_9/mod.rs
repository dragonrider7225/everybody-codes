use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    ops::Deref,
    str::FromStr,
    time::Instant,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Base {
    Guanine,
    Cytosine,
    Adenine,
    Thymine,
}

impl Display for Base {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Guanine => write!(f, "G"),
            Self::Cytosine => write!(f, "C"),
            Self::Adenine => write!(f, "A"),
            Self::Thymine => write!(f, "T"),
        }
    }
}

impl TryFrom<char> for Base {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'G' => Ok(Self::Guanine),
            'C' => Ok(Self::Cytosine),
            'A' => Ok(Self::Adenine),
            'T' => Ok(Self::Thymine),
            _ => Err(format!("{value:?} is not a valid base")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Dna(Vec<Base>);

impl Dna {
    pub fn similarity(&self, parents: [&Self; 2]) -> usize {
        let (p1_similarity, p2_similarity) = self
            .into_iter()
            .zip(parents[0].into_iter().zip(parents[1]))
            .fold((0, 0), |mut acc, (s, (p1, p2))| {
                if s == p1 {
                    acc.0 += 1;
                }
                if s == p2 {
                    acc.1 += 1;
                }
                acc
            });
        p1_similarity * p2_similarity
    }
}

impl Deref for Dna {
    type Target = [Base];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Dna {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for base in &self.0 {
            write!(f, "{base}")?;
        }
        Ok(())
    }
}

impl FromIterator<Base> for Dna {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Base>,
    {
        Self(iter.into_iter().collect())
    }
}

impl FromStr for Dna {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.chars().map(Base::try_from).collect()
    }
}

impl IntoIterator for Dna {
    type IntoIter = <Vec<Base> as IntoIterator>::IntoIter;
    type Item = Base;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'d> IntoIterator for &'d Dna {
    type IntoIter = <&'d Vec<Base> as IntoIterator>::IntoIter;
    type Item = &'d Base;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

fn find_child(dnas: [&Dna; 3]) -> Option<usize> {
    let mut _0_is_child = true;
    let mut _1_is_child = true;
    let mut _2_is_child = true;
    #[expect(
        clippy::needless_range_loop,
        reason = "Indexing *all* elements of an array, not just a single array."
    )]
    for idx in 0..dnas[0].len() {
        if ![dnas[0][idx], dnas[1][idx]].contains(&dnas[2][idx]) {
            _2_is_child = false;
            if !_0_is_child && !_1_is_child {
                return None;
            }
        }
        if ![dnas[0][idx], dnas[2][idx]].contains(&dnas[1][idx]) {
            _1_is_child = false;
            if !_0_is_child && !_2_is_child {
                return None;
            }
        }
        if ![dnas[1][idx], dnas[2][idx]].contains(&dnas[0][idx]) {
            _0_is_child = false;
            if !_1_is_child && !_2_is_child {
                return None;
            }
        }
    }
    Some(if _0_is_child {
        0
    } else if _1_is_child {
        1
    } else {
        // if _2_is_child {
        2
    })
}

fn read_dnas(input: &mut dyn BufRead) -> io::Result<Vec<(usize, Dna)>> {
    input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.split_once(':')
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Each line of notes must contain a colon",
                        )
                    })
                    .and_then(|(scale_number, dna)| {
                        dna.parse::<Dna>()
                            .map_err(|e| {
                                io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("{e:?}: {dna:?} is not a valid DNA sequence"),
                                )
                            })
                            .and_then(|dna| {
                                scale_number
                                    .parse::<usize>()
                                    .map(|scale_number| (scale_number, dna))
                                    .map_err(|e| {
                                        io::Error::new(
                                            io::ErrorKind::InvalidData,
                                            format!("{e:?}: Can't parse {scale_number:?} as usize"),
                                        )
                                    })
                            })
                    })
            })
        })
        .collect()
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let dnas = read_dnas(input)?
        .into_iter()
        .map(|(_, dna)| dna)
        .collect::<Vec<_>>();
    let idx = find_child([&dnas[0], &dnas[1], &dnas[2]]).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "The notes do not contain a child and their parents",
        )
    })?;
    let (parents, child) = match idx {
        0 => ([&dnas[1], &dnas[2]], &dnas[0]),
        1 => ([&dnas[0], &dnas[2]], &dnas[1]),
        2 => ([&dnas[0], &dnas[1]], &dnas[2]),
        _ => unreachable!("find_child should always returns `Some(0|1|2)|None`. Got {idx}"),
    };
    Ok(parents
        .into_iter()
        .map(|parent| {
            parent
                .into_iter()
                .zip(child)
                .filter(|&(base1, base2)| base1 == base2)
                .count()
        })
        .product())
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let dnas = read_dnas(input)?
        .into_iter()
        .map(|(_, dna)| dna)
        .collect::<Vec<_>>();
    let dnas = &dnas[..];
    let triples = (0..dnas.len()).flat_map(|i| {
        ((i + 1)..dnas.len())
            .flat_map(move |j| ((j + 1)..dnas.len()).map(move |k| [&dnas[i], &dnas[j], &dnas[k]]))
    });
    Ok(triples
        .into_iter()
        .filter_map(|triple| match find_child(triple)? {
            0 => Some(triple[0].similarity([triple[1], triple[2]])),
            1 => Some(triple[1].similarity([triple[0], triple[2]])),
            2 => Some(triple[2].similarity([triple[0], triple[1]])),
            _ => None,
        })
        .sum())
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let dnas = read_dnas(input)?;
    let dnas = &dnas[..];
    let start = Instant::now();
    let triples = (0..dnas.len()).flat_map(|i| {
        ((i + 1)..dnas.len()).flat_map(move |j| {
            ((j + 1)..dnas.len()).map(move |k| {
                (
                    [dnas[i].0, dnas[j].0, dnas[k].0],
                    [&dnas[i].1, &dnas[j].1, &dnas[k].1],
                )
            })
        })
    });
    let relationships = triples
        .into_iter()
        .filter_map(|(ids, triple)| match find_child(triple)? {
            0 => Some((ids[0], [ids[1], ids[2]])),
            1 => Some((ids[1], [ids[0], ids[2]])),
            2 => Some((ids[2], [ids[0], ids[1]])),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut unvisited = dnas.iter().map(|&(id, _)| id).collect::<Vec<_>>();
    let mut trees = vec![vec![]];
    while let Some(next_root) = unvisited.pop() {
        let mut frontier = vec![next_root];
        while let Some(next) = frontier.pop() {
            let current = trees.last_mut().unwrap();
            current.push(next);
            let neighbors = relationships
                .iter()
                .filter_map(|&(child, [p1, p2])| {
                    Some([child, p1, p2])
                        .filter(|set| set.contains(&next))
                        .map(|set| set.into_iter().filter(|&id| id != next))
                })
                .flatten()
                .collect::<Vec<_>>();
            frontier.extend(unvisited.extract_if(.., |id| neighbors.contains(&*id)));
        }
        trees.push(vec![]);
    }
    let ret = trees
        .into_iter()
        .max_by_key(Vec::len)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "No scales found in notes"))?
        .into_iter()
        .sum();
    eprintln!("Computation took {:?}", Instant::now() - start);
    Ok(ret)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 9 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_09-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 9 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_09-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 9 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_09-3.txt"
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
            "1:CAAGCGCTAAGTTCGCTGGATGTGTGCCCGCG\n",
            "2:CTTGAATTGGGCCGTTTACCTGGTTTAACCAT\n",
            "3:CTAGCGCTGAGCTGGCTGCCTGGTTGACCGCG\n",
        );
        let expected = 414;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "1:GCAGGCGAGTATGATACCCGGCTAGCCACCCC\n",
            "2:TCTCGCGAGGATATTACTGGGCCAGACCCCCC\n",
            "3:GGTGGAACATTCGAAAGTTGCATAGGGTGGTG\n",
            "4:GCTCGCGAGTATATTACCGAACCAGCCCCTCA\n",
            "5:GCAGCTTAGTATGACCGCCAAATCGCGACTCA\n",
            "6:AGTGGAACCTTGGATAGTCTCATATAGCGGCA\n",
            "7:GGCGTAATAATCGGATGCTGCAGAGGCTGCTG\n",
        );
        let expected = 1245;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "1:GCAGGCGAGTATGATACCCGGCTAGCCACCCC\n",
            "2:TCTCGCGAGGATATTACTGGGCCAGACCCCCC\n",
            "3:GGTGGAACATTCGAAAGTTGCATAGGGTGGTG\n",
            "4:GCTCGCGAGTATATTACCGAACCAGCCCCTCA\n",
            "5:GCAGCTTAGTATGACCGCCAAATCGCGACTCA\n",
            "6:AGTGGAACCTTGGATAGTCTCATATAGCGGCA\n",
            "7:GGCGTAATAATCGGATGCTGCAGAGGCTGCTG\n",
        );
        let expected = 12;
        let actual = part3(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        const TEST_DATA_2: &str = concat!(
            "1:GCAGGCGAGTATGATACCCGGCTAGCCACCCC\n",
            "2:TCTCGCGAGGATATTACTGGGCCAGACCCCCC\n",
            "3:GGTGGAACATTCGAAAGTTGCATAGGGTGGTG\n",
            "4:GCTCGCGAGTATATTACCGAACCAGCCCCTCA\n",
            "5:GCAGCTTAGTATGACCGCCAAATCGCGACTCA\n",
            "6:AGTGGAACCTTGGATAGTCTCATATAGCGGCA\n",
            "7:GGCGTAATAATCGGATGCTGCAGAGGCTGCTG\n",
            "8:GGCGTAAAGTATGGATGCTGGCTAGGCACCCG\n",
        );
        let expected = 36;
        let actual = part3(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
