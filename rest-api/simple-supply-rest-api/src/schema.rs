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

allow_tables_to_appear_in_same_query!(
    agents,
    auths,
);
