FROM target/consensource:rust-base-1.38-nightly

COPY database /database
COPY rest_api /rest_api
WORKDIR rest_api
RUN cargo build

ENV PATH=$PATH:/rest_api/target/debug/