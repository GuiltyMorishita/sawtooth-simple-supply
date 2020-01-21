CREATE TABLE IF NOT EXISTS records (
    id               BIGSERIAL PRIMARY KEY,
    record_id        VARCHAR,
    start_block_num  BIGINT,
    end_block_num    BIGINT
);
