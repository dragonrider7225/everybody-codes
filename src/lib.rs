//! This crate aggregates my solutions to all [Everybody Codes](https://everybody.codes/) problems.

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

use std::io;

use extended_io as eio;

/// The entry point for my solutions to Everybody Codes.
pub fn run(event: Option<String>, quest: Option<u32>) -> io::Result<()> {
    let event = match event {
        Some(event) => event,
        None => eio::prompt("Enter the event to run: ")?,
    };
    let quest_prompt = move || {
        quest
            .ok_or(())
            .or_else(|_| eio::prompt("Enter quest to run: "))
    };
    match &*event {
        "algorithmia" => algorithmia::run_quest(quest_prompt()?),
        _ => unimplemented!("Event {}", event),
    }
}
