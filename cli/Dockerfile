FROM target/consensource:rust-base-1.30

COPY protos /protos
COPY common /common
RUN cd /common \
    && cargo build

COPY cli /cli
WORKDIR cli
RUN cargo build

ENV PATH=$PATH:/cli/target/debug/