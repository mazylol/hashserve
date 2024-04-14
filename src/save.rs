use std::{
    fs::OpenOptions,
    io::{BufRead, Write},
};

pub fn save(command: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("data.hsrv")?;

    if let Err(e) = writeln!(file, "{}", command) {
        eprintln!("Couldn't write to file: {}", e);
    }

    Ok(())
}

pub fn load() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = OpenOptions::new().read(true).open("data.hsrv")?;
    let reader = std::io::BufReader::new(file);

    let commands: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    Ok(commands)
}
