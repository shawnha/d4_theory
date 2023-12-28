
mod memory;

use memory::{MemoryReader, Result};

fn main() -> Result<()> {
    let game_reader = MemoryReader::new("Diablo IV.exe")?;
    println!("{}", game_reader.process_id);

    Ok(())
}
