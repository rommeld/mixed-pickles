use mixed_pickles::fetch_log;
use std::io;

fn main() -> io::Result<()> {
    match fetch_log() {
        Ok(log) => println!("{:#?}", log),
        Err(e) => eprintln!("Failed to run git log: {}", e),
    }
    Ok(())
}
