use actix_web::HttpResponse;
use autometrics::autometrics;

#[autometrics]
pub async fn health_handler() -> HttpResponse {
    HttpResponse::Ok().finish()
}