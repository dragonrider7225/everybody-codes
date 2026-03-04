use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

fn part1(input: &mut dyn BufRead) -> io::Result<u32> {
    todo!("Melody Made of Code Quest 1 Part 1")
}

fn part2(input: &mut dyn BufRead) -> io::Result<u32> {
    todo!("Melody Made of Code Quest 1 Part 2")
}

fn part3(input: &mut dyn BufRead) -> io::Result<u32> {
    todo!("Melody Made of Code Quest 1 Part 3")
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Melody Made of Code Quest 1 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "melody-made-of-code_01-1.txt"
            )?))?
        );
    }
    {
        println!("Melody Made of Code Quest 1 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "melody-made-of-code_01-2.txt"
            )?))?
        );
    }
    {
        println!("Melody Made of Code Quest 1 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "melody-made-of-code_01-3.txt"
            )?))?
        );
    }
    Ok(())
}
