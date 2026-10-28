use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    mem,
    str::FromStr,
};

#[derive(Debug)]
struct Orderings {
    inner: Vec<(u64, Vec<u64>)>,
}

impl Orderings {
    pub fn get(&self, alloy1: u64, alloy2: u64) -> Option<Ordering> {
        self.inner.iter().find_map(|&(id, ref harder_alloys)| {
            if id == alloy1 && harder_alloys.contains(&alloy2) {
                Some(Ordering::Less)
            } else if id == alloy2 && harder_alloys.contains(&alloy1) {
                Some(Ordering::Greater)
            } else {
                None
            }
        })
    }

    pub fn checksum(&mut self) -> u64 {
        self.inner
            .sort_unstable_by_key(|(_, greaters)| greaters.len());
        self.inner
            .iter()
            .map(|&(alloy_id, _)| alloy_id)
            .fold(0, |acc, alloy_id| {
                acc * 10u64.pow(1 + alloy_id.checked_ilog10().unwrap_or(0)) + alloy_id
            })
    }

    /// Adds the ordering `alloy1 < alloy2` to this set. Returns whether the set was modified by
    /// this call.
    pub fn insert(&mut self, alloy1: u64, alloy2: u64) -> bool {
        let to_modify = self
            .inner
            .iter_mut()
            .filter(|&&mut (id, _)| id == alloy1 || id == alloy2)
            .map(|&mut (id, ref mut harder_alloys)| (id, harder_alloys))
            .collect::<Vec<_>>();
        let mut ret = false;
        for (id, harder_alloys) in to_modify {
            if id == alloy1 {
                if !harder_alloys.contains(&alloy2) {
                    ret = true;
                    harder_alloys.push(alloy2);
                }
            } else if id == alloy2 {
                match harder_alloys.iter().position(|&id| id == alloy2) {
                    None => {}
                    Some(idx) => {
                        ret = true;
                        harder_alloys.swap_remove(idx);
                    }
                }
            } else {
                unreachable!()
            }
        }
        ret
    }

    /// Returns the number of alloys recorded in this set of orderings as being softer than alloy
    /// number `alloy_id`. If the set of orderings is not complete, the returned value may be too
    /// low.
    pub fn alloys_softer_than(&self, alloy_id: u64) -> Vec<u64> {
        self.inner
            .iter()
            .filter(|(_, harder_alloys)| harder_alloys.contains(&alloy_id))
            .map(|&(id, _)| id)
            .collect()
    }

    /// If `self` consists of `n` alloys with `n*(n - 1)/2 - 1` comparisons recorded between them,
    /// try to infer the missing comparison and return whether the attempt was successful.
    /// If `self` has any other number of recorded comparisons, give up and return false.
    pub fn infer_missing_comparison(&mut self) -> bool {
        let mut harder_counts = self
            .inner
            .iter()
            .map(|(alloy_id, harder_alloys)| (*alloy_id, harder_alloys.len()))
            .collect::<Vec<_>>();
        if harder_counts.iter().map(|&(_, count)| count).sum::<usize>() + 1
            != self.inner.len() * (self.inner.len() - 1) / 2
        {
            return false;
        }
        harder_counts.sort_unstable_by_key(|&(_, harder_count)| harder_count);
        for [(id1, count1), (id2, count2)] in harder_counts.array_windows::<2>() {
            if count1 == count2 {
                let softer_alloys1 = self.alloys_softer_than(*id1);
                let softer_alloys2 = self.alloys_softer_than(*id2);
                if softer_alloys1.len() == softer_alloys2.len() {
                    // Since the two alloys have the same number of harder alloys and the same
                    // number of softer alloys, the missing comparison is between them and no alloy
                    // is harder than one but softer than the other so it is impossible to infer
                    // which is harder.
                    return false;
                } else {
                    let (softer_id, softer_softer) = if softer_alloys1.len() < softer_alloys2.len()
                    {
                        (id1, softer_alloys1)
                    } else {
                        (id2, softer_alloys2)
                    };
                    let all_alloys = self
                        .inner
                        .iter()
                        .map(|&(id, _)| id)
                        .filter(|id| id != softer_id)
                        .collect::<Vec<_>>();
                    let softer_harder = self
                        .inner
                        .iter_mut()
                        .find(|&&mut (id, _)| id == *softer_id)
                        .map(|(_, harder_alloys)| harder_alloys)
                        .unwrap();
                    for id in all_alloys {
                        if !softer_softer.contains(softer_id) && !softer_harder.contains(softer_id)
                        {
                            softer_harder.push(id);
                            return true;
                        }
                    }
                }
                break;
            }
        }
        false
    }
}

impl Display for Orderings {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "[")?;
        for (less, greaters) in &self.inner {
            writeln!(f, "    Alloy {less}: {} harder alloys,", greaters.len())?;
        }
        writeln!(f, "]")
    }
}

impl FromStr for Orderings {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let orderings = s
            .split_ascii_whitespace()
            .map(|cmp| -> io::Result<_> {
                let (left, right) = cmp.split_once(['<', '>'])
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("{cmp:?} is an invalid comparison. All comparisons must be of the form 'a<b' or 'a>b'.")))?;
                let operator_idx = left.len();
                let left = left.parse::<u64>().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Both arguments to a comparison operator must be numbers: {left:?}")))?;
                let right = right.parse::<u64>().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Both arguments to a comparison operator must be numbers: {right:?}")))?;
                match &cmp[operator_idx..(operator_idx+1)] {
                    "<" => Ok((left, right)),
                    ">" => Ok((right, left)),
                    op => unreachable!("Split {cmp:?} on non-lt non-gt operator {op:?}"),
                }
            })
            .try_fold(vec![], |mut acc, cmp| {
                cmp.map(|(less, greater)| {
                    let num_alloys = less.max(greater) as usize;
                    while acc.len() <= num_alloys {
                        acc.push(vec![]);
                    }
                    acc[less as usize].push(greater);
                    acc
                })
            })?
            .into_iter()
            .enumerate()
            .skip(1)
            .map(|(alloy_id, harder_alloys)| (alloy_id as u64, harder_alloys))
            .collect::<Vec<_>>();
        Ok(Self { inner: orderings })
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<u64> {
    input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                let mut orderings = line.parse::<Orderings>()?;
                #[cfg(debug_assertions)]
                eprintln!("{orderings}");
                let res = orderings.checksum();
                #[cfg(debug_assertions)]
                eprintln!("{res}");
                Ok(res)
            })
        })
        .sum()
}

fn part2(input: &mut dyn BufRead) -> io::Result<u64> {
    input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                let mut orderings = line.parse::<Orderings>()?;
                #[cfg(debug_assertions)]
                eprintln!("{orderings}");
                if !orderings.infer_missing_comparison() {
                    eprintln!("Couldn't infer missing comparison for {orderings:?}");
                }
                let res = orderings.checksum();
                #[cfg(debug_assertions)]
                eprintln!("{res}");
                Ok(res)
            })
        })
        .sum()
}

fn part3(input: &mut dyn BufRead) -> io::Result<u64> {
    input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                let mut orderings = line.parse::<Orderings>()?;
                #[cfg(debug_assertions)]
                eprintln!("{orderings}");
                let alloys = orderings
                    .inner
                    .iter()
                    .map(|&(id, _)| id)
                    .collect::<Vec<_>>();
                let alloys2 = alloys.clone();
                let alloys3 = alloys.clone();
                let mut remaining_comparisons = alloys
                    .iter()
                    .flat_map(move |&alloy1| {
                        alloys2
                            .clone()
                            .into_iter()
                            .filter(move |&alloy2| alloy1 < alloy2)
                            .flat_map({
                                let alloys3 = alloys3.clone();
                                move |alloy2| {
                                    alloys3
                                        .clone()
                                        .into_iter()
                                        .map(move |alloy3| (alloy1, alloy2, alloy3))
                                }
                            })
                    })
                    .filter(|&(a1, a2, _)| a1 < a2)
                    .collect::<Vec<_>>();
                while !remaining_comparisons.is_empty() {
                    for (alloy1, alloy2, alloy3) in
                        mem::take(&mut remaining_comparisons).into_iter()
                    {
                        if orderings.get(alloy1, alloy2).is_some() {
                            continue;
                        }
                        match (orderings.get(alloy1, alloy3), orderings.get(alloy3, alloy2)) {
                            (None, None) | (_, None) | (None, _) => {
                                remaining_comparisons.push((alloy1, alloy2, alloy3))
                            }
                            (Some(Ordering::Less), Some(Ordering::Less)) => {
                                orderings.insert(alloy1, alloy2);
                            }
                            (Some(Ordering::Greater), Some(Ordering::Greater)) => {
                                orderings.insert(alloy2, alloy1);
                            }
                            (Some(_), Some(_)) => {
                                // alloy3 is either harder than both alloy1 and alloy2 or softer
                                // than both. Either way, it can't tell us about the relationship
                                // between alloy1 and alloy2.
                            }
                        }
                    }
                }
                let res = orderings.checksum();
                #[cfg(debug_assertions)]
                eprintln!("{res}");
                Ok(res)
            })
        })
        .sum()
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Lord of the wings Quest 2 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "lord-of-the-wings_02-1.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 2 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "lord-of-the-wings_02-2.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 2 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "lord-of-the-wings_02-3.txt"
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
            "4<3 2<5 3>1 1>2 1<5 4<1 3>2 4<2 3>5 4<5\n",
            "4>1 2>1 3>2 1<5 4>5 2<5 3>1 4>3 2<4 5<3\n",
            "5>1 5>2 4>1 2<4 3>2 3<4 1>2 3>5 1<3 4>5\n",
        );
        let expected = 122_157;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "2<5 3>1 1>2 1<5 4<1 3>2 4<2 3>5 4<5\n",
            "2>1 3>2 1<5 4>5 2<5 3>1 4>3 2<4 5<3\n",
            "5>1 5>2 4>1 3>2 3<4 1>2 3>5 1<3 4>5\n",
        );
        let expected = 122_157;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "3>1 1>2 1<5 4<2 3>5\n",
            "2>1 2<5 3>1 4>3 2<4 5<3\n",
            "5>1 3<4 1>2 3>5\n",
        );
        let expected = 122_157;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
