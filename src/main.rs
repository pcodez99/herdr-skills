mod application;
mod cli;
mod domain;
mod repository;
mod storage;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    // All files created by this single-threaded CLI, including SQLite journals, are private.
    unsafe { libc::umask(0o077) };
    if let Err(error) = cli::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
