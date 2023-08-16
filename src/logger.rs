use std::{
    env,
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
};

pub struct Logger {
    log_file: String,
}

impl Logger {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let log_dir = format!("{}/logs", env::var("HOME").unwrap());
        let p = Path::new(&log_dir);
        if (!p.is_dir()) {
            fs::create_dir(p)?;
        }

        let log_file = format!("{}/ycurl.txt", log_dir);
        let p = Path::new(&log_file);
        if (!p.is_file()) {
            File::create(p)?;
        }

        Ok(Self { log_file })
    }

    pub fn log(&self, s: &str) -> Result<(), Box<dyn Error>> {
        fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.log_file)?
            .write_all((s.to_owned() + "\n").as_bytes())
            .map_err(|e| e.into())
    }
}
