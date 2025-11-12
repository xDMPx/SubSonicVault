use std::str::FromStr;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};

struct AppState {
    base_dir: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let base_dir: String = std::env::args()
        .skip(1)
        .next()
        .expect("Arg with dir location required");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                base_dir: base_dir.clone(),
            }))
            .service(home)
    })
    .bind(("0.0.0.0", 65421))?
    .run()
    .await
}

#[get("/")]
async fn home(data: web::Data<AppState>) -> impl Responder {
    let files = traverse_dir(&data.base_dir);
    let files = files.iter().map(|p| format!("{:?}\n", p));
    let mut files = files.collect::<Vec<String>>();
    files.sort_unstable();

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(files.concat())
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
