FROM target/consensource:rust-base-1.38-nightly

COPY protos /protos
COPY common /common
RUN cd /common \
    && rustup default stable \
    && cargo build

COPY database /database
COPY state_delta_subscriber /state_delta_subscriber
WORKDIR state_delta_subscriber
RUN cargo build

ENV PATH=$PATH:/state_delta_subscriber/target/debug/