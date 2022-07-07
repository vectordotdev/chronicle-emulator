FROM rust:1.62.0

WORKDIR /app
RUN apt update && apt install lld clang -y

COPY . .

RUN cargo build --release

ENTRYPOINT ["./target/release/chronicle-emulator"]
