# FROM messense/rust-musl-cross:x86_64-musl AS chef
# ENV DISCORD_TOKEN = ${DISCORD_TOKEN}
# ENV BRAWL_STARS_TOKEN = ${BRAWL_STARS_TOKEN}
# ENV DATABASE_URL = ${DATABASE_URL}
# WORKDIR /dbc-bot

# FROM chef AS planner
# # Copy source code from previous stage
# COPY . .
# # Generate info for caching dependencies
# RUN cargo chef prepare --recipe-path recipe.json

# # Create a new stage with a minimal image
# FROM chef AS builder
# COPY --from=planner /api-deployment-example/recipe.json recipe.json
# # Build & cache dependencies
# RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
# # Copy source code from previous stage
# COPY . .
# # Build application
# RUN cargo build --release --target x86_64-unknown-linux-musl
# ENTRYPOINT [ "/dbc-bot"]

FROM messense/rust-musl-cross:x86_64-musl AS builder
ENV DISCORD_TOKEN = ${DISCORD_TOKEN}
ENV BRAWL_STARS_TOKEN = ${BRAWL_STARS_TOKEN}
ENV DATABASE_URL = ${DATABASE_URL}
WORKDIR /dbc-bot

COPY . .
RUN apt-get update && apt-get install musl-tools pkg-config libssl-dev -y
RUN export OPENSSL_DIR="/usr/lib/openssl"
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /dbc-bot/target/x86_64-unknown-linux-musl/release/dbc-bot /dbc-bot
ENTRYPOINT ["/dbc-bot"]
# Temp add
RUN docker login -u ${DOCKER_USERNAME} -p ${DOCKER_PASSWORD} 




