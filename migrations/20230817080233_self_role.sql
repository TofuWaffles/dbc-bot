-- Add migration script here

CREATE TABLE IF NOT EXISTS self_role_message (
    message_id BIGINT PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    ping_channel_id BIGINT NOT NULL
)