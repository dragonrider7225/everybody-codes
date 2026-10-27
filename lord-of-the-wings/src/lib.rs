use std::io;

mod quest_1;
mod quest_2;
mod quest_3;
mod quest_4;
mod quest_5;
mod quest_6;
mod quest_7;
mod quest_8;
mod quest_9;

mod quest_10;
mod quest_11;
mod quest_12;
mod quest_13;
mod quest_14;
mod quest_15;
mod quest_16;
mod quest_17;
mod quest_18;
mod quest_19;
mod quest_20;

pub fn run_quest(quest: u32) -> io::Result<()> {
    match quest {
        1 => quest_1::run(),
        2 => quest_2::run(),
        3 => quest_3::run(),
        4 => quest_4::run(),
        5 => quest_5::run(),
        6 => quest_6::run(),
        7 => quest_7::run(),
        8 => quest_8::run(),
        9 => quest_9::run(),
        10 => quest_10::run(),
        11 => quest_11::run(),
        12 => quest_12::run(),
        13 => quest_13::run(),
        14 => quest_14::run(),
        15 => quest_15::run(),
        16 => quest_16::run(),
        17 => quest_17::run(),
        18 => quest_18::run(),
        19 => quest_19::run(),
        20 => quest_20::run(),
        quest => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid quest: {quest}"),
        )),
    }
}
