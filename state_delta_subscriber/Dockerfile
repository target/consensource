FROM target/consensource:rust-nightly

COPY . .
WORKDIR state_delta_subscriber
RUN cargo update
RUN cargo +nightly build
ENV PATH=$PATH:/state_delta_subscriber/target/debug/
