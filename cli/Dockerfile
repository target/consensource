FROM target/consensource:build_agent

# Need sawtooth-sdk to create keys
RUN echo "deb http://repo.sawtooth.me/ubuntu/1.0/stable xenial universe" >> /etc/apt/sources.list \
 && apt-get update \
 && apt-get install -y -q --allow-unauthenticated \
      python3-sawtooth-sdk \
      python3-sawtooth-cli \
      unzip \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

COPY . .

WORKDIR cli
RUN cargo build

ENV PATH=$PATH:/project/consensource/cli/target/debug/