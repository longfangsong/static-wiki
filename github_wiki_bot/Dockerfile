FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
COPY . /github_wiki_bot
WORKDIR /github_wiki_bot
RUN cargo build --release

FROM alpine
MAINTAINER longfangsong@icloud.com
COPY --from=builder /github_wiki_bot/target/release/github_wiki_bot /
WORKDIR /
ENTRYPOINT ["/github_wiki_bot"]
