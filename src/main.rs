use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use std::env;

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let port = env::var("PORT").unwrap_or("8080".to_string());

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
