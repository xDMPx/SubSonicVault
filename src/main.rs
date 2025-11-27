use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger, web};
use rand::Rng;
use std::str::FromStr;
use std::sync::Mutex;

struct AppState {
    base_dir: String,
    audiofiles: Mutex<Vec<std::path::PathBuf>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let options = process_args()
        .map_err(|err| {
            match err {
                Error::InvalidOption(option) => eprintln!("Provided option {option} is invalid"),
                Error::InvalidOptionsStructure => eprintln!("Invalid input"),
            }
            std::process::exit(-1);
        })
        .unwrap();

    let port = if let Some(port) = options.iter().find_map(|o| match o {
        ProgramOption::Port(p) => Some(*p),
        _ => None,
    }) {
        port
    } else {
        65421
    };

    let base_dir = options
        .iter()
        .find_map(|o| match o {
            ProgramOption::BaseDir(path) => Some(path.to_str().unwrap().to_string()),
            _ => None,
        })
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                base_dir: base_dir.clone(),
                audiofiles: Mutex::new(traverse_dir(&base_dir)),
            }))
            .wrap(Logger::default())
            .service(home)
            .service(scan)
            .service(get_files)
            .service(get_file_by_id)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

#[get("/")]
async fn home(data: web::Data<AppState>) -> impl Responder {
    let audiofiles = data.audiofiles.lock().unwrap();
    let mut rng = rand::rng();
    let i = rng.random_range(..audiofiles.len());

    let file = &audiofiles[i];
    let file_ext = file.extension().unwrap();
    let file_name = file.file_name().unwrap();

    HttpResponse::Ok()
        .content_type(extension_to_mime(file_ext))
        .body(std::fs::read(file).unwrap())
        .customize()
        .insert_header((
            "Content-Disposition",
            format!("inline; filename*=UTF-8''{}", file_name.to_string_lossy()),
        ))
}

#[get("/scan")]
async fn scan(data: web::Data<AppState>) -> impl Responder {
    let files = traverse_dir(&data.base_dir);
    let mut audiofiles = data.audiofiles.lock().unwrap();
    *audiofiles = files.clone();

    let files = files.iter().map(|p| format!("{:?}\n", p));
    let mut files = files.collect::<Vec<String>>();
    files.sort_unstable();

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(files.concat())
}

#[derive(serde::Serialize)]
struct AudioFile {
    id: u64,
    path: String,
    mime: String,
}

#[get("/files")]
async fn get_files(data: web::Data<AppState>) -> impl Responder {
    let audiofiles = data.audiofiles.lock().unwrap();
    let audiofiles: Vec<AudioFile> = audiofiles
        .iter()
        .enumerate()
        .map(|(i, f)| AudioFile {
            id: i as u64,
            path: format!("{f:?}"),
            mime: extension_to_mime(f.extension().unwrap()),
        })
        .collect();

    let audiofiles_json = serde_json::to_vec(&audiofiles).unwrap();

    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(audiofiles_json)
}

#[get("/file/{id}")]
async fn get_file_by_id(data: web::Data<AppState>, path: web::Path<usize>) -> impl Responder {
    let file_id = path.into_inner();
    let audiofiles = data.audiofiles.lock().unwrap();

    let file = &audiofiles[file_id];
    let file_ext = file.extension().unwrap();
    let file_name = file.file_name().unwrap();

    HttpResponse::Ok()
        .content_type(extension_to_mime(file_ext))
        .body(std::fs::read(file).unwrap())
        .customize()
        .insert_header((
            "Content-Disposition",
            format!("inline; filename*=UTF-8''{}", file_name.to_string_lossy()),
        ))
}

fn is_audiofile(path: std::path::PathBuf) -> bool {
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

fn traverse_dir(base_dir: &str) -> Vec<std::path::PathBuf> {
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

fn extension_to_mime(file_ext: &std::ffi::OsStr) -> String {
    match file_ext.to_str().unwrap() {
        "m4b" | "m4a" => "audio/mp4".to_owned(),
        ext => format!("audio/{}", ext),
    }
}

enum ProgramOption {
    BaseDir(std::path::PathBuf),
    Port(u16),
}

#[derive(Debug)]
enum Error {
    InvalidOption(String),
    InvalidOptionsStructure,
}

fn process_args() -> Result<Vec<ProgramOption>, Error> {
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
