use actix_web::{web, HttpResponse, Responder};

pub async fn get_data() -> impl Responder {
    HttpResponse::Ok().json("Some data")
}

pub async fn post_data(item: web::Json<String>) -> impl Responder {
    HttpResponse::Created().json(item.into_inner())
}
