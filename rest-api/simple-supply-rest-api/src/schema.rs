table! {
    agents (id) {
        id -> Int8,
        public_key -> Nullable<Varchar>,
        name -> Nullable<Varchar>,
        timestamp -> Nullable<Int8>,
        start_block_num -> Nullable<Int8>,
        end_block_num -> Nullable<Int8>,
    }
}

table! {
    auths (public_key) {
        public_key -> Varchar,
        hashed_password -> Nullable<Varchar>,
        encrypted_private_key -> Nullable<Varchar>,
    }
}

table! {
    blocks (block_num) {
        block_num -> Int8,
        block_id -> Nullable<Varchar>,
    }
}

table! {
    record_locations (id) {
        id -> Int8,
        record_id -> Nullable<Varchar>,
        latitude -> Nullable<Int8>,
        longitude -> Nullable<Int8>,
        timestamp -> Nullable<Int8>,
        start_block_num -> Nullable<Int8>,
        end_block_num -> Nullable<Int8>,
    }
}

table! {
    record_owners (id) {
        id -> Int8,
        record_id -> Nullable<Varchar>,
        agent_id -> Nullable<Varchar>,
        timestamp -> Nullable<Int8>,
        start_block_num -> Nullable<Int8>,
        end_block_num -> Nullable<Int8>,
    }
}

table! {
    records (id) {
        id -> Int8,
        record_id -> Nullable<Varchar>,
        start_block_num -> Nullable<Int8>,
        end_block_num -> Nullable<Int8>,
    }
}

allow_tables_to_appear_in_same_query!(
    agents,
    auths,
    blocks,
    record_locations,
    record_owners,
    records,
);
