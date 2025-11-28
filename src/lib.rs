use std::str::FromStr;
use std::sync::Mutex;

pub struct AppState {
    pub base_dir: String,
    pub audiofiles: Mutex<Vec<std::path::PathBuf>>,
}

#[derive(serde::Serialize)]
pub struct AudioFile {
    pub id: u64,
    pub path: String,
    pub mime: String,
}

pub enum ProgramOption {
    BaseDir(std::path::PathBuf),
    Port(u16),
}

#[derive(Debug)]
pub enum Error {
    InvalidOption(String),
    InvalidOptionsStructure,
}

pub fn is_audiofile(path: std::path::PathBuf) -> bool {
    if let Some(ext) = path.extension() {
        if ext == "m4b" {
            return true;
        } else if ext == "m4a" {
            return true;
        } else if ext == "mp3" {
            return true;
        } else if ext == "flac" {
            return true;
        } else if ext == "wav" {
            return true;
        } else if ext == "opus" {
            return true;
        }
    }

    false
}

pub fn traverse_dir(base_dir: &str) -> Vec<std::path::PathBuf> {
    let mut dir_list = vec![std::path::PathBuf::from_str(base_dir).unwrap()];
    let mut audiofiles = Vec::new();
    while dir_list.len() > 0 {
        let entries = std::fs::read_dir(dir_list.pop().unwrap()).unwrap();
        for entry in entries {
            if let Ok(file) = entry {
                if let Ok(file_type) = file.file_type() {
                    if file_type.is_file() && is_audiofile(file.path()) {
                        audiofiles.push(file.path());
                    } else if file_type.is_dir() {
                        dir_list.push(file.path());
                    }
                }
            }
        }
    }

    return audiofiles;
}

pub fn extension_to_mime(file_ext: &std::ffi::OsStr) -> String {
    match file_ext.to_str().unwrap() {
        "m4b" | "m4a" => "audio/mp4".to_owned(),
        ext => format!("audio/{}", ext),
    }
}

pub fn process_args() -> Result<Vec<ProgramOption>, Error> {
    let mut options = vec![];
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    let last_arg = args.pop().ok_or(Error::InvalidOptionsStructure)?;
    let base_dir_path = last_arg;
    let base_dir_path = std::path::PathBuf::from(base_dir_path);
    if !base_dir_path.is_dir() {
        return Err(Error::InvalidOptionsStructure);
    }
    options.push(ProgramOption::BaseDir(base_dir_path));

    for arg in args {
        let arg = match arg.as_str() {
            s if s.starts_with("--port=") => {
                if let Some(Ok(port)) = s.split_once('=').map(|(_, s)| s.parse::<u16>()) {
                    Ok(ProgramOption::Port(port))
                } else {
                    Err(Error::InvalidOption(arg))
                }
            }
            _ => Err(Error::InvalidOption(arg)),
        };
        options.push(arg?);
    }

    Ok(options)
}
