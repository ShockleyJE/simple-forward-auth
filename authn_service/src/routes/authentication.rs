use actix_web::{HttpResponse, web::Json, web, Responder, HttpRequest};
use tracing::info;
use crate::authn::{
    authn_service_impl::*,
    ports::*
};

// Issue our token. In this case, note that body's JSON must serialize to the type Issuance which we
// defined in our own module, ensuring we don't have to handle that case in our function itself
pub async fn issue_token_handler<T: AuthnService>(service: web::Data<T>, body: Json<Issuance>) -> impl Responder {
    match service.issue(&body).await {
        Some(token) => HttpResponse::Ok().body(token),
        None => HttpResponse::InternalServerError().finish()
    }
}

// Validate the token from the X-API-KEY header value
pub async fn validate_token_handler<T: AuthnService>(req: HttpRequest, service: web::Data<T>) -> impl Responder {
    let mut token = String::from("");

    // If the token does not exist in headers, we return early
    match req.headers().get("X-API-KEY") {
        Some(header_token) => token = header_token.to_str().expect("Expected token to be serializable to string").to_string(),
        None => return HttpResponse::Unauthorized().finish()
    };
    
    // Handle whether we can authenticate a token and return our response. Note
    // that there is not a trailing semicolon, since we intend to return the evaluation
    match service.authenticate(&token).await {
        Some(authenticated) => {
            // In this arm we handle whether a token is authenticated
            match authenticated {
                true => HttpResponse::Ok().finish(),
                false => HttpResponse::Forbidden().finish()
            }
        },
        // In this arm we respond to the lack of presence of
        // an authentication state, likely an error, when
        // authenticating the provided token
        None => HttpResponse::InternalServerError().finish()
    }
}
