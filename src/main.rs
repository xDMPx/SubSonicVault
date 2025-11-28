use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger, web};
use rand::Rng;
use std::sync::Mutex;
use subsonic_vault::{AppState, ProgramOption, extension_to_mime, process_args, traverse_dir};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let options = process_args()
        .map_err(|err| {
            match err {
                subsonic_vault::Error::InvalidOption(option) => {
                    eprintln!("Provided option {option} is invalid")
                }
                subsonic_vault::Error::InvalidOptionsStructure => eprintln!("Invalid input"),
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
