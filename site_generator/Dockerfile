FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
# If we are lucky enough, these will build the dependencies and cache the git layer
COPY ./Cargo.toml /site_generator/
RUN mkdir /site_generator/src && echo "fn main() {}" > /site_generator/src/main.rs
WORKDIR /site_generator
RUN cargo build --release
# Do the real build
COPY ./* /site_generator/
RUN cargo build --release

FROM alpine
MAINTAINER longfangsong@icloud.com
RUN apk add npm && npm -g install typescript
COPY --from=builder /site_generator/target/release/site_generator /
WORKDIR /
ENTRYPOINT ["/site_generator"]
