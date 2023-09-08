use crate::authn::ports::{TokenDatabase, TokenCache, AuthnService};
use crate::routes::{health_handler, readiness_handler, issue_token_handler, validate_token_handler, };
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use opentelemetry::sdk::export;
use opentelemetry_prometheus::PrometheusExporter;
use sqlx::PgPool;
use std::net::TcpListener;
use std::sync::Arc;
use actix_web_opentelemetry::{RequestTracing, RequestMetrics, PrometheusMetricsHandler};

pub fn configure_service<T: 'static + AuthnService>(service: web::Data<T>, cfg: &mut web::ServiceConfig) {
    cfg.app_data(service);
    cfg.route("/", web::get().to(validate_token_handler::<T>));
    cfg.route("/", web::post().to(issue_token_handler::<T>));
}

fn configure_auth(redis_client: Arc<redis::Client>, pg_pool: Arc<PgPool>, cfg: &mut web::ServiceConfig){
    use crate::authn::authn_service_impl::AuthnServiceImpl;
    use crate::authn::postgres_token_repo::PostgresTokenDatabaseImpl;
    use crate::authn::redis_token_repo::RedisTokenCacheImpl;

    let auth_database = PostgresTokenDatabaseImpl::build(pg_pool);
    let auth_cache = RedisTokenCacheImpl::build(redis_client);
    let auth_service: AuthnServiceImpl<RedisTokenCacheImpl, PostgresTokenDatabaseImpl> = AuthnServiceImpl { token_cache: auth_cache, token_database: auth_database };

    configure_service(web::Data::new(auth_service), cfg);
}

fn configure_features(redis_client: Arc<redis::Client>, pg_pool: Arc<PgPool>, cfg: &mut web::ServiceConfig) {
    configure_auth(redis_client.clone(), pg_pool.clone(), cfg);
}

pub fn run_app(listener: TcpListener, db_pool: Arc<PgPool>, cache_client: Arc<redis::Client>, request_metrics: RequestMetrics) -> Result<Server, std::io::Error> {    
    let server = HttpServer::new(move || {
        App::new()
            //Start producing OT spans when a HTTP request is being received by actix
            .wrap(RequestTracing::new())
            .wrap(request_metrics.clone())
            //Register our routes for our non-app services
            .route("/health", web::get().to(health_handler))
            .route("/readiness", web::get().to(readiness_handler))
            .configure(|cfg| configure_features(cache_client.clone(), db_pool.clone(), cfg))
            //Ensure our DB connection is available to our handlers
            
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub fn run_metrics(listener: TcpListener, exporter: PrometheusExporter) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            //Start producing OT spans when a HTTP request is being received by actix
            .wrap(RequestTracing::new())
            .route("/metrics", web::get().to(PrometheusMetricsHandler::new(exporter.clone())))
    })
    .listen(listener)?
    .run();
    Ok(server)
}