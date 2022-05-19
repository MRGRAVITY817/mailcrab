# Builder stage
FROM rust:1.60.0 AS builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim AS runtime 

WORKDIR /app
COPY --from=builder /app/target/release/mailcrab mailcrab
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./mailcrab"]
