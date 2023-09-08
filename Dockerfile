FROM rust:1.67.0 AS r-build

# Set the workdir and copy the source files for building
# If the Cargo.toml or Cargo.lock files have not changed,
WORKDIR /usr/src/mybuild
COPY ./authn_service ./

# Build in offline mode for SQLx is possible, though we will build with a local connection
# See: https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query
# See: https://github.com/launchbadge/sqlx
ENV DATABASE_URL="postgres://postgres:password@host.docker.internal:5432/authentication"

RUN cargo build --release 

# For this we will need the sqlx-cli for applying migrations
RUN cargo install --version='~0.6' sqlx-cli --no-default-features --features rustls,postgres

RUN chmod +x ./target/release/authn_service

# Better ways to do this, but we'll have an init script run migrations
CMD ["./target/release/authn_service"]