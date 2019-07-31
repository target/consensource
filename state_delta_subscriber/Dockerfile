FROM target/consensource:build_agent

COPY . .

WORKDIR state_delta_subscriber

RUN rustup default nightly
RUN cargo build
