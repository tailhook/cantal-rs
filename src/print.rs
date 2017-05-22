use std::io::{self, Write};

use collection::Collection;


pub fn print<C: Collection, W: Write>(col: C, out: W) -> io::Result<()> {
    Ok(())
}
