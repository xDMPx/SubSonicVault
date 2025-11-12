use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};

struct AppState {
    base_dir: String,
}

#[get("/")]
async fn hello(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(format!("Base dir set to: {}", data.base_dir))
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok()
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
            .service(hello)
            .service(ping)
    })
    .bind(("0.0.0.0", 65421))?
    .run()
    .await
}
