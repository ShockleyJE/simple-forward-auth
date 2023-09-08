# Zero-To-Rust-At-Neurelo
As presented at Carolina Code Conference 2023

This is a simple authentication server designed to work with forward-auth middleware found with in most reverse proxies.  

The demonstration fill be done with Traefik & the Forward-Auth middleware with Kubernetes as the target platform

We will have a live load test, and use Actix request-response level metrics emmitted by [OTEL-prometheus](https://crates.io/crates/opentelemetry-prometheus), and function-level metrics via [Autometrics](https://autometrics.dev/)

We'll look into tracinng, and maybe even a metrics-tracing crossover episode with exemplars if we can. 

## Getting started

Install autometrics
```
brew install autometrics-dev/tap/am
```

Start the autometrics prometheus server
```
am start http://localhost:9464/metrics
# http://127.0.0.1:6789/explorer/#/functions
```

Start Jaeger
```
docker run -d --name jaeger \
  -e COLLECTOR_ZIPKIN_HOST_PORT=:9411 \
  -p 5775:5775/udp \
  -p 6831:6831/udp \
  -p 6832:6832/udp \
  -p 5778:5778 \
  -p 16686:16686 \
  -p 14268:14268 \
  -p 14250:14250 \
  -p 9411:9411 \
  jaegertracing/all-in-one:1.22
```

# Add sqlx to path (Ubuntu)
cargo install sqlx-cli
export PATH=/home/ubuntu/.cargo/bin:$PATH

## Start the database & cache (requires docker)

```
cd data-plane
chmod +x authn_service/scripts/init_db.sh && chmod +x authn_service/scripts/init_cache.sh
authn_service/scripts/init_db.sh && authn_service/scripts/init_cache.sh
```

## Run the server

```
cd data-plane
cargo run -p authn_service
```

## Mock a request to issue an API token

```
curl --location 'localhost:8000/' \
--header 'Content-Type: application/json' \
--data '{
    "environment" : "howderino"
}'
```

## Bulid SQLx offline build files
```
❯ cargo sqlx prepare --workspace -- -p authn_service
```

## Build with DB passthrough on amazon linux
```
docker build . --add-host host.docker.internal:host-gateway -t shockleyje/ccc-demo
```

## Demo

Grafana
```
k port-forward -n monitoring svc/grafana 3030:3000
http://localhost:3030/login
admin:<secret>
```

Auth service
```
k port-forward svc/auth-service 9090:80
```

K6
```
k6 run load.js
```