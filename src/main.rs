use actix_web::{App, HttpResponse, HttpServer, Responder, get};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(ping))
        .bind(("0.0.0.0", 65420))?
        .run()
        .await
}
