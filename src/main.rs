//! An executable wrapper around my solutions to [Everybody Codes](https://everybody.codes/).

#![warn(
    clippy::create_dir,
    clippy::infinite_loop,
    clippy::let_underscore_must_use,
    clippy::missing_panics_doc,
    clippy::return_self_not_must_use,
    clippy::same_name_method,
    clippy::should_panic_without_expect,
    clippy::use_debug,
    missing_copy_implementations,
    rust_2018_idioms
)]
#![warn(clippy::missing_docs_in_private_items, missing_docs)]
#![deny(
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    missing_debug_implementations,
    unsafe_op_in_unsafe_fn
)]
#![cfg_attr(
    not(debug_assertions),
    deny(clippy::dbg_macro, clippy::todo, clippy::unimplemented)
)]

use clap::{Arg, Command};

use std::io;

/// The CLI for the program.
fn app() -> Command {
    Command::new("Everybody Codes")
        .version("0.1.0")
        .author("Kevin M. <dragonrider7225@gmail.com>")
        .about("Runs one quest of one event of Everybody Codes <https://everybody.codes>")
        .max_term_width(100)
        .arg(
            Arg::new("event")
                .short('e')
                .long("event")
                .value_name("EVENT")
                .value_parser(["algorithmia"])
                .help("Selects the event to run"),
        )
        .arg(
            Arg::new("quest")
                .short('q')
                .long("quest")
                .value_name("QUEST")
                .value_parser(1..=20)
                .help("Selects the quest to run (1..=20)"),
        )
}

fn main() -> io::Result<()> {
    let matches = app().get_matches();
    let event = matches.get_one::<String>("event").cloned();
    let quest = matches.get_one::<i64>("quest").map(|&n| n as _);
    everybody_codes::run(event, quest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_app() {
        app().debug_assert();
    }
}
