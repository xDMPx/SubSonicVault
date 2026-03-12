use actix_web::{App, HttpServer, middleware::Logger, web};
use std::sync::Mutex;
use subsonic_vault::services::{
    get_file_artwork_by_id, get_file_by_id, get_file_metadata_by_id, get_files, home, ping, scan,
};
use subsonic_vault::{AppState, ProgramOption, print_help, process_args, traverse_dir};

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
            print_help();
            std::process::exit(-1);
        })
        .unwrap();
    if options.contains(&ProgramOption::PrintHelp) {
        print_help();
        std::process::exit(-1);
    }

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

    let cache = std::collections::HashMap::new();
    let (audiofiles, cache) = traverse_dir(&base_dir, cache).unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                base_dir: base_dir.clone(),
                audiofiles: Mutex::new(audiofiles.clone()),
                hashing_cache: Mutex::new(cache.clone()),
            }))
            .wrap(Logger::default())
            .service(home)
            .service(scan)
            .service(get_files)
            .service(get_file_by_id)
            .service(get_file_metadata_by_id)
            .service(get_file_artwork_by_id)
            .service(ping)
            .service(actix_files::Files::new("/player", "./player/dist").index_file("index.html"))
            .service(actix_files::Files::new("/assets", "./player/dist/assets"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
