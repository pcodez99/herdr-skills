mod application;
mod cli;
mod domain;
mod repository;
mod storage;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    #[expect(
        unsafe_code,
        reason = "umask keeps every file this CLI creates, SQLite journals included, private; std has no safe wrapper"
    )]
    unsafe {
        libc::umask(0o077)
    };
    if let Err(error) = cli::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
