#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]
#![allow(
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::single_call_fn,
    clippy::question_mark_used
)]

fn main() {
    match uyamlt::run(&std::env::args().collect::<Vec<_>>()) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(0)
        }
    }
}
