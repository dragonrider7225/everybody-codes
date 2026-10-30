use std::{
    cmp::Reverse,
    fs::File,
    io::{self, BufRead, BufReader},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum QuestPart {
    One = 1,
    Two = 2,
    Three = 3,
}

impl FromStr for QuestPart {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>()
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Non-numeric value for quest part",
                )
            })
            .and_then(Self::try_from)
    }
}

impl TryFrom<u32> for QuestPart {
    type Error = io::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid quest part: {value:?}"),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct QuestPartResult {
    id: usize,
    quest: usize,
    part: QuestPart,
    time: u32,
}

impl FromStr for QuestPartResult {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn get_field<T>(
            ctx: &str,
            fields: &[(&str, &str)],
            name: &str,
            err_convert: impl FnOnce(&str, &str, <T as FromStr>::Err) -> io::Error,
        ) -> io::Result<T>
        where
            T: FromStr,
        {
            fields
                .iter()
                .find_map(|&(part_id, part_value)| (part_id == name).then_some(part_value))
                .map(|value| value.parse::<T>().map_err(|e| err_convert(ctx, name, e)))
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Missing field {name:?} in {ctx:?}"),
                    )
                })
                .flatten()
        }

        fn number_shim(ctx: &str, name: &str, _: std::num::ParseIntError) -> io::Error {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Non-numeric field {name:?} in {ctx:?}"),
            )
        }

        let parts = s
            .split_ascii_whitespace()
            .map(|part| {
                part.split_once('=').ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Field {part:?} in {s:?} is missing '='"),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id: get_field(s, &parts, "id", number_shim)?,
            quest: get_field(s, &parts, "quest", number_shim)?,
            part: get_field(s, &parts, "part", |_, _, e| e)?,
            time: get_field(s, &parts, "time", number_shim)?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct QuestResult {
    id: usize,
    #[cfg(debug_assertions)]
    quest: usize,
    times: [u32; 3],
}

impl QuestResult {
    pub fn new(id: usize, quest: usize) -> Self {
        #[cfg(not(debug_assertions))]
        let _ = quest;
        Self {
            id,
            #[cfg(debug_assertions)]
            quest,
            times: [0; 3],
        }
    }

    fn add_result(&mut self, result: QuestPartResult) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(self.id, result.id);
            assert_eq!(self.quest, result.quest);
        }
        self.times[result.part as u32 as usize - 1] = result.time;
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<u32> {
    let mut quest_results = input
        .lines()
        .map(|line| line.and_then(|line| line.parse::<QuestPartResult>()))
        .try_fold(vec![], |mut acc, result| {
            result.map(|result| {
                while acc.len() <= result.id {
                    acc.push(0);
                }
                acc[result.id] += result.time;
                acc
            })
        })?;
    quest_results.sort();
    debug_assert_eq!(quest_results[0], 0);
    Ok(quest_results
        .into_iter()
        .enumerate()
        .inspect(|(position, time)| {
            #[cfg(not(debug_assertions))]
            let _ = (position, time);
            #[cfg(debug_assertions)]
            eprintln!("{position}: {time}")
        })
        .map(|(position, time)| position as u32 * time)
        .sum())
}

fn part2(input: &mut dyn BufRead) -> io::Result<u32> {
    let mut quest_results = input
        .lines()
        .map(|line| line.and_then(|line| line.parse::<QuestPartResult>()))
        .try_fold(vec![(Reverse(20 * 3), 0)], |mut acc, result| {
            result.map(|result| {
                while acc.len() <= result.id {
                    acc.push((Reverse(0), 0));
                }
                acc[result.id].0 .0 += 1;
                acc[result.id].1 += result.time;
                acc
            })
        })?;
    #[cfg(debug_assertions)]
    {
        for (id, &(Reverse(num_parts), time)) in quest_results.iter().enumerate() {
            eprintln!("id={id}: {num_parts} parts, total time = {time}");
        }
    }
    quest_results.sort();
    #[cfg(debug_assertions)]
    {
        for (position, &(Reverse(num_parts), time)) in quest_results.iter().enumerate() {
            eprintln!("position {position}: {num_parts} parts, total time = {time}");
        }
    }
    Ok(quest_results
        .into_iter()
        .enumerate()
        .fold(0, |acc, (position, (_, time))| acc + position as u32 * time))
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut quest_results = input
        .lines()
        .map(|line| line.and_then(|line| line.parse::<QuestPartResult>()))
        .try_fold(vec![], |mut acc, result| {
            result.map(|result| {
                while acc.len() <= result.quest {
                    acc.push(vec![]);
                }
                let results_by_participant = &mut acc[result.quest];
                while results_by_participant.len() <= result.id {
                    results_by_participant
                        .push(QuestResult::new(results_by_participant.len(), result.quest));
                }
                results_by_participant[result.id].add_result(result);
                acc
            })
        })?;
    #[cfg(debug_assertions)]
    {
        let results_by_id = quest_results.iter().fold(vec![], |mut acc, quest_results| {
            quest_results.iter().for_each(|result| {
                while acc.len() <= result.id {
                    acc.push(vec![]);
                }
                let participant_results = &mut acc[result.id];
                while participant_results.len() <= result.quest {
                    participant_results
                        .push(QuestResult::new(result.id, participant_results.len()));
                }
                participant_results[result.quest] = *result;
            });
            acc
        });
        for (id, results) in results_by_id.iter().enumerate() {
            let (num_parts, time) = results.iter().fold((0, 0), |(num_parts, time), result| {
                let (quest_parts, quest_time) =
                    result
                        .times
                        .into_iter()
                        .fold((0, 0), |(quest_parts, quest_time), time| {
                            if time > 0 {
                                (quest_parts + 1, quest_time + time)
                            } else {
                                (quest_parts, quest_time)
                            }
                        });
                (num_parts + quest_parts, time + quest_time)
            });
            eprintln!("id={id}: {num_parts} parts, total time = {time}");
        }
    }
    quest_results.swap_remove(0);
    let finish_times_by_participant_by_part_by_quest =
        quest_results
            .into_iter()
            .enumerate()
            .fold(vec![], |mut acc, (quest, quest_results)| {
                while acc.len() <= quest {
                    acc.push([const { vec![] }; 3]);
                }
                let finish_times_by_participant_by_part = &mut acc[quest];
                for result in quest_results {
                    while finish_times_by_participant_by_part[0].len() <= result.id {
                        for positions in finish_times_by_participant_by_part.iter_mut() {
                            positions.push(0);
                        }
                    }
                    finish_times_by_participant_by_part
                        .iter_mut()
                        .map(|times_by_participant| &mut times_by_participant[result.id])
                        .zip(result.times)
                        .take_while(|&(_, part_time)| part_time > 0)
                        .fold(0, |mut total_time, (finish_time, part_time)| {
                            total_time += part_time;
                            *finish_time = total_time;
                            total_time
                        });
                }
                acc
            });
    let participants_and_finish_times_by_participant_by_part_by_quest =
        finish_times_by_participant_by_part_by_quest
            .into_iter()
            .map(|finish_times_by_participant_by_part| {
                let mut ret = [const { vec![] }; 3];
                finish_times_by_participant_by_part
                    .into_iter()
                    .enumerate()
                    .for_each(|(part, finish_times_by_participant)| {
                        ret[part] = finish_times_by_participant
                            .into_iter()
                            .enumerate()
                            .collect()
                    });
                ret
            })
            .collect::<Vec<_>>();
    let mut points_by_participant = participants_and_finish_times_by_participant_by_part_by_quest
        .into_iter()
        .flat_map(|participants_and_finish_times_by_participant_by_part| {
            participants_and_finish_times_by_participant_by_part
                .into_iter()
                .enumerate()
                .flat_map(
                    |(part_idx, mut participants_and_finish_times_by_participant)| {
                        let max_points = (part_idx + 1) * 50;
                        participants_and_finish_times_by_participant
                            .sort_unstable_by_key(|&(_, finish_time)| finish_time);
                        participants_and_finish_times_by_participant
                            .into_iter()
                            .filter(|&(_, time)| time != 0)
                            .enumerate()
                            .fold(
                                (vec![], 0),
                                move |(mut acc, last_time): (Vec<(_, usize)>, _),
                                      (idx, (participant, time))| {
                                    if time == last_time {
                                        acc.push((participant, acc.last().unwrap().1));
                                    } else {
                                        acc.push((participant, max_points.saturating_sub(idx)));
                                    }
                                    (acc, time)
                                },
                            )
                            .0
                    },
                )
        })
        .fold(
            vec![],
            |mut points_by_participant, (participant, points)| {
                while points_by_participant.len() <= participant {
                    points_by_participant.push(0);
                }
                points_by_participant[participant] += points;
                points_by_participant
            },
        );
    #[cfg(debug_assertions)]
    {
        let mut participants_and_points_by_position = points_by_participant
            .iter()
            .copied()
            .enumerate()
            .collect::<Vec<_>>();
        participants_and_points_by_position.sort_unstable_by_key(|&(_, points)| Reverse(points));
        participants_and_points_by_position
            .into_iter()
            .enumerate()
            .fold(
                (usize::MAX, 0),
                |(last_points, last_position), (idx, (participant, points))| {
                    let position = if points < last_points {
                        idx + 1
                    } else {
                        last_position
                    };
                    eprintln!("Position {position}: id={participant}, {points} points");
                    (points, position)
                },
            );
    }
    points_by_participant.sort_unstable_by_key(|&points| Reverse(points));
    Ok(points_by_participant
        .into_iter()
        .filter(|&points| points > 0)
        .enumerate()
        .fold(
            (0, (usize::MAX, 0)),
            |(acc, (last_points, last_position)), (idx, points)| {
                let position = if points < last_points {
                    idx + 1
                } else {
                    last_position
                };
                (acc + position * points, (points, position))
            },
        )
        .0)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Lord of the wings Quest 4 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "lord-of-the-wings_04-1.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 4 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "lord-of-the-wings_04-2.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 4 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "lord-of-the-wings_04-3.txt"
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
            "id=1 quest=1 part=1 time=187\n",
            "id=3 quest=1 part=1 time=413\n",
            "id=5 quest=1 part=3 time=744\n",
            "id=2 quest=1 part=3 time=745\n",
            "id=4 quest=1 part=2 time=558\n",
            "id=1 quest=1 part=2 time=299\n",
            "id=1 quest=1 part=3 time=362\n",
            "id=4 quest=1 part=3 time=1268\n",
            "id=4 quest=1 part=1 time=457\n",
            "id=5 quest=1 part=1 time=257\n",
            "id=2 quest=1 part=1 time=192\n",
            "id=3 quest=1 part=3 time=1168\n",
            "id=5 quest=1 part=2 time=346\n",
            "id=3 quest=1 part=2 time=455\n",
            "id=2 quest=1 part=2 time=562\n",
        );
        let expected = 27_598;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "id=2 quest=1 part=1 time=100\n",
            "id=4 quest=2 part=1 time=150\n",
            "id=1 quest=2 part=1 time=140\n",
            "id=1 quest=1 part=2 time=180\n",
            "id=2 quest=2 part=1 time=130\n",
            "id=4 quest=1 part=1 time=110\n",
            "id=1 quest=2 part=2 time=210\n",
            "id=2 quest=2 part=2 time=240\n",
            "id=1 quest=2 part=3 time=330\n",
            "id=2 quest=1 part=2 time=170\n",
            "id=3 quest=1 part=1 time=90\n",
            "id=4 quest=1 part=2 time=175\n",
            "id=3 quest=2 part=1 time=120\n",
            "id=1 quest=1 part=1 time=120\n",
            "id=3 quest=2 part=2 time=220\n",
            "id=2 quest=2 part=3 time=350\n",
            "id=3 quest=1 part=2 time=160\n",
        );
        let expected = 6470;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "id=2 quest=1 part=1 time=100\n",
            "id=4 quest=2 part=1 time=150\n",
            "id=1 quest=2 part=1 time=140\n",
            "id=1 quest=1 part=2 time=180\n",
            "id=2 quest=2 part=1 time=130\n",
            "id=4 quest=1 part=1 time=110\n",
            "id=1 quest=2 part=2 time=210\n",
            "id=2 quest=2 part=2 time=240\n",
            "id=1 quest=2 part=3 time=330\n",
            "id=2 quest=1 part=2 time=170\n",
            "id=3 quest=1 part=1 time=90\n",
            "id=4 quest=1 part=2 time=175\n",
            "id=3 quest=2 part=1 time=120\n",
            "id=1 quest=1 part=1 time=120\n",
            "id=3 quest=2 part=2 time=220\n",
            "id=2 quest=2 part=3 time=350\n",
            "id=3 quest=1 part=2 time=160\n",
        );
        let expected = 2998;
        let actual = part3(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);

        const TEST_DATA_2: &str = concat!(
            "id=2 quest=1 part=1 time=90\n",
            "id=4 quest=2 part=1 time=150\n",
            "id=1 quest=2 part=1 time=130\n",
            "id=1 quest=1 part=2 time=180\n",
            "id=2 quest=2 part=1 time=130\n",
            "id=4 quest=1 part=1 time=110\n",
            "id=1 quest=2 part=2 time=210\n",
            "id=2 quest=2 part=2 time=220\n",
            "id=1 quest=2 part=3 time=330\n",
            "id=2 quest=1 part=2 time=170\n",
            "id=3 quest=1 part=1 time=90\n",
            "id=4 quest=1 part=2 time=170\n",
            "id=3 quest=2 part=1 time=120\n",
            "id=1 quest=1 part=1 time=120\n",
            "id=3 quest=2 part=2 time=220\n",
            "id=2 quest=2 part=3 time=350\n",
            "id=3 quest=1 part=2 time=170\n",
        );
        let expected = 3004;
        let actual = part3(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
