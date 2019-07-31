FROM rust:latest

RUN apt-get update && apt-get install -y unzip libzmq3-dev protobuf-compiler

WORKDIR /project/consensource

COPY . .

WORKDIR rest_api
RUN rustup override set nightly
RUN rustup install nightly-2019-07-03
RUN cargo update
RUN cargo build

ENV PATH=$PATH:/project/consensource/rest_api/target/debug/
