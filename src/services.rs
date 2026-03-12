use crate::{
    AppState, AudioFile, AudioFileMetadata, PingResponse, extension_to_mime, traverse_dir,
};
use actix_web::{CustomizeResponder, HttpRequest, HttpResponse, Responder, get, web};
use lofty::{
    file::{AudioFile as LofyAudioFile, TaggedFileExt},
    tag::Accessor,
};
use rand::Rng;

#[derive(Debug)]
pub enum ServiceError {
    PoisonError,
    ValuesExtractionError,
}

#[get("/")]
async fn home(data: web::Data<AppState>) -> impl Responder {
    if let Ok(responder) = _home(data) {
        responder
    } else {
        HttpResponse::InternalServerError()
            .body("Internal Server Error")
            .customize()
    }
}

fn _home(data: web::Data<AppState>) -> Result<CustomizeResponder<HttpResponse>, ServiceError> {
    let audiofiles = data
        .audiofiles
        .lock()
        .map_err(|_| ServiceError::PoisonError)?;
    let audiofiles_len = audiofiles.values().count();
    let audiofiles = audiofiles.values();
    let mut rng = rand::rng();
    let i = rng.random_range(..audiofiles_len);

    let values = (|| {
        let file = audiofiles.skip(i - 1).next()?;
        let file_ext = file.extension()?;
        let file_name = file.file_name()?;
        let mime = extension_to_mime(file_ext)?;
        let file_body = std::fs::read(file).ok()?;
        Some((file_body, file_name, mime))
    })();

    let (file_body, file_name, mime) = values.ok_or(ServiceError::ValuesExtractionError)?;

    Ok(HttpResponse::Ok()
        .content_type(mime)
        .body(file_body)
        .customize()
        .insert_header((
            "Content-Disposition",
            format!("inline; filename*=UTF-8''{}", file_name.to_string_lossy()),
        )))
}

#[get("/scan")]
async fn scan(data: web::Data<AppState>) -> impl Responder {
    if let Ok(mut cache) = data.hashing_cache.lock() {
        if let Ok((files, updated_cache)) = traverse_dir(&data.base_dir, cache.clone()) {
            if let Ok(mut audiofiles) = data.audiofiles.lock() {
                *audiofiles = files.clone();
                *cache = updated_cache;

                let files = files.iter().map(|(k, v)| format!("{}:{:?}\n", k, v));
                let mut files = files.collect::<Vec<String>>();
                files.sort_unstable();

                HttpResponse::Ok()
                    .content_type("text/plain; charset=utf-8")
                    .body(files.concat())
            } else {
                HttpResponse::InternalServerError().body("Internal Server Error")
            }
        } else {
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    } else {
        HttpResponse::InternalServerError().body("Internal Server Error")
    }
}

#[get("/files")]
async fn get_files(data: web::Data<AppState>) -> impl Responder {
    if let Ok(audiofiles) = data.audiofiles.lock() {
        let audiofiles: Vec<AudioFile> = audiofiles
            .iter()
            .filter_map(|(hash, f)| {
                let mime = extension_to_mime(f.extension()?)?;
                Some(AudioFile {
                    id: hash.to_owned(),
                    path: format!("{f:?}"),
                    mime,
                })
            })
            .collect();

        let audiofiles_json = serde_json::to_vec(&audiofiles);

        if let Ok(audiofiles_json) = audiofiles_json {
            return HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .body(audiofiles_json);
        }
    }
    HttpResponse::InternalServerError().body("Internal Server Error")
}

#[get("/file/{id}")]
async fn get_file_by_id(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let hash = path.into_inner();
    if let Ok(audiofiles) = data.audiofiles.lock() {
        if let Some(file) = audiofiles.get(&hash) {
            let values = (|| {
                let file_ext = file.extension()?;
                let file_name = file.file_name()?;
                let mime = extension_to_mime(file_ext)?;

                let file_body = std::fs::read(file).ok()?;
                Some((file_body, file_name, mime))
            })();
            if let Some((file_body, file_name, mime)) = values {
                return HttpResponse::Ok()
                    .content_type(mime)
                    .body(file_body)
                    .customize()
                    .insert_header((
                        "Content-Disposition",
                        format!("inline; filename*=UTF-8''{}", file_name.to_string_lossy()),
                    ));
            } else {
                return HttpResponse::InternalServerError()
                    .body("Internal Server Error")
                    .customize();
            }
        } else {
            return HttpResponse::NotFound().body("Invalid hash").customize();
        }
    }
    HttpResponse::InternalServerError()
        .body("Internal Server Error")
        .customize()
}

#[get("/file/{id}/metadata")]
async fn get_file_metadata_by_id(
    req: HttpRequest,
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let hash = path.into_inner();
    if let Ok(audiofiles) = data.audiofiles.lock() {
        if let Some(file) = audiofiles.get(&hash) {
            if let Ok(tagged_file) = lofty::read_from_path(file) {
                let title = tagged_file
                    .tags()
                    .iter()
                    .find_map(|t| t.title())
                    .map(|x| x.to_string());
                let artist = tagged_file
                    .tags()
                    .iter()
                    .find_map(|t| t.artist())
                    .map(|x| x.to_string());
                let album = tagged_file
                    .tags()
                    .iter()
                    .find_map(|t| t.album())
                    .map(|x| x.to_string());
                let genre = tagged_file
                    .tags()
                    .iter()
                    .find_map(|t| t.genre())
                    .map(|x| x.to_string());
                let release_year = tagged_file
                    .tags()
                    .iter()
                    .find_map(|t| t.date())
                    .map(|x| x.to_string());
                let duration = tagged_file.properties().duration().as_secs();

                let picture_count: u32 = tagged_file.tags().iter().map(|t| t.picture_count()).sum();
                let artwork_url = if picture_count != 0 {
                    let url = req.full_url().join("metadata/artwork");
                    if let Ok(url) = url {
                        Some(url.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                let metadata = AudioFileMetadata {
                    title,
                    artist,
                    album,
                    genre,
                    release_year,
                    duration,
                    artwork_url,
                };

                if let Ok(metadata_json) = serde_json::to_vec(&metadata) {
                    return HttpResponse::Ok()
                        .content_type("application/json; charset=utf-8")
                        .body(metadata_json);
                }
            }
        } else {
            return HttpResponse::NotFound().body("Invalid hash");
        }
    }
    HttpResponse::InternalServerError().body("Internal Server Error")
}

#[get("/file/{id}/metadata/artwork")]
async fn get_file_artwork_by_id(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let hash = path.into_inner();
    if let Ok(audiofiles) = data.audiofiles.lock() {
        if let Some(file) = &audiofiles.get(&hash) {
            if let Ok(tagged_file) = lofty::read_from_path(file) {
                let artwork = tagged_file.tags().iter().flat_map(|t| t.pictures()).next();

                if let Some(artwork) = artwork {
                    return HttpResponse::Ok().body(artwork.data().to_vec());
                } else {
                    return HttpResponse::NotFound().body("No embedded cover art");
                }
            }
        } else {
            return HttpResponse::NotFound().body("Invalid hash");
        }
    }
    HttpResponse::InternalServerError().body("Internal Server Error")
}

#[get("/ping")]
async fn ping() -> impl Responder {
    if let Ok(body) = serde_json::to_vec(&PingResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }) {
        HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .body(body)
    } else {
        HttpResponse::InternalServerError().body("Internal Server Error")
    }
}
