
mod memory;

use memory::{MemoryReader, Error, Result};

fn main() -> Result<()> {
    let mut game_reader = MemoryReader::new("Diablo IV.exe")?;
    println!("{}", game_reader.get_process_id());

    Ok(())
}
