FROM rust:1.62.0 as builder

WORKDIR /app
RUN apt update && apt install lld clang -y

COPY . .

RUN cargo build --release

FROM rust:1.62.0 as runtime

WORKDIR /app

COPY --from=builder /app/target/release/chronicle-emulator chronicle-emulator

ENTRYPOINT ["./chronicle-emulator"]
