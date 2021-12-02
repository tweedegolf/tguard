FROM rust:bullseye

RUN cargo install --locked trunk wasm-bindgen-cli cargo-watch;

RUN rustup target add wasm32-unknown-unknown;