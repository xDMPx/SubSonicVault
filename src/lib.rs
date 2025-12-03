use md5::{Digest, Md5};
use std::io::{Read, Seek};
use std::str::FromStr;
use std::sync::Mutex;

pub struct AppState {
    pub base_dir: String,
    pub audiofiles: Mutex<std::collections::HashMap<String, std::path::PathBuf>>,
}

#[derive(serde::Serialize)]
pub struct AudioFile {
    pub id: String,
    pub path: String,
    pub mime: String,
}

#[derive(PartialEq)]
pub enum ProgramOption {
    BaseDir(std::path::PathBuf),
    Port(u16),
    PrintHelp,
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

pub fn traverse_dir(
    base_dir: &str,
) -> Result<std::collections::HashMap<String, std::path::PathBuf>, HashError> {
    let mut dir_list = vec![std::path::PathBuf::from_str(base_dir).unwrap()];
    let mut audiofiles_paths = Vec::new();
    while dir_list.len() > 0 {
        let entries = std::fs::read_dir(dir_list.pop().unwrap()).unwrap();
        for entry in entries {
            if let Ok(file) = entry {
                if let Ok(file_type) = file.file_type() {
                    if file_type.is_file() && is_audiofile(file.path()) {
                        audiofiles_paths.push(file.path());
                    } else if file_type.is_dir() {
                        dir_list.push(file.path());
                    }
                }
            }
        }
    }

    let mut audiofiles = std::collections::HashMap::new();
    let audiofiles_paths_len = audiofiles_paths.len();
    let workers = std::thread::available_parallelism()
        .map(|x| x.get())
        .unwrap_or(2)
        - 1;

    let mut handles = vec![];
    for _ in 0..workers {
        let split_index = audiofiles_paths.len() - (audiofiles_paths_len / (workers));
        let chunk = audiofiles_paths.split_off(split_index);
        let handle = std::thread::spawn(|| {
            let mut audiofiles = vec![];
            for path in chunk {
                audiofiles.push((hex::encode(md5_hash(&path).unwrap()), path));
            }
            audiofiles
        });
        handles.push(handle);
    }
    for path in audiofiles_paths {
        audiofiles.insert(hex::encode(md5_hash(&path).unwrap()), path);
    }
    for handle in handles {
        audiofiles.extend(handle.join().unwrap());
    }

    Ok(audiofiles)
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
    if last_arg != "--help" {
        let base_dir_path = last_arg;
        let base_dir_path = std::path::PathBuf::from(base_dir_path);
        if !base_dir_path.is_dir() {
            return Err(Error::InvalidOptionsStructure);
        }
        options.push(ProgramOption::BaseDir(base_dir_path));
    } else {
        args.push(last_arg);
    }

    for arg in args {
        let arg = match arg.as_str() {
            "--help" => Ok(ProgramOption::PrintHelp),
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

pub fn print_help() {
    println!("Usage: {} [OPTIONS] DIRECTORY", env!("CARGO_PKG_NAME"));
    println!("       {} --help", env!("CARGO_PKG_NAME"));
    println!("Options:");
    println!("\t --help");
    println!("\t --port=<u16>");
}

#[derive(Debug)]
pub struct HashError {
    path: std::path::PathBuf,
    error: std::io::Error,
}

const BUF_SIZE: usize = 1024 * 1024;

fn md5_hash(path: &std::path::Path) -> Result<Vec<u8>, HashError> {
    let mut hasher = Md5::new();

    let mut file = std::fs::File::open(path).map_err(|e| HashError {
        path: path.to_owned(),
        error: e,
    })?;
    let mut buf: Vec<u8> = vec![0; BUF_SIZE];
    while file
        .metadata()
        .map_err(|e| HashError {
            path: path.to_owned(),
            error: e,
        })?
        .len()
        - file.stream_position().map_err(|e| HashError {
            path: path.to_owned(),
            error: e,
        })?
        > BUF_SIZE as u64
    {
        file.read_exact(&mut buf).map_err(|e| HashError {
            path: path.to_owned(),
            error: e,
        })?;
        hasher.update(&buf);
    }

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| HashError {
        path: path.to_owned(),
        error: e,
    })?;
    hasher.update(buf);

    let hash = hasher.finalize();

    Ok(hash.to_vec())
}
