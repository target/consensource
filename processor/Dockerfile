FROM target/consensource:rust-base-1.38-nightly

COPY protos /protos
COPY common /common
RUN cd /common \
    && cargo build

COPY processor /processor
WORKDIR processor
RUN cargo build

ENV PATH=$PATH:/processor/target/debug/