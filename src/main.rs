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
    let base_dir: String = std::env::args()
        .skip(1)
        .next()
        .expect("Arg with dir location required");

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
    .bind(("0.0.0.0", 65421))?
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

    HttpResponse::Ok()
        .content_type(extension_to_mime(file_ext))
        .body(std::fs::read(file).unwrap())
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

    HttpResponse::Ok()
        .content_type(extension_to_mime(file_ext))
        .body(std::fs::read(file).unwrap())
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
