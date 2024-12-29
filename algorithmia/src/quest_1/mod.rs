use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

enum Enemy {
    A,
    B,
    C,
    D,
}

impl Enemy {
    fn potion_cost(&self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 3,
            Self::D => 5,
        }
    }
}

impl TryFrom<u8> for Enemy {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'A' => Ok(Self::A),
            b'B' => Ok(Self::B),
            b'C' => Ok(Self::C),
            b'D' => Ok(Self::D),
            _ => Err(()),
        }
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let input = io::read_to_string(input)?;
    Ok(input
        .trim()
        .bytes()
        .map(|b| {
            Enemy::try_from(b)
                .expect("Invalid enemy: {b:?}")
                .potion_cost()
        })
        .sum())
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let input = io::read_to_string(input)?;
    let input = input.trim().as_bytes();
    Ok(input
        .chunks(2)
        .map(|chunk| [Enemy::try_from(chunk[0]), Enemy::try_from(chunk[1])])
        .map(|pair| match pair {
            [Ok(enemy), Err(_)] | [Err(_), Ok(enemy)] => enemy.potion_cost(),
            [Ok(enemy1), Ok(enemy2)] => enemy1.potion_cost() + enemy2.potion_cost() + 2,
            [Err(_), Err(_)] => 0,
        })
        .sum())
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let input = io::read_to_string(input)?;
    let input = input.trim().as_bytes();
    Ok(input
        .chunks(3)
        .map(|chunk| {
            chunk
                .iter()
                .filter_map(|&e| Enemy::try_from(e).ok())
                .fold((0, 0), |(count, cost), enemy| {
                    (count + 1, cost + enemy.potion_cost())
                })
        })
        .filter(|&(count, _)| count > 0)
        .map(|(count, cost)| count * (count - 1) + cost)
        .sum())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Algorithmia Quest 1 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open("algorithmia_01-1.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 1 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open("algorithmia_01-2.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 1 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open("algorithmia_01-3.txt")?))?
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use io::Cursor;

    use super::*;

    #[test]
    fn test_part1() -> io::Result<()> {
        let expected = 5;
        let actual = part1(&mut Cursor::new("ABBAC\n"))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        let expected = 28;
        let actual = part2(&mut Cursor::new("AxBCDDCAxD\n"))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        let expected = 30;
        let actual = part3(&mut Cursor::new("xBxAAABCDxCC\n"))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
