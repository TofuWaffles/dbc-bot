FROM messense/rust-musl-cross:x86_64-musl AS builder
ENV DISCORD_TOKEN = ${DISCORD_TOKEN}
ENV BRAWL_STARS_TOKEN = ${BRAWL_STARS_TOKEN}
ENV DATABASE_URL = ${DATABASE_URL}
WORKDIR /dbc-bot
# Copy the source code
COPY . .
# Build the application
RUN cargo build --release --target=x86_64-unknown-linux-musl
# Create a new stage with a minimal image
FROM scratch
COPY --from=builder /dbc-bot/target/x86_64-unknown-linux-musl/release/dbc-bot /dbc-bot
ENTRYPOINT [ "/dbc-bot"]


