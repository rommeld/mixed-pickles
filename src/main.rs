use mixed_pickles::latest_commit;
use std::io;

fn main() -> io::Result<()> {
    let parsed_commit = latest_commit().ok_or("Was not able to parse commit.");
    println!("{:?}", parsed_commit);
    Ok(())
}
