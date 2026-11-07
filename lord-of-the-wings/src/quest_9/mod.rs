use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    str::FromStr,
};

struct Variety {
    id: usize,
    growth: usize,
    limit: usize,
}

impl Variety {
    pub fn seconds_to_element_limit(&self) -> usize {
        let mut num_branches = vec![1usize];
        let mut num_points = 0;
        for seconds in 0.. {
            let total_elements = num_branches
                .iter()
                .enumerate()
                .map(|(branch_length, &branch_count)| branch_length * branch_count)
                .sum::<usize>()
                + num_points;
            #[cfg(debug_assertions)]
            eprintln!("{self} has {total_elements} elements after {seconds} seconds");
            if total_elements >= self.limit {
                return seconds;
            }
            num_branches.insert(0, 0);
            if num_branches.len() > self.growth {
                let new_points = num_branches.pop().unwrap();
                num_branches[self.growth / 2] += 2 * new_points;
                num_points += new_points;
                num_branches[1] += new_points;
            }
        }
        panic!("Couldn't reach limit in \"infinite\" time")
    }

    pub fn seconds_to_leaf_limit(&self) -> usize {
        let mut num_branches = HashMap::<usize, usize>::new();
        num_branches.insert(1, 1);
        let mut num_leaves = 1;
        let mut seconds = 1;
        loop {
            #[cfg(debug_assertions)]
            eprintln!("{self} has {num_leaves} leaves after {seconds} seconds");
            if num_leaves >= self.limit && seconds > 0 {
                break seconds;
            }
            let (&longest_branch_length, &longest_branch_count) = num_branches
                .iter()
                .max_by_key(|&(&branch_length, _)| branch_length)
                .unwrap();
            num_branches.remove(&longest_branch_length);
            let steps_compressed = self.growth - longest_branch_length;
            num_branches = num_branches
                .into_iter()
                .map(|(branch_length, branch_count)| {
                    (branch_length + steps_compressed, branch_count)
                })
                .collect();
            *num_branches.entry(self.growth / 2).or_insert(0) += 2 * longest_branch_count;
            *num_branches.entry(1).or_insert(0) += longest_branch_count;
            num_leaves += longest_branch_count;
            seconds += steps_compressed;
        }
    }
}

impl Display for Variety {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "id={}, growth={}, limit={}",
            self.id, self.growth, self.limit
        )
    }
}

impl FromStr for Variety {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s
            .split(", ")
            .map(|part| {
                part.split_once('=')
                    .ok_or_else(|| format!("Missing {:?} in {part:?}", '='))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        let id = parts["id"]
            .parse()
            .map_err(|_| format!("Missing {:?} in {s:?}", "id"))?;
        let growth = parts["growth"]
            .parse()
            .map_err(|_| format!("Missing {:?} in {s:?}", "growth"))?;
        let limit = parts["limit"]
            .parse()
            .map_err(|_| format!("Missing {:?} in {s:?}", "limit"))?;
        Ok(Self { id, growth, limit })
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Variety>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
        })
        .map(|variety| {
            variety.map(|variety| {
                let ret = variety.seconds_to_element_limit();
                #[cfg(debug_assertions)]
                eprintln!("{variety} requires {ret} seconds to reach its limit");
                ret
            })
        })
        .sum()
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Variety>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
        })
        .map(|variety| variety.map(|variety| variety.seconds_to_leaf_limit()))
        .sum()
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    part2(input)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Lord of the wings Quest 9 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "lord-of-the-wings_09-1.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 9 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "lord-of-the-wings_09-2.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 9 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "lord-of-the-wings_09-3.txt"
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
            "id=1, growth=3, limit=10\n",
            "id=2, growth=5, limit=20\n",
            "id=3, growth=7, limit=30\n",
        );
        let expected = 27;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "id=1, growth=3, limit=10\n",
            "id=2, growth=5, limit=20\n",
            "id=3, growth=7, limit=30\n",
        );
        let expected = 42;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
