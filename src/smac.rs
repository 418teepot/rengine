use std::{fs, io::Write};
pub fn smac() -> std::io::Result<()> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let mut log_file = fs::File::create("test.log")?;
    log_file.write_all(input.as_bytes())?;
    println!("cost={input}");
    Ok(())
}