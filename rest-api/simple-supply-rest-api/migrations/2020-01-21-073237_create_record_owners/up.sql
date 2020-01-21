CREATE TABLE IF NOT EXISTS record_owners (
    id               BIGSERIAL PRIMARY KEY,
    record_id        VARCHAR,
    agent_id         VARCHAR,
    timestamp        BIGINT,
    start_block_num  BIGINT,
    end_block_num    BIGINT
);
