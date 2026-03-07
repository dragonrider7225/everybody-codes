use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

mod node;
use node::Node;

fn part1(input: &mut dyn BufRead) -> io::Result<u32> {
    let ret = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Node>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
        })
        .reduce(|acc, next| {
            acc.and_then(|mut acc| {
                let res = acc.insert(next?);
                assert!(res.is_ok(), "Nodes do not form a valid tree");
                Ok(acc)
            })
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Empty tree"))??
        .iter_inorder()
        .enumerate()
        .fold(0, |acc, (idx, id)| acc + (idx as u32 + 1) * id);
    Ok(ret)
}

fn part2(input: &mut dyn BufRead) -> io::Result<u32> {
    let ret = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Node>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
        })
        .reduce(|acc, next| {
            acc.and_then(|mut acc| {
                match acc.insert_weak(next?) {
                    Ok(()) => {}
                    Err(next) => {
                        eprintln!("Could not insert {next:?} into tree");
                    }
                }
                Ok(acc)
            })
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Empty tree"))??
        .iter_inorder()
        .enumerate()
        .fold(0, |acc, (idx, id)| acc + (idx as u32 + 1) * id);
    Ok(ret)
}

fn part3(input: &mut dyn BufRead) -> io::Result<u32> {
    let ret = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Node>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
        })
        .reduce(|acc, next| {
            acc.and_then(|mut acc| {
                let mut res = Err(Box::new(next?));
                while let Err(next) = res {
                    res = acc.insert_with_displacement(next);
                }
                Ok(acc)
            })
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Empty tree"))??
        .iter_inorder()
        .enumerate()
        .fold(0, |acc, (idx, id)| acc + (idx as u32 + 1) * id);
    Ok(ret)
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Melody Made of Code Quest 3 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "melody-made-of-code_03-1.txt"
            )?))?
        );
    }
    {
        println!("Melody Made of Code Quest 3 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "melody-made-of-code_03-2.txt"
            )?))?
        );
    }
    {
        println!("Melody Made of Code Quest 3 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "melody-made-of-code_03-3.txt"
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
            "id=1, plug=BLUE HEXAGON, leftSocket=GREEN CIRCLE, rightSocket=BLUE PENTAGON, data=?\n",
            "id=2, plug=GREEN CIRCLE, leftSocket=BLUE HEXAGON, rightSocket=BLUE CIRCLE, data=?\n",
            "id=3, plug=BLUE PENTAGON, leftSocket=BLUE CIRCLE, rightSocket=BLUE CIRCLE, data=?\n",
            "id=4, plug=BLUE CIRCLE, leftSocket=RED HEXAGON, rightSocket=BLUE HEXAGON, data=?\n",
            "id=5, plug=RED HEXAGON, leftSocket=GREEN CIRCLE, rightSocket=RED HEXAGON, data=?\n",
        );
        let expected = 43;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "id=1, plug=RED TRIANGLE, leftSocket=RED TRIANGLE, rightSocket=RED TRIANGLE, data=?\n",
            "id=2, plug=GREEN TRIANGLE, leftSocket=BLUE CIRCLE, rightSocket=GREEN CIRCLE, data=?\n",
            "id=3, plug=BLUE PENTAGON, leftSocket=BLUE CIRCLE, rightSocket=GREEN CIRCLE, data=?\n",
            "id=4, plug=RED TRIANGLE, leftSocket=BLUE PENTAGON, rightSocket=GREEN PENTAGON, data=?\n",
            "id=5, plug=RED PENTAGON, leftSocket=GREEN CIRCLE, rightSocket=GREEN CIRCLE, data=?\n",
        );
        let expected = 50;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_a() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "id=1, plug=RED TRIANGLE, leftSocket=RED TRIANGLE, rightSocket=RED TRIANGLE, data=?\n",
            "id=2, plug=GREEN TRIANGLE, leftSocket=BLUE CIRCLE, rightSocket=GREEN CIRCLE, data=?\n",
            "id=3, plug=BLUE PENTAGON, leftSocket=BLUE CIRCLE, rightSocket=GREEN CIRCLE, data=?\n",
            "id=4, plug=RED TRIANGLE, leftSocket=BLUE PENTAGON, rightSocket=GREEN PENTAGON, data=?\n",
            "id=5, plug=RED PENTAGON, leftSocket=GREEN CIRCLE, rightSocket=GREEN CIRCLE, data=?\n",
        );
        let expected = 38;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_b() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "id=1, plug=RED TRIANGLE, leftSocket=BLUE TRIANGLE, rightSocket=GREEN TRIANGLE, data=?\n",
            "id=2, plug=GREEN TRIANGLE, leftSocket=BLUE CIRCLE, rightSocket=GREEN CIRCLE, data=?\n",
            "id=3, plug=BLUE PENTAGON, leftSocket=BLUE CIRCLE, rightSocket=GREEN CIRCLE, data=?\n",
            "id=4, plug=RED TRIANGLE, leftSocket=BLUE PENTAGON, rightSocket=GREEN PENTAGON, data=?\n",
            "id=5, plug=BLUE TRIANGLE, leftSocket=GREEN CIRCLE, rightSocket=RED CIRCLE, data=?\n",
            "id=6, plug=BLUE TRIANGLE, leftSocket=GREEN CIRCLE, rightSocket=RED CIRCLE, data=?\n",
        );
        let expected = 60;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
