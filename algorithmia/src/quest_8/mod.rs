use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

fn part1(_input: &mut dyn BufRead) -> io::Result<u32> {
    todo!("Algorithmia Quest 8 Part 1")
}

fn part2(_input: &mut dyn BufRead) -> io::Result<u32> {
    todo!("Algorithmia Quest 8 Part 2")
}

fn part3(_input: &mut dyn BufRead) -> io::Result<u32> {
    todo!("Algorithmia Quest 8 Part 3")
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Algorithmia Quest 8 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open("algorithmia_08-1.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 8 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open("algorithmia_08-2.txt")?))?
        );
    }
    {
        println!("Algorithmia Quest 8 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open("algorithmia_08-3.txt")?))?
        );
    }
    Ok(())
}
