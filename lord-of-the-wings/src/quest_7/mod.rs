use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    str::FromStr,
};

#[derive(Clone, Debug)]
struct Cloud {
    id: usize,
    energy: u64,
    connections: Vec<usize>,
}

impl FromStr for Cloud {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn get_field<T>(
            fields: &HashMap<&str, &str>,
            name: &str,
            from_str: impl FnOnce(&str) -> Result<T, String>,
        ) -> Result<T, String> {
            fields
                .get(name)
                .copied()
                .ok_or_else(|| format!("Missing field {name:?}"))
                .and_then(from_str)
        }

        let fields = s
            .split(", ")
            .map(|field| {
                field
                    .split_once('=')
                    .ok_or_else(|| format!("No {:?} in {field:?}", '='))
            })
            .try_fold(HashMap::new(), |mut acc, part| {
                let (name, value) = part?;
                if acc.insert(name, value).is_some() {
                    return Err(format!("Duplicate field {name:?} in {s:?}"));
                }
                Ok(acc)
            })?;
        if fields.len() > 3 {
            return Err(format!("Too many fields in cloud summary {s:?}"));
        }
        let id = get_field(&fields, "id", |id| {
            id.parse().map_err(|_| format!("Non-numeric id {id:?}"))
        })
        .map_err(|e| format!("{e} in cloud summary {s:?}"))?;
        let energy = get_field(&fields, "energy", |energy| {
            energy
                .parse()
                .map_err(|_| format!("Non-numeric energy amount {energy:?}"))
        })
        .map_err(|e| format!("{e} in cloud summary {s:?}"))?;
        let connections = get_field(&fields, "connections", |connections| {
            connections
                .split(',')
                .map(|id| {
                    id.parse::<usize>()
                        .map_err(|_| format!("Non-numeric id {id:?}"))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(|e| format!("{e} in cloud summary {s:?}"))?;
        Ok(Self {
            id,
            energy,
            connections,
        })
    }
}

fn apply_impulse(
    clouds: &mut HashMap<usize, Cloud>,
    target: usize,
    impulse: u64,
    energy_limit: u64,
) {
    let mut impulses_to = [((target, impulse), 1)]
        .into_iter()
        .collect::<HashMap<_, _>>();
    while !impulses_to.is_empty() {
        std::mem::take(&mut impulses_to)
            .into_iter()
            .filter(|&((_, impulse), _)| impulse > 0)
            .for_each(|((cloud_id, impulse), impulse_count)| {
                if let Some(cloud) = clouds.get_mut(&cloud_id) {
                    cloud.energy += impulse * impulse_count;
                    clouds[&cloud_id]
                        .connections
                        .iter()
                        .copied()
                        .filter(|cloud_id| clouds.contains_key(cloud_id))
                        .for_each(|cloud_id| {
                            *impulses_to.entry((cloud_id, impulse - 1)).or_insert(0) +=
                                impulse_count;
                        });
                }
            });
        for cloud in clouds
            .extract_if(|_, cloud| cloud.energy > energy_limit)
            .map(|(_, cloud)| cloud)
        {
            cloud.connections.into_iter().for_each(|cloud_id| {
                *impulses_to.entry((cloud_id, 10)).or_insert(0) += 1;
            });
        }
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<u64> {
    let mut clouds = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Cloud>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
            .map(|cloud| (cloud.id, cloud))
        })
        .collect::<io::Result<HashMap<_, _>>>()?;
    let target = clouds
        .values()
        .max_by_key(|cloud| cloud.connections.len())
        .unwrap()
        .id;
    apply_impulse(&mut clouds, target, 5, u64::MAX);
    Ok(clouds.into_values().map(|cloud| cloud.energy).sum())
}

fn part2(input: &mut dyn BufRead) -> io::Result<u64> {
    let mut clouds = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Cloud>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
            .map(|cloud| (cloud.id, cloud))
        })
        .collect::<io::Result<HashMap<_, _>>>()?;
    let target = clouds
        .values()
        .max_by_key(|cloud| cloud.connections.len())
        .unwrap()
        .id;
    apply_impulse(&mut clouds, target, 10, u64::MAX);
    Ok(clouds.into_values().map(|cloud| cloud.energy).sum())
}

fn part3(input: &mut dyn BufRead, energy_limit: u64) -> io::Result<u64> {
    let clouds = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Cloud>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
            .map(|cloud| (cloud.id, cloud))
        })
        .collect::<io::Result<HashMap<_, _>>>()?;
    Ok(clouds
        .keys()
        .map(|&target| {
            let mut clouds = clouds.clone();
            apply_impulse(&mut clouds, target, 10, energy_limit);
            clouds.into_values().map(|cloud| cloud.energy).sum()
        })
        .max()
        .unwrap_or(0))
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Lord of the wings Quest 7 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "lord-of-the-wings_07-1.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 7 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "lord-of-the-wings_07-2.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 7 Part 3");
        println!(
            "{}",
            part3(
                &mut BufReader::new(File::open("lord-of-the-wings_07-3.txt")?),
                1_000_000,
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
            "id=1, energy=5, connections=2,5,7\n",
            "id=2, energy=9, connections=6,7\n",
            "id=3, energy=3, connections=4\n",
            "id=4, energy=8, connections=3\n",
            "id=5, energy=1, connections=6,7\n",
            "id=6, energy=7, connections=1,7\n",
            "id=7, energy=6, connections=5\n",
        );
        let expected = 102;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "id=1, energy=5, connections=2,5,7\n",
            "id=2, energy=9, connections=6,7\n",
            "id=3, energy=3, connections=4\n",
            "id=4, energy=8, connections=3\n",
            "id=5, energy=1, connections=6,7\n",
            "id=6, energy=7, connections=1,7\n",
            "id=7, energy=6, connections=5\n",
        );
        let expected = 1316;
        let actual = part2(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);

        const TEST_DATA_2: &str = concat!(
            "id=1, energy=5, connections=2,3,5,7\n",
            "id=2, energy=9, connections=1,3,4,5,6,7\n",
            "id=3, energy=3, connections=1,2,4,6,7\n",
            "id=4, energy=8, connections=1,2,7\n",
            "id=5, energy=1, connections=2,3,4,6\n",
            "id=6, energy=7, connections=1,2,3,4\n",
            "id=7, energy=6, connections=1,2,4,6\n",
        );
        let expected = 1214633;
        let actual = part2(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "id=1, energy=5, connections=2,5,7\n",
            "id=2, energy=9, connections=6,7\n",
            "id=3, energy=3, connections=4\n",
            "id=4, energy=8, connections=3\n",
            "id=5, energy=1, connections=6,7\n",
            "id=6, energy=7, connections=1,7\n",
            "id=7, energy=6, connections=5\n",
        );
        let expected = 187;
        let actual = part3(&mut Cursor::new(TEST_DATA), 100)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
