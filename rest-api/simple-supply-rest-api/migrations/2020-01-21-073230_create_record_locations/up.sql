CREATE TABLE IF NOT EXISTS record_locations (
    id               BIGSERIAL PRIMARY KEY,
    record_id        VARCHAR,
    latitude         BIGINT,
    longitude        BIGINT,
    timestamp        BIGINT,
    start_block_num  BIGINT,
    end_block_num    BIGINT
);
