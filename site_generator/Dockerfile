FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
COPY . /site_generator
WORKDIR /site_generator
RUN cargo build --release

FROM alpine
MAINTAINER longfangsong@icloud.com
RUN apk add npm && npm -g install typescript
COPY --from=builder /site_generator/target/release/site_generator /
WORKDIR /
ENV RUST_LOG info
ENTRYPOINT ["/site_generator"]
