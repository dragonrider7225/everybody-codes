use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead, BufReader},
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct StringCache {
    cache: HashSet<&'static str>,
}

impl StringCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, s: &str) -> &'static str {
        match self.cache.iter().copied().find(|&_s| _s == s) {
            Some(_s) => _s,
            None => self.insert_static(Box::leak(Box::from(s))),
        }
    }

    pub fn insert_static(&mut self, s: &'static str) -> &'static str {
        match self.cache.iter().copied().find(|&_s| _s == s) {
            Some(_s) => _s,
            None => {
                self.cache.insert(s);
                s
            }
        }
    }
}

struct Tree {
    edges: HashMap<&'static str, Vec<&'static str>>,
    rev_edges: HashMap<&'static str, &'static str>,
}

impl Tree {
    pub fn path_to_unique_fruit(&self) -> Option<Vec<&'static str>> {
        fn is_fruit(&(_, child): &(&str, &str)) -> bool {
            child == "@"
        }

        let mut frontier = vec![("", "RR")];
        while !frontier.is_empty() {
            for (_, node) in std::mem::take(&mut frontier) {
                frontier.extend(
                    self.edges
                        .get(node)
                        .iter()
                        .flat_map(|&x| x)
                        .map(|&child| (node, child)),
                );
            }
            let Some(first_fruit) = frontier.iter().copied().find(is_fruit) else {
                continue;
            };
            if Some(first_fruit) == frontier.iter().copied().rfind(is_fruit) {
                println!("Found unique fruit under {}", first_fruit.0);
                let mut ret = vec![first_fruit.1, first_fruit.0];
                while ret.last() != Some(&"RR") {
                    let parent = self.rev_edges[ret.last().unwrap()];
                    ret.push(parent);
                }
                ret.reverse();
                return Some(ret);
            }
            frontier.retain_mut(|x| !is_fruit(&*x));
        }
        None
    }

    pub fn from_read(input: &mut dyn BufRead, cache: &mut StringCache) -> io::Result<Self> {
        Ok(Tree::from_iter(
            input
                .lines()
                .map(|line| {
                    let line = line?;
                    let (parent, children) = line.split_once(':').ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("{line:?} is not a parent-children relationship"),
                        )
                    })?;
                    let parent = cache.insert(parent);
                    let children = children
                        .split(',')
                        .map(|s| cache.insert(s))
                        .collect::<Vec<_>>();
                    Ok((parent, children))
                })
                .collect::<io::Result<Vec<_>>>()?,
        ))
    }
}

impl FromIterator<(&'static str, Vec<&'static str>)> for Tree {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (&'static str, Vec<&'static str>)>,
    {
        let edges: HashMap<&'static str, Vec<&'static str>> = iter.into_iter().collect();
        let rev_edges = edges
            .iter()
            .flat_map(|(&parent, children)| {
                children.iter().copied().map(move |child| (child, parent))
            })
            .collect::<HashMap<_, _>>();
        Self { edges, rev_edges }
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<String> {
    let mut cache = StringCache::new();
    let tree = Tree::from_read(input, &mut cache)?;
    tree.path_to_unique_fruit().map_or_else(
        || {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Tree does not contain any fruits at a unique height",
            ))
        },
        |path| Ok(path.into_iter().collect()),
    )
}

fn part2(input: &mut dyn BufRead) -> io::Result<String> {
    let mut cache = StringCache::new();
    let tree = Tree::from_read(input, &mut cache)?;
    tree.path_to_unique_fruit().map_or_else(
        || {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Tree does not contain any fruits at a unique height",
            ))
        },
        |path| {
            Ok(path
                .into_iter()
                .map(|s| s.chars().next().unwrap())
                .collect())
        },
    )
}

fn part3(input: &mut dyn BufRead) -> io::Result<String> {
    part2(input)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Algorithmia Quest 6 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open("algorithmia_06-1.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 6 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open("algorithmia_06-2.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 6 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open("algorithmia_06-3.txt")?))?
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
            "RR:A,B,C\n",
            "A:D,E\n",
            "B:F,@\n",
            "C:G,H\n",
            "D:@\n",
            "E:@\n",
            "F:@\n",
            "G:@\n",
            "H:@\n",
        );
        let expected = "RRB@";
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "RR:ABAB\n",
            "ABAB:CDCD\n",
            "CDCD:EFEF\n",
            "EFEF:ROLO\n",
            "ROLO:@\n",
        );
        let expected = "RACER@";
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
