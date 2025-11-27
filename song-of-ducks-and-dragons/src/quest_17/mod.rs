use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader},
    ops::BitOr,
};

use dependencies::priority_queue::PriorityQueue;

fn read_grid(input: &mut dyn BufRead) -> io::Result<Vec<Vec<Tile>>> {
    input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.bytes()
                    .map(|b| match b {
                        b'@' => Ok(Tile::Volcano),
                        b'S' => Ok(Tile::Start),
                        b'0'..=b'9' => Ok(Tile::Num((b - b'0') as _)),
                        _ => Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("{:?} is not a valid tile character", b as char),
                        )),
                    })
                    .collect::<io::Result<Vec<_>>>()
            })
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Tile {
    Volcano,
    Num(usize),
    Start,
}

impl Tile {
    pub fn value(&self) -> usize {
        match self {
            &Self::Num(n) => n,
            Self::Volcano | Self::Start => 0,
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Start => write!(f, "S"),
            Tile::Volcano => write!(f, "@"),
            Tile::Num(n) => write!(f, "{n}"),
        }
    }
}

fn is_burnt(position: (usize, usize), volcano: (usize, usize), volcano_radius: usize) -> bool {
    let d_squared = position.0.abs_diff(volcano.0).pow(2) + position.1.abs_diff(volcano.1).pow(2);
    d_squared <= volcano_radius.pow(2)
}

fn volcano_effect(grid: &[Vec<Tile>], radius: usize) -> usize {
    let volcano = grid
        .iter()
        .enumerate()
        .flat_map(|(row_idx, row)| {
            row.iter()
                .copied()
                .enumerate()
                .filter_map(move |(col_idx, tile)| {
                    Some((col_idx, row_idx)).filter(|_| tile == Tile::Volcano)
                })
        })
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Notes do not contain volcano"))
        .unwrap();
    grid.iter()
        .enumerate()
        .flat_map(|(row_idx, row)| {
            row.iter().enumerate().filter_map(move |(col_idx, tile)| {
                Some(tile)
                    .and_then(|tile| match tile {
                        Tile::Num(n) => Some(n),
                        Tile::Volcano | Tile::Start => None,
                    })
                    .filter(|_| {
                        is_burnt((col_idx, row_idx), volcano, radius)
                            && !is_burnt((col_idx, row_idx), volcano, radius - 1)
                    })
            })
        })
        .sum()
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    const R: usize = 10;

    let grid = read_grid(input)?;
    let volcano = grid
        .iter()
        .enumerate()
        .flat_map(|(row_idx, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, tile)| matches!(tile, Tile::Volcano))
                .map(move |(col_idx, _)| (col_idx, row_idx))
        })
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "No volcano in notes"))?;
    Ok(grid
        .into_iter()
        .enumerate()
        .flat_map(|(row_idx, row)| {
            row.into_iter()
                .enumerate()
                .filter(move |&(col_idx, _)| is_burnt((col_idx, row_idx), volcano, R))
                .map(move |(_, tile)| tile.value())
        })
        .sum())
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let grid = read_grid(input)?;
    let (r, total) = (1..)
        .map(|radius| (radius, volcano_effect(&grid, radius)))
        .take_while(|&(_, total)| total > 0)
        .fold((0, 0), |(acc_r, acc_total), (r, total)| {
            if total > acc_total {
                (r, total)
            } else {
                (acc_r, acc_total)
            }
        });
    Ok(r * total)
}

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
struct LoopStatus {
    left: bool,
    bottom: bool,
    right: bool,
}

impl LoopStatus {
    pub fn done(&self) -> bool {
        self.left && self.bottom && self.right
    }
}

impl BitOr for LoopStatus {
    type Output = Option<Self>;

    fn bitor(self, rhs: Self) -> Self::Output {
        Some(Self {
            left: self.left || rhs.left,
            bottom: self.bottom || rhs.bottom,
            right: self.right || rhs.right,
        })
        .filter(|ret| !(ret.left && ret.right && !ret.bottom))
    }
}

impl Debug for LoopStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}{}{}]",
            if self.left { "L" } else { " " },
            if self.bottom { "B" } else { " " },
            if self.right { "R" } else { " " }
        )
    }
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let grid = read_grid(input)?;
    let volcano = grid
        .iter()
        .enumerate()
        .flat_map(|(row_idx, row)| {
            row.iter()
                .copied()
                .enumerate()
                .filter_map(move |(col_idx, tile)| {
                    Some((col_idx, row_idx)).filter(|_| matches!(tile, Tile::Volcano))
                })
        })
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "No volcano in notes"))?;
    let start = grid
        .iter()
        .enumerate()
        .flat_map(|(row_idx, row)| {
            row.iter()
                .copied()
                .enumerate()
                .filter_map(move |(col_idx, tile)| {
                    Some((col_idx, row_idx)).filter(|_| matches!(tile, Tile::Start))
                })
        })
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "No volcano in notes"))?;
    let neighbors = |position: (usize, usize), volcano_radius: usize| {
        let grid = &grid[..];
        [(0, -1), (0, 1), (-1, 0), (1, 0)]
            .into_iter()
            .filter_map(move |delta| {
                Some((
                    position.0.checked_add_signed(delta.0)?,
                    position.1.checked_add_signed(delta.1)?,
                ))
            })
            .filter(move |neighbor| neighbor.1 < grid.len() && neighbor.0 < grid[0].len())
            .filter(move |&neighbor| !is_burnt(neighbor, volcano, volcano_radius))
            .map(move |neighbor| {
                let on_volcano_row = neighbor.1 == volcano.1;
                (
                    neighbor,
                    LoopStatus {
                        left: on_volcano_row && neighbor.0 < volcano.0,
                        bottom: neighbor.0 == volcano.0 && neighbor.1 > volcano.1,
                        right: on_volcano_row && neighbor.0 > volcano.0,
                    },
                )
            })
    };
    (0..volcano.0)
        .filter_map(|volcano_radius| {
            let next_burn = (volcano_radius + 1) * 30;
            let mut visited = HashSet::new();
            let mut sources = HashMap::new();
            let mut frontier = PriorityQueue::new();
            frontier.push((start, LoopStatus::default()), Reverse(0));
            sources.insert((start, LoopStatus::default()), None);
            while let Some(((next, loop_status), Reverse(time))) = frontier.pop() {
                if next == start && loop_status.done() {
                    #[cfg(test)]
                    {
                        grid.iter().enumerate().for_each(|(row_idx, row)| {
                            row.iter().enumerate().for_each(|(col_idx, &tile)| {
                                if is_burnt((col_idx, row_idx), volcano, volcano_radius) {
                                    eprint!(".");
                                } else {
                                    eprint!("{}", tile);
                                }
                            });
                            eprintln!();
                        });
                        let mut path = vec![(time, next)];
                        let mut ls = loop_status;
                        loop {
                            let Some(&(next_time, next)) = path.last() else {
                                unreachable!()
                            };
                            let Some(&Some((last, last_ls))) = sources.get(&(next, ls)) else {
                                break;
                            };
                            ls = last_ls;
                            path.push((next_time - grid[next.1][next.0].value(), last));
                        }
                        eprintln!("Path: [");
                        while let Some((time, pos)) = path.pop() {
                            eprintln!("    {time}: ({}, {}),", pos.0, pos.1);
                        }
                        eprintln!("]");
                    }
                    return Some(time * volcano_radius);
                }
                visited.insert((next, loop_status));
                for (neighbor, neighbor_loop) in neighbors(next, volcano_radius) {
                    let Some(neighbor_loop) = neighbor_loop | loop_status else {
                        continue;
                    };
                    if visited.contains(&(neighbor, neighbor_loop)) {
                        continue;
                    }
                    let neighbor_time = time + grid[neighbor.1][neighbor.0].value();
                    if neighbor_time >= next_burn {
                        continue;
                    }
                    match frontier.push_increase((neighbor, neighbor_loop), Reverse(neighbor_time))
                    {
                        Some(Reverse(old_time)) if neighbor_time == old_time => {}
                        _ => {
                            sources.insert((neighbor, neighbor_loop), Some((next, loop_status)));
                        }
                    }
                }
            }
            None
        })
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "No loop possible"))
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Song of Ducks and Dragons Quest 17 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_17-1.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 17 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_17-2.txt"
            )?))?
        );
    }
    {
        println!("Song of Ducks and Dragons Quest 17 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "song-of-ducks-and-dragons_17-3.txt"
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
            "189482189843433862719\n",
            "279415473483436249988\n",
            "432746714658787816631\n",
            "428219317375373724944\n",
            "938163982835287292238\n",
            "627369424372196193484\n",
            "539825864246487765271\n",
            "517475755641128575965\n",
            "685934212385479112825\n",
            "815992793826881115341\n",
            "1737798467@7983146242\n",
            "867597735651751839244\n",
            "868364647534879928345\n",
            "519348954366296559425\n",
            "134425275832833829382\n",
            "764324337429656245499\n",
            "654662236199275446914\n",
            "317179356373398118618\n",
            "542673939694417586329\n",
            "987342622289291613318\n",
            "971977649141188759131\n",
        );
        let expected = 1573;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "4547488458944\n",
            "9786999467759\n",
            "6969499575989\n",
            "7775645848998\n",
            "6659696497857\n",
            "5569777444746\n",
            "968586@767979\n",
            "6476956899989\n",
            "5659745697598\n",
            "6874989897744\n",
            "6479994574886\n",
            "6694118785585\n",
            "9568991647449\n",
        );
        let expected = 1090;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA_1: &str = concat!(
            "2645233S5466644\n",
            "634566343252465\n",
            "353336645243246\n",
            "233343552544555\n",
            "225243326235365\n",
            "536334634462246\n",
            "666344656233244\n",
            "6426432@2366453\n",
            "364346442652235\n",
            "253652463426433\n",
            "426666225623563\n",
            "555462553462364\n",
            "346225464436334\n",
            "643362324542432\n",
            "463332353552464\n",
        );
        let expected = 592;
        let actual = part3(&mut Cursor::new(TEST_DATA_1))?;
        assert_eq!(expected, actual);
        const TEST_DATA_2: &str = concat!(
            "545233443422255434324\n",
            "5222533434S2322342222\n",
            "523444354223232542432\n",
            "553522225435232255242\n",
            "232343243532432452524\n",
            "245245322252324442542\n",
            "252533232225244224355\n",
            "523533554454232553332\n",
            "522332223232242523223\n",
            "524523432425432244432\n",
            "3532242243@4323422334\n",
            "542524223994422443222\n",
            "252343244322522222332\n",
            "253355425454255523242\n",
            "344324325233443552555\n",
            "423523225325255345522\n",
            "244333345244325322335\n",
            "242244352245522323422\n",
            "443332352222535334325\n",
            "323532222353523253542\n",
            "553545434425235223552\n",
        );
        let expected = 330;
        let actual = part3(&mut Cursor::new(TEST_DATA_2))?;
        assert_eq!(expected, actual);
        const TEST_DATA_3: &str = concat!(
            "5441525241225111112253553251553\n",
            "133522122534119S911411222155114\n",
            "3445445533355599933443455544333\n",
            "3345333555434334535435433335533\n",
            "5353333345335554434535533555354\n",
            "3533533435355443543433453355553\n",
            "3553353435335554334453355435433\n",
            "5435355533533355533535335345335\n",
            "4353545353545354555534334453353\n",
            "4454543553533544443353355553453\n",
            "5334554534533355333355543533454\n",
            "4433333345445354553533554555533\n",
            "5554454343455334355445533453453\n",
            "4435554534445553335434455334353\n",
            "3533435453433535345355533545555\n",
            "534433533533535@353533355553345\n",
            "4453545555435334544453344455554\n",
            "4353333535535354535353353535355\n",
            "4345444453554554535355345343354\n",
            "3534544535533355333333445433555\n",
            "3535333335335334333534553543535\n",
            "5433355333553344355555344553435\n",
            "5355535355535334555435534555344\n",
            "3355433335553553535334544544333\n",
            "3554333535553335343555345553535\n",
            "3554433545353554334554345343343\n",
            "5533353435533535333355343333555\n",
            "5355555353355553535354333535355\n",
            "4344534353535455333455353335333\n",
            "5444333535533453535335454535553\n",
            "3534343355355355553543545553345\n",
        );
        let expected = 3180;
        let actual = part3(&mut Cursor::new(TEST_DATA_3))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
