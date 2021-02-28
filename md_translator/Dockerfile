FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
COPY . /md_translator
WORKDIR /md_translator
RUN cargo build --release

FROM alpine
MAINTAINER longfangsong@icloud.com
COPY --from=builder /md_translator/target/release/md_translator /
WORKDIR /
ENTRYPOINT ["/md_translator"]
