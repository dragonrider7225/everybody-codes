use std::io;

mod quest_1;
mod quest_2;
mod quest_3;

pub fn run_quest(quest: u32) -> io::Result<()> {
    match quest {
        1 => quest_1::run(),
        2 => quest_2::run(),
        3 => quest_3::run(),
        quest => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid quest: {quest}"),
        )),
    }
}
