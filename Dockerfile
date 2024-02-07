ARG RUST_VERSION=1.74.0
ARG Python_VERSION=3.9.7
ARG APP_NAME=dbc-bot

FROM python:${Python_VERSION}-alpine AS python
COPY /src/bracket_tournament/bracket_generation.py /src/bracket_tournament/bracket_generation.py
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

################################################################################
# Create a stage for building the Rust application.
FROM messense/rust-musl-cross:x86_64-musl as builder
ARG APP_NAME
WORKDIR /dbc-bot

COPY . .
COPY --from=python /src/bracket_tournament/bracket_generation.py /src/bracket_tournament/bracket_generation.py
RUN cargo build --release

################################################################################
# Create a new stage for running the application.
FROM alpine:3.18 AS final

# Create a non-privileged user
ARG UID=10001
RUN adduser -D -u ${UID} appuser

# Set environment variables
ENV DISCORD_TOKEN=${DISCORD_TOKEN} \
    BRAWL_STARS_TOKEN=${BRAWL_STARS_TOKEN} \
    DATABASE_URL=${DATABASE_URL}

# Copy the Rust executable
COPY --from=rust /dbc-bot/target/release/${APP_NAME} /bin/${APP_NAME}

# Switch to non-privileged user
USER appuser

# What the container should run when it is started.
CMD ["/bin/dbc-bot"]
