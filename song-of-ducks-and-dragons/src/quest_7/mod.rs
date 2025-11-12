use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
};

fn read_names(input: &mut dyn BufRead) -> io::Result<Vec<String>> {
    input
        .lines()
        .take_while(|line| !line.as_ref().is_ok_and(String::is_empty))
        .map(|line| line.map(|line| line.split(',').map(str::to_string).collect::<Vec<_>>()))
        .try_fold(vec![], |mut acc, line| {
            acc.extend(line?);
            Ok(acc)
        })
}

type Rules = HashMap<u8, Vec<u8>>;

fn read_rules(input: &mut dyn BufRead) -> io::Result<Rules> {
    input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                let Some((first, seconds)) = line.split_once(" > ") else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "{line:?}: Invalid rule",
                    ));
                };
                Ok((
                    first.chars().next().unwrap() as u8,
                    seconds
                        .split(',')
                        .map(|letter| letter.chars().next().unwrap() as u8)
                        .collect::<Vec<_>>(),
                ))
            })
        })
        .collect()
}

fn part1(input: &mut dyn BufRead) -> io::Result<String> {
    let names = read_names(input)?;
    let rules = read_rules(input)?;
    for name in names {
        let name_bytes = name.as_bytes();
        if name_bytes
            .windows(2)
            .all(|window| rules[&window[0]].contains(&window[1]))
        {
            return Ok(name);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Could not find valid name",
    ))
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let names = read_names(input)?;
    let rules = read_rules(input)?;
    Ok(names
        .into_iter()
        .enumerate()
        .map(|(idx, name)| (idx + 1, name))
        .filter_map(|(idx, name)| {
            let name_bytes = name.as_bytes();
            Some(idx).filter(|_| {
                name_bytes
                    .windows(2)
                    .all(|window| rules[&window[0]].contains(&window[1]))
            })
        })
        .sum())
}

fn part3(input: &mut dyn BufRead) -> io::Result<u32> {
    let prefixes = read_names(input)?;
    let rules = read_rules(input)?;
    let mut valid_prefixes = prefixes
        .into_iter()
        .filter(|name| {
            name.as_bytes()
                .windows(2)
                .all(|window| rules[&window[0]].contains(&window[1]))
        })
        .collect::<Vec<_>>();
    let mut i = 0;
    'outer: while i < valid_prefixes.len() {
        let mut j = i + 1;
        while j < valid_prefixes.len() {
            if valid_prefixes[j].starts_with(&valid_prefixes[i]) {
                valid_prefixes.swap_remove(j);
                continue;
            }
            if valid_prefixes[i].starts_with(&valid_prefixes[j]) {
                valid_prefixes.swap_remove(i);
                continue 'outer;
            }
            j += 1;
        }
        i += 1;
    }
    let mut total_names = 0;
    let mut prefixes = valid_prefixes;
    while !prefixes.is_empty() {
        prefixes = prefixes
            .drain(..)
            .flat_map(|prefix| {
                let prefix_len = prefix.len();
                if (7..=11).contains(&prefix_len) {
                    total_names += 1;
                }
                rules
                    .get(prefix.as_bytes().last().unwrap())
                    .into_iter()
                    .filter(move |_| prefix_len < 11)
                    .flatten()
                    .map(move |&next_letter| {
                        let mut prefix = prefix.clone();
                        prefix.push(next_letter as char);
                        prefix
                    })
            })
            .collect();
    }
    Ok(total_names)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 7 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_07-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 7 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_07-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 7 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_07-3.txt"
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
            "Oronris,Urakris,Oroneth,Uraketh\n",
            "\n",
            "r > a,i,o\n",
            "i > p,w\n",
            "n > e,r\n",
            "o > n,m\n",
            "k > f,r\n",
            "a > k\n",
            "U > r\n",
            "e > t\n",
            "O > r\n",
            "t > h\n",
        );
        let expected = "Oroneth";
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "Xanverax,Khargyth,Nexzeth,Helther,Braerex,Tirgryph,Kharverax\n",
            "\n",
            "r > v,e,a,g,y\n",
            "a > e,v,x,r\n",
            "e > r,x,v,t\n",
            "h > a,e,v\n",
            "g > r,y\n",
            "y > p,t\n",
            "i > v,r\n",
            "K > h\n",
            "v > e\n",
            "B > r\n",
            "t > h\n",
            "N > e\n",
            "p > h\n",
            "H > e\n",
            "l > t\n",
            "z > e\n",
            "X > a\n",
            "n > v\n",
            "x > z\n",
            "T > i\n",
        );
        let expected = 23;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "Xaryt\n",
            "\n",
            "X > a,o\n",
            "a > r,t\n",
            "r > y,e,a\n",
            "h > a,e,v\n",
            "t > h\n",
            "v > e\n",
            "y > p,t\n",
        );
        let expected = 25;
        let actual = part3(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        const TEST_DATA_2: &str = concat!(
            "Khara,Xaryt,Noxer,Kharax\n",
            "\n",
            "r > v,e,a,g,y\n",
            "a > e,v,x,r,g\n",
            "e > r,x,v,t\n",
            "h > a,e,v\n",
            "g > r,y\n",
            "y > p,t\n",
            "i > v,r\n",
            "K > h\n",
            "v > e\n",
            "B > r\n",
            "t > h\n",
            "N > e\n",
            "p > h\n",
            "H > e\n",
            "l > t\n",
            "z > e\n",
            "X > a\n",
            "n > v\n",
            "x > z\n",
            "T > i\n",
        );
        let expected = 1154;
        let actual = part3(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
