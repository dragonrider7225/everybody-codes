use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader, Write},
    iter,
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct PlantId(usize);

impl Display for PlantId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Plant {}", self.0)
    }
}

impl FromStr for PlantId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
            .map(Self)
            .map_err(|e| format!("{e:?}: {s:.50?} must be a number"))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Plant {
    Free,
    Connections {
        thickness: usize,
        connections: Vec<(PlantId, isize)>,
    },
}

impl Plant {
    fn read(input: &mut dyn BufRead) -> io::Result<Option<(PlantId, Self)>> {
        let mut lines = input
            .lines()
            .take_while(|line| !line.as_ref().is_ok_and(|line| line.is_empty()));
        let Some(first_line) = lines.next() else {
            return Ok(None);
        };
        let first_line = first_line?;
        let (id, thickness) = first_line
            .strip_prefix("Plant ")
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Plant definition must begin with \"Plant \"",
                )
            })?
            .strip_suffix(":")
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "First line of plant definition must end with \":\"",
                )
            })?
            .split_once(" with thickness ")
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "First line of plant definition must match /Plant \\d+ with thickness \\d+:/",
                )
            })?;
        let id = id
            .parse::<PlantId>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let thickness = thickness.parse::<usize>().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{e:?}: {thickness:?} must be a number"),
            )
        })?;
        let connections = lines
            .map(|line| line.and_then(|line| {
                let Some((target, thickness)) = line.split_once(" with thickness ") else {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Each line of a plant definition after the first must contain \" with thickness \""));
                };
                let thickness = thickness.parse::<isize>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}: {thickness:?} must be a number")))?;
                let target = match target.strip_prefix("- branch to Plant ") {
                    Some(id) => Some(id.parse::<PlantId>().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?),
                    None => None,
                };
                Ok((target, thickness))
            }))
            .collect::<io::Result<Vec<_>>>()?;
        let plant = if connections[0].0.is_none() {
            Self::Free
        } else {
            Self::Connections {
                thickness,
                connections: connections.into_iter().map(|(target, thickness)| (target.expect("A plant with any branch connected to another plant must have all branches connected to other plants"), thickness)).collect(),
            }
        };
        Ok(Some((id, plant)))
    }

    fn read_plants(input: &mut dyn BufRead) -> io::Result<HashMap<PlantId, Self>> {
        iter::from_fn(|| Plant::read(input).transpose()).collect()
    }

    pub fn thickness(&self) -> usize {
        match self {
            Self::Free => 1,
            Self::Connections { thickness, .. } => *thickness,
        }
    }

    pub fn sources(&self) -> impl Iterator<Item = PlantId> {
        let base = match self {
            Self::Free => [].iter(),
            Self::Connections { connections, .. } => connections.iter(),
        };
        base.map(|&(id, _)| id)
    }

    pub fn energy_received<'p, F>(
        &self,
        id: PlantId,
        plants: &mut F,
        cache: &mut HashMap<PlantId, usize>,
    ) -> usize
    where
        F: FnMut(PlantId) -> &'p Plant,
    {
        if let Some(received) = cache.get(&id) {
            return *received;
        }
        let received = match self {
            Self::Free => 1,
            Self::Connections { connections, .. } => connections
                .iter()
                .map(|&(source_id, thickness)| {
                    thickness
                        * (plants(source_id).energy_emitted(source_id, plants, cache) as isize)
                })
                .sum(),
        };
        let received = received.max(0) as usize;
        cache.insert(id, received);
        received
    }

    pub fn energy_emitted<'p, F>(
        &self,
        id: PlantId,
        plants: &mut F,
        cache: &mut HashMap<PlantId, usize>,
    ) -> usize
    where
        F: FnMut(PlantId) -> &'p Plant,
    {
        let received = self.energy_received(id, plants, cache);
        if received >= self.thickness() {
            received
        } else {
            0
        }
    }
}

fn read_test_case(input: &mut dyn BufRead) -> Option<io::Result<HashMap<PlantId, usize>>> {
    input.lines().next().map(|line| {
        line.and_then(|line| {
            line.split(' ')
                .enumerate()
                .map(|(n, active)| match active {
                    "0" => Ok((PlantId(n + 1), 0usize)),
                    "1" => Ok((PlantId(n + 1), 1)),
                    _ => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Test cases must consist solely of '0', '1', and ' '.",
                    )),
                })
                .collect()
        })
    })
}

fn read_test_cases(
    input: &mut dyn BufRead,
) -> impl IntoIterator<Item = io::Result<HashMap<PlantId, usize>>> {
    iter::from_fn(|| read_test_case(input))
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let plants = Plant::read_plants(input)?;
    let (&final_id, final_plant) = plants
        .iter()
        .find(|&(&id, _)| {
            !plants
                .values()
                .any(|plant| plant.sources().any(|source| source == id))
        })
        .expect("There should be a plant which does not provide brightness to any other plant");
    Ok(final_plant.energy_received(final_id, &mut |id| &plants[&id], &mut HashMap::new()))
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let plants = Plant::read_plants(input)?;
    let (&final_id, final_plant) = plants
        .iter()
        .find(|&(&id, _)| {
            !plants
                .values()
                .any(|plant| plant.sources().any(|source| source == id))
        })
        .expect("There should be a plant which does not provide brightness to any other plant");
    read_test_cases(input)
        .into_iter()
        .map(|test_case| {
            test_case.map(|mut test_case| {
                final_plant.energy_emitted(final_id, &mut |id| &plants[&id], &mut test_case)
            })
        })
        .try_fold(0, |acc, n| Ok(acc + n?))
}

fn compress_test_case(test_case: &[usize]) -> u128 {
    test_case
        .iter()
        .rev()
        .fold(0u128, |acc, &n| 2 * acc + (n as u128))
}

fn write_network(mut network_out: impl Write, plants: &HashMap<PlantId, Plant>) -> io::Result<()> {
    writeln!(network_out, "strict digraph SoDaD_18_3 {{")?;
    writeln!(network_out, "color=white;")?;
    for (&PlantId(id), plant) in plants {
        writeln!(network_out, "_{id} [label={id}];")?;
        match plant {
            Plant::Free => {}
            Plant::Connections { connections, .. } => {
                for &(PlantId(source), strength) in connections {
                    write!(network_out, "_{source} -> _{id}")?;
                    if strength < 0 {
                        write!(network_out, " [color=red]")?;
                    }
                    writeln!(network_out, ";")?;
                }
            }
        }
    }
    writeln!(network_out, "}}")
}

fn part3(input: &mut dyn BufRead, network_out: impl Write) -> io::Result<usize> {
    let plants = Plant::read_plants(input)?;
    let (&final_id, final_plant) = plants
        .iter()
        .find(|&(&id, _)| {
            !plants
                .values()
                .any(|plant| plant.sources().any(|source| source == id))
        })
        .expect("There should be a plant which does not provide brightness to any other plant");
    let test_cases = read_test_cases(input)
        .into_iter()
        .map(|test_case| {
            test_case.map(|test_case| {
                (1..=test_case.len())
                    .map(PlantId)
                    .map(|n| test_case[&n])
                    .collect::<Vec<_>>()
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    write_network(network_out, &plants)?;
    eprintln!("This implementation assumes that every input node is either always added or always subtracted");
    eprintln!("The network has been written as a GraphViz graph");
    let num_inputs = test_cases[0].len();
    let best_case = (0..num_inputs)
        .map(|i| PlantId(i + 1))
        .map(|id| {
            let value = plants
                .values()
                .filter_map(|p| match p {
                    Plant::Free => None,
                    Plant::Connections { connections, .. } => Some(&connections[..]),
                })
                .flatten()
                .filter_map(|&(source, strength)| Some(strength).filter(|_| source == id))
                .fold(None, |acc, strength| {
                    let this = strength.cmp(&0);
                    match acc {
                        None => Some(this),
                        Some(acc) if acc == this => Some(acc),
                        Some(_) => Some(Ordering::Equal),
                    }
                })
                .unwrap_or(Ordering::Equal);
            match value {
                Ordering::Less => (id, Some(0)),
                Ordering::Equal => (id, None),
                Ordering::Greater => (id, Some(1)),
            }
        })
        .collect::<HashMap<_, _>>();
    let num_variable_inputs = best_case.len() - best_case.values().flatten().count();
    let best_output = if num_variable_inputs != 0 {
        let all_test_cases = (0..(1u128 << num_inputs)).map(|mut n| {
            let mut inputs = vec![];
            for _ in 0..num_inputs {
                if n.is_multiple_of(2u128) {
                    inputs.push(0);
                } else {
                    inputs.push(1);
                }
                n /= 2;
            }
            inputs
        });
        all_test_cases
            .filter_map(|test_case| {
                {
                    let test_case = compress_test_case(&test_case);
                    if test_case.is_multiple_of(100_000) {
                        eprintln!(
                            "Test case: {test_case} ({})",
                            test_case.checked_ilog2().unwrap_or(0)
                        );
                    }
                }
                let total = final_plant.energy_emitted(
                    final_id,
                    &mut |id| &plants[&id],
                    &mut test_case
                        .iter()
                        .enumerate()
                        .map(|(n, &active)| (PlantId(n + 1), active))
                        .collect(),
                );
                if total > 0 {
                    Some(total)
                } else {
                    None
                }
            })
            .max()
            .expect("There must be at least one way to make an output")
    } else {
        let best_output = final_plant.energy_emitted(
            final_id,
            &mut |id| &plants[&id],
            &mut best_case
                .iter()
                .map(|(&id, &value)| (id, value.unwrap()))
                .collect(),
        );
        dbg!(best_output);
        best_output
    };
    Ok(test_cases
        .into_iter()
        .map(|test_case| {
            final_plant.energy_emitted(
                final_id,
                &mut |id| &plants[&id],
                &mut test_case
                    .iter()
                    .enumerate()
                    .map(|(n, &active)| (PlantId(n + 1), active))
                    .collect(),
            )
        })
        .filter(|&total| total > 0)
        .map(|total| best_output - total)
        .sum())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 18 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_18-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 18 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_18-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 18 Part 3");
        let image = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open("SoDaD_18_3.gv")?;
        println!(
            "{}",
            part3(
                &mut BufReader::new(File::open("song-of-ducks-and-dragons_18-3.txt")?),
                image,
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
        const TEST_DATA: &str = concat!(
            "Plant 1 with thickness 1:\n",
            "- free branch with thickness 1\n",
            "\n",
            "Plant 2 with thickness 1:\n",
            "- free branch with thickness 1\n",
            "\n",
            "Plant 3 with thickness 1:\n",
            "- free branch with thickness 1\n",
            "\n",
            "Plant 4 with thickness 17:\n",
            "- branch to Plant 1 with thickness 15\n",
            "- branch to Plant 2 with thickness 3\n",
            "\n",
            "Plant 5 with thickness 24:\n",
            "- branch to Plant 2 with thickness 11\n",
            "- branch to Plant 3 with thickness 13\n",
            "\n",
            "Plant 6 with thickness 15:\n",
            "- branch to Plant 3 with thickness 14\n",
            "\n",
            "Plant 7 with thickness 10:\n",
            "- branch to Plant 4 with thickness 15\n",
            "- branch to Plant 5 with thickness 21\n",
            "- branch to Plant 6 with thickness 34\n",
        );
        let expected = 774;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "Plant 1 with thickness 1:\n",
            "- free branch with thickness 1\n",
            "\n",
            "Plant 2 with thickness 1:\n",
            "- free branch with thickness 1\n",
            "\n",
            "Plant 3 with thickness 1:\n",
            "- free branch with thickness 1\n",
            "\n",
            "Plant 4 with thickness 10:\n",
            "- branch to Plant 1 with thickness -25\n",
            "- branch to Plant 2 with thickness 17\n",
            "- branch to Plant 3 with thickness 12\n",
            "\n",
            "Plant 5 with thickness 14:\n",
            "- branch to Plant 1 with thickness 14\n",
            "- branch to Plant 2 with thickness -26\n",
            "- branch to Plant 3 with thickness 15\n",
            "\n",
            "Plant 6 with thickness 150:\n",
            "- branch to Plant 4 with thickness 5\n",
            "- branch to Plant 5 with thickness 6\n",
            "\n",
            "\n",
            "1 0 1\n",
            "0 0 1\n",
            "0 1 1\n",
        );
        let expected = 324;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "Plant 1 with thickness 1:\n",
            "- free branch with thickness 1\n",
            "\n",
            "Plant 2 with thickness 1:\n",
            "- free branch with thickness 1\n",
            "\n",
            "Plant 3 with thickness 1:\n",
            "- free branch with thickness 1\n",
            "\n",
            "Plant 4 with thickness 1:\n",
            "- free branch with thickness 1\n",
            "\n",
            "Plant 5 with thickness 8:\n",
            "- branch to Plant 1 with thickness -8\n",
            "- branch to Plant 2 with thickness 11\n",
            "- branch to Plant 3 with thickness 13\n",
            "- branch to Plant 4 with thickness -7\n",
            "\n",
            "Plant 6 with thickness 7:\n",
            "- branch to Plant 1 with thickness 14\n",
            "- branch to Plant 2 with thickness -9\n",
            "- branch to Plant 3 with thickness 12\n",
            "- branch to Plant 4 with thickness 9\n",
            "\n",
            "Plant 7 with thickness 23:\n",
            "- branch to Plant 5 with thickness 17\n",
            "- branch to Plant 6 with thickness 18\n",
            "\n",
            "\n",
            "0 1 0 0\n",
            "0 1 0 1\n",
            "0 1 1 1\n",
            "1 1 0 1\n",
        );
        let expected = 946;
        let actual = part3(&mut Cursor::new(TEST_DATA), io::sink())?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
