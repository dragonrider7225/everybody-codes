use std::{
    collections::HashSet,
    fmt::{self, Debug, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    iter,
    ops::Index,
    str::FromStr,
};

#[derive(Clone, Debug)]
pub struct Die {
    faces: Vec<i32>,
    seed: usize,
    pulse: usize,
    roll_number: usize,
    last_roll: usize,
}

impl Die {
    pub fn new(faces: Vec<i32>, seed: usize) -> Self {
        Self {
            faces,
            seed,
            pulse: seed,
            roll_number: 0,
            last_roll: 0,
        }
    }
}

impl FromStr for Die {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid_input = |ctx| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "{ctx}: Die definition must be of the form {:?}",
                    "<id: usize>: faces=<faces: Vec<i32>> seed=<seed: usize>"
                ),
            )
        };
        let (_id, definition) = s
            .split_once(": faces=[")
            .ok_or_else(|| invalid_input("faces split"))?;
        let (faces, seed) = definition
            .split_once("] seed=")
            .ok_or_else(|| invalid_input("seed split"))?;
        let faces = faces
            .split(",")
            .map(|n| n.parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let seed = seed
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        Ok(Self::new(faces, seed))
    }
}

impl Iterator for Die {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        self.roll_number += 1;
        let spin = self.roll_number * self.pulse;
        self.last_roll = (self.last_roll + spin) % self.faces.len();
        self.pulse = ((self.pulse + spin) % self.seed) + 1 + self.roll_number + self.seed;
        Some(self.faces[self.last_roll])
    }
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut dice = input
        .lines()
        .map(|line| line.and_then(|line| line.parse::<Die>()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(iter::successors(Some(0), |&total| {
        Some(total + dice.iter_mut().flat_map(Iterator::next).sum::<i32>())
    })
    .enumerate()
    .find(|&(_, total)| total >= 10_000)
    .expect("Total never reaches 10000")
    .0)
}

fn part2(input: &mut dyn BufRead) -> io::Result<Vec<usize>> {
    let mut lines = input.lines();
    let mut dice = lines
        .by_ref()
        .take_while(|line_r| line_r.as_ref().is_ok_and(|line| !line.is_empty()))
        .map(|line| line.and_then(|line| line.parse::<Die>()))
        .collect::<Result<Vec<_>, _>>()?;
    let spaces = lines
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing spaces"))
        .flatten()?
        .bytes()
        .map(|b| b - b'0')
        .collect::<Vec<_>>();
    let mut finished = vec![];
    let mut positions = vec![0; dice.len()];
    let mut still_going = (0..dice.len()).collect::<Vec<_>>();
    let mut next_going = vec![];
    for _i in 1.. {
        for die_id in still_going {
            if dice[die_id].next().unwrap() == spaces[positions[die_id]].into() {
                positions[die_id] += 1;
                if positions[die_id] >= spaces.len() {
                    finished.push(die_id + 1);
                    continue;
                }
            }
            next_going.push(die_id);
        }
        if next_going.is_empty() {
            break;
        }
        still_going = vec![];
        std::mem::swap(&mut still_going, &mut next_going);
    }
    Ok(finished)
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Position {
    row: usize,
    column: usize,
}

impl Position {
    /// Find all neighbors of this position in a grid that has exactly `height` rows and `width`
    /// columns.
    fn neighbors(&self, width: usize, height: usize) -> impl Iterator<Item = Self> {
        let mut ret = vec![*self];
        if self.row > 0 {
            ret.push(Self {
                row: self.row - 1,
                ..*self
            });
        }
        if self.column > 0 {
            ret.push(Self {
                column: self.column - 1,
                ..*self
            });
        }
        if self.row + 1 < height {
            ret.push(Self {
                row: self.row + 1,
                ..*self
            });
        }
        if self.column + 1 < width {
            ret.push(Self {
                column: self.column + 1,
                ..*self
            });
        }
        ret.into_iter()
    }

    /// Get all valid positions in a grid that has exactly `height` rows and `width` columns.
    fn grid_coordinates(width: usize, height: usize) -> impl Iterator<Item = Self> {
        (0..width).flat_map(move |column| (0..height).map(move |row| Self { row, column }))
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.column, self.row)
    }
}

impl<T> Index<Position> for Vec<Vec<T>> {
    type Output = T;

    fn index(&self, index: Position) -> &Self::Output {
        &self[index.row][index.column]
    }
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let mut lines = input.lines();
    let dice = lines
        .by_ref()
        .take_while(|line_r| line_r.as_ref().is_ok_and(|line| !line.is_empty()))
        .map(|line| line.and_then(|line| line.parse::<Die>()))
        .collect::<Result<Vec<_>, _>>()?;
    let spaces = lines
        .map(|line| Ok(line?.bytes().map(|b| (b - b'0') as i32).collect::<Vec<_>>()))
        .collect::<io::Result<Vec<_>>>()?;
    let width = spaces[0].len();
    let height = spaces.len();
    let mut collected = HashSet::<Position>::new();
    for mut die in dice {
        let mut tokens = Position::grid_coordinates(width, height).collect::<HashSet<_>>();
        while !tokens.is_empty() {
            let roll = die.next().unwrap();
            let survivors = tokens
                .into_iter()
                .filter(|&token| roll == spaces[token])
                .collect::<HashSet<_>>();
            collected.extend(survivors.iter().copied());
            tokens = survivors
                .into_iter()
                .flat_map(|token| token.neighbors(width, height).collect::<Vec<_>>())
                .collect();
        }
    }
    Ok(collected.len())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("The Entertainment Hub Quest 3 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "the-entertainment-hub_03-1.txt"
            )?))?
        );
    }
    {
        println!("The Entertainment Hub Quest 3 Part 2");
        let ids = part2(&mut BufReader::new(File::open(
            "the-entertainment-hub_03-2.txt",
        )?))?;
        print!("{}", ids[0]);
        for id in &ids[1..] {
            print!(",{id}");
        }
        println!();
    }
    {
        println!("The Entertainment Hub Quest 3 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "the-entertainment-hub_03-3.txt"
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
    fn test_die() {
        let mut die = Die::new(vec![1, 2, 4, -1, 5, 7, 9], 3);
        let expected = vec![-1, 9, -1, -1, 5, 4, 4, 2, 5, 2, 4, 5, 9, 2, 2, 4, 2, 4, 7];
        let mut actual = die.by_ref().take(10).collect::<Vec<_>>();
        for _ in 2..=10 {
            actual.push(die.nth(9).unwrap());
        }
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_part1() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "1: faces=[1,2,3,4,5,6] seed=7\n",
            "2: faces=[-1,1,-1,1,-1] seed=13\n",
            "3: faces=[9,8,7,8,9] seed=17\n",
        );
        let expected = 844;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "1: faces=[1,2,3,4,5,6,7,8,9] seed=13\n",
            "2: faces=[1,2,3,4,5,6,7,8,9] seed=29\n",
            "3: faces=[1,2,3,4,5,6,7,8,9] seed=37\n",
            "4: faces=[1,2,3,4,5,6,7,8,9] seed=43\n",
            "\n",
            "51257284\n",
        );
        let expected = vec![1, 3, 4, 2];
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_small() -> io::Result<()> {
        const TEST_DATA_SMALL: &str = concat!(
            "1: faces=[1,2,3,4,5,6,7,8,9] seed=13\n",
            "\n",
            "1523758297\n",
            "4822941583\n",
            "7627997892\n",
            "4397697132\n",
            "1799773472\n",
        );
        let expected = 33;
        let actual = part3(&mut Cursor::new(TEST_DATA_SMALL))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3_large() -> io::Result<()> {
        const TEST_DATA_LARGE: &str = concat!(
            "1: faces=[1,2,3,4,5,6,7,8,9] seed=339211\n",
            "2: faces=[1,2,3,4,5,6,7,8,9] seed=339517\n",
            "3: faces=[1,2,3,4,5,6,7,8,9] seed=339769\n",
            "4: faces=[1,2,3,4,5,6,7,8,9] seed=339049\n",
            "5: faces=[1,2,3,4,5,6,7,8,9] seed=338959\n",
            "6: faces=[1,2,3,4,5,6,7,8,9] seed=340111\n",
            "7: faces=[1,2,3,4,5,6,7,8,9] seed=339679\n",
            "8: faces=[1,2,3,4,5,6,7,8,9] seed=339121\n",
            "9: faces=[1,2,3,4,5,6,7,8,9] seed=338851\n",
            "\n",
            "94129478611916584144567479397512595367821487689499329543245932151\n",
            "45326719759656232865938673559697851227323497148536117267854241288\n",
            "44425936468288462848395149959678842215853561564389485413422813386\n",
            "64558359733811767982282485122488769592428259771817485135798694145\n",
            "17145764554656647599363636643624443394141749674594439266267914738\n",
            "89687344812176758317288229174788352467288242171125512646356965953\n",
            "72436836424726621961424876248346712363842529736689287535527512173\n",
            "18295771348356417112646514812963612341591986162693455745689374361\n",
            "56445661964557624561727322332461348422854112571195242864151143533\n",
            "77537797151985578367895335725777225518396231453691496787716283477\n",
            "37666899356978497489345173784484282858559847597424967325966961183\n",
            "26423131974661694562195955939964966722352323745667498767153191712\n",
            "99821139398463125478734415536932821142852955688669975837535594682\n",
            "17768265895455681847771319336534851247125295119363323122744953158\n",
            "25655579913247189643736314385964221584784477663153155222414634387\n",
            "62881693835262899543396571369125158422922821541597516885389448546\n",
            "71751114798332662666694134456689735288947441583123159231519473489\n",
            "94932859392146885633942828174712588132581248183339538341386944937\n",
            "53828883514868969493559487848248847169557825166338328352792866332\n",
            "54329673374115668178556175692459528276819221245996289611868492731\n",
            "97799599164121988455613343238811122469229423272696867686953891233\n",
            "56249752581283778997317243845187615584225693829653495119532543712\n",
            "39171354221177772498317826968247939792845866251456175433557619425\n",
            "56425749216121421458547849142439211299266255482219915528173596421\n",
            "48679971256541851497913572722857258171788611888347747362797259539\n",
            "32676924489943265499379145361515824954991343541956993467914114579\n",
            "45733396847369746189956225365375253819969643711633873473662833395\n",
            "42291594527499443926636288241672629499242134451937866578992236427\n",
            "47615394883193571183931424851238451485822477158595936634849167455\n",
            "16742896921499963113544858716552428241241973653655714294517865841\n",
            "57496921774277833341488566199458567884285639693339942468585269698\n",
            "22734249697451127789698862596688824444191118289959746248348491792\n",
            "28575193613471799766369217455617858422158428235521423695479745656\n",
            "74234343226976999161289522983885254212712515669681365845434541257\n",
            "43457237419516813368452247532764649744546181229533942414983335895\n",
        );
        let expected = 1125;
        let actual = part3(&mut Cursor::new(TEST_DATA_LARGE))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
