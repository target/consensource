FROM rust:latest

RUN apt-get update && apt-get install -y unzip libzmq3-dev protobuf-compiler

WORKDIR /project/consensource

COPY . .

WORKDIR rest_api
RUN rustup update nightly
RUN rustup override set nightly
RUN cargo build

ENV PATH=$PATH:/project/consensource/rest_api/target/debug/
