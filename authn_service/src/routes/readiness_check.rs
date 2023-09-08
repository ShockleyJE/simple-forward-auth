use actix_web::HttpResponse;
use autometrics::autometrics;

#[autometrics]
pub async fn readiness_handler() -> HttpResponse {
    HttpResponse::Ok().finish()
}