use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

fn part1(_input: &mut dyn BufRead) -> io::Result<u32> {
    todo!("Song of Ducks and Dragons Quest 7 Part 1")
}

fn part2(_input: &mut dyn BufRead) -> io::Result<u32> {
    todo!("Song of Ducks and Dragons Quest 7 Part 2")
}

fn part3(_input: &mut dyn BufRead) -> io::Result<u32> {
    todo!("Song of Ducks and Dragons Quest 7 Part 3")
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
