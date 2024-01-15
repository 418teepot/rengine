use std::{fs, io::Write, env};
pub fn smac() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut log_file = fs::File::create("test.log")?;
    for arg in args.iter() {
        log_file.write_all(arg.as_bytes())?;
        log_file.write_all(b"\n")?;
    }
    println!("cost={}", args[0]);
    Ok(())
}