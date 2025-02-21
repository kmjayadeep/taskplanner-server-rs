FROM rust:1.83-slim AS builder

# create a new empty shell project
RUN USER=root cargo new --bin taskplanner-server
WORKDIR /taskplanner-server

# curl is needed to download swagger
RUN apt-get update && apt-get install -y curl

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./.sqlx ./.sqlx
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/taskplanner_server*
RUN cargo build --release

# our final base
FROM ubuntu:noble

# copy the build artifact from the build stage
COPY --from=builder /taskplanner-server/target/release/taskplanner-server .

# set the startup command to run your binary
CMD ["./taskplanner-server"]

EXPOSE 8000
