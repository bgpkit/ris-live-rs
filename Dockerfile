# select build image
FROM rust:1.46 as build

# create a new empty shell project
RUN USER=root cargo new --bin ris_live_rs
WORKDIR /ris_live_rs

# copy your source tree
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml

# build for release
RUN cargo build --release

# our final base
FROM debian:buster-slim
LABEL maintainer="mingwei@bgpkit.com"
LABEL org.opencontainers.image.source="https://github.com/bgpkit/ris-live-rs"
LABEL org.opencontainers.image.description="ris-live-reader is a commandline tool that reads real-time bgp messagse from RIPE RIS Live websocket stream."

RUN DEBIAN=NONINTERACTIVE apt update; apt install -y libssl-dev libpq-dev ca-certificates tzdata tini; rm -rf /var/lib/apt/lists/*

# copy the build artifact from the build stage
COPY --from=build /ris_live_rs/target/release/ris-live-reader /usr/local/bin


# set the startup command to run your binary
ENTRYPOINT ["tini", "--", "ris-live-reader"]
