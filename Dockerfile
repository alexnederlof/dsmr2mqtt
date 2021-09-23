# We use the latest Rust stable release as base image
FROM rustlang/rust:nightly-slim AS builder
# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app

# Run deps first. If nothing changes, this step will be cached by docker
COPY Cargo.* .
RUN cargo fetch

# Copy all files from our working environment to our Docker image
COPY . .
# Let's build our binary!
# We'll use the release profile to make it faaaast
RUN cargo build --release

FROM debian:buster-slim AS runtime
WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
# RUN apt-get update -y \
#     && apt-get install -y --no-install-recommends openssl \
#     # Clean up
#     && apt-get autoremove -y \
#     && apt-get clean -y \
#     && rm -rf /var/lib/apt/lists/*

# ENV APP_LISTEN=0.0.0.0:8080
# ENV APP_ENVIRONMENT production


COPY --from=builder /app/target/release/dsmr dsmr
ENTRYPOINT ["./dsmr"]