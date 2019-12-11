FROM target/consensource:rust-base-1.38

COPY database /database
COPY rest_api /rest_api
WORKDIR rest_api
RUN rustup default nightly \
    && cargo build

ENV PATH=$PATH:/rest_api/target/debug/