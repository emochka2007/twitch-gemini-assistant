CREATE OR REPLACE FUNCTION update_timestamp()
    RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE extension if not exists "uuid-ossp";
CREATE table if not exists chat_messages
(
    id         uuid primary key                     default uuid_generate_v4(),
    username   VARCHAR(256)                not null,
    text       text                        not null,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT current_timestamp
);

CREATE OR REPLACE TRIGGER set_timestamp_for_chat_messages
    BEFORE UPDATE
    ON chat_messages
    FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

CREATE TABLE IF NOT EXISTS chaos_command_execution
(
    id         UUID PRIMARY KEY                     DEFAULT uuid_generate_v4(),
    origin_id  UUID                        NOT NULL,
    link       TEXT,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT current_timestamp
);

CREATE OR REPLACE TRIGGER set_timestamp_for_chaos_command_execution
    BEFORE UPDATE
    ON chaos_command_execution
    FOR EACH ROW
EXECUTE FUNCTION update_timestamp();
