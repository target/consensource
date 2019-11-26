FROM target/consensource:rust-nightly

COPY . .
WORKDIR processor
RUN cargo update
RUN cargo +nightly build
ENV PATH=$PATH:/processor/target/debug/
