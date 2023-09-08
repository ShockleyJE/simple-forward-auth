use actix_web::dev::Server;
use authn_service::authn::ports::AuthnService;
use authn_service::infrastructure;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPool;
use std::net::TcpListener;
use std::sync::Arc;
use authn_service::configuration::get_configuration;
use authn_service::startup::{run_app, run_metrics};
use actix_web_opentelemetry::{PrometheusMetricsHandler, RequestMetricsBuilder, RequestTracing};
use opentelemetry::{
    global,
    sdk::{
        export::metrics::aggregation,
        metrics::{controllers, processors, selectors},
        propagation::TraceContextPropagator,
        trace::{Sampler, Config},
    },
    trace::Tracer
};
use futures::future::join_all;
use authn_service::telemetry::{get_subscriber, init_subscriber};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // Following the documentation here on otel-prometheus
    // https://docs.rs/actix-web-opentelemetry/0.13.0/actix_web_opentelemetry/
    let controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
            aggregation::cumulative_temporality_selector(),
        )
    )
    .build();
    let exporter = opentelemetry_prometheus::exporter(controller).init();
    let meter = global::meter("actix_web");

    // Request metrics middleware
    let request_metrics = RequestMetricsBuilder::new().build(meter);

    // This is our Jaeger OT exporter which is used to ship our spans. 
    // We are making a lot of assumptions about defaults with 
    // this non-explicit configuration
    //global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    //let _tracer = opentelemetry_jaeger::new_agent_pipeline()
    //    // Assume all events have been sampled - I.e. ship all the traces
    //    .with_trace_config(Config::default().with_sampler(Sampler::AlwaysOn))
    //    .install_simple()?;

    //global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    //let tracer = opentelemetry_jaeger::new_agent_pipeline().install_simple()?;
    
    let subscriber = get_subscriber("authn_service".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Fetch our configuration and connect to auth db + cache
    let configuration = get_configuration().expect("Failed to read configuration.");
    // Todo: have this use the infrastructure module
    let db_connection_pool =
        Arc::new(
            PgPool::connect(configuration.database.connection_string().expose_secret())
                .await
                .expect("Failed to connect to Postgres.")
        );
    let cache_client = Arc::new(infrastructure::redis::configure_with_redis_url(configuration.cache.connection_string().expose_secret()).await); 

    // First we will bind our application server on port 'a', which will return a Server
    let app_address = format!("0.0.0.0:{}", configuration.application_port);
    let app_listener = TcpListener::bind(app_address)?;
    let app_server = run_app(app_listener, db_connection_pool, cache_client, request_metrics)?;
    // We do the same for the metrics server which we create on port 'b'
    let metrics_address = format!("0.0.0.0:{}", configuration.metrics_port);
    let metrics_listener = TcpListener::bind(metrics_address)?;
    let metrics_server = run_metrics(metrics_listener, exporter)?;

    // Now we concurrently poll each of the servers we received
    join_all([app_server, metrics_server]).await;

    Ok(())
}