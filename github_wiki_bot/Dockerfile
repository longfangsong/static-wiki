FROM rust AS builder
RUN apt update && apt install -y libssl-dev ca-certificates openssl && rustup component add rustfmt
# If we are lucky enough, these will build the dependencies and cache the git layer
COPY ./Cargo.toml /github_wiki_bot/
RUN mkdir /github_wiki_bot/src && echo "fn main() {}" > /github_wiki_bot/src/main.rs
WORKDIR /github_wiki_bot
RUN cargo build --release
# Do the real build
COPY . /github_wiki_bot
WORKDIR /github_wiki_bot
RUN cargo build --release

FROM debian
RUN apt update && apt install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
MAINTAINER longfangsong@icloud.com
COPY --from=builder /github_wiki_bot/target/release/github_wiki_bot /
WORKDIR /
ENV RUST_LOG info
ENTRYPOINT ["/github_wiki_bot"]
