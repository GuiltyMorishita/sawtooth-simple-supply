CREATE TABLE IF NOT EXISTS agents (
    id               BIGSERIAL PRIMARY KEY,
    public_key       VARCHAR,
    name             VARCHAR,
    timestamp        BIGINT,
    start_block_num  BIGINT,
    end_block_num    BIGINT
);
