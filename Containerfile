FROM rust:1.67.1-bullseye as chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim as runtime
WORKDIR /app
COPY --from=builder /app/target/release/api /usr/local/bin
COPY --from=builder /app/.env .
ENTRYPOINT [ "/usr/local/bin/api" ]
