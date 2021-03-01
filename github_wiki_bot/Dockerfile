FROM rust AS builder
RUN apt update && apt install -y libssl-dev ca-certificates openssl && rustup component add rustfmt
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
