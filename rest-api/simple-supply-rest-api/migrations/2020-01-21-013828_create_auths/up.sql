CREATE TABLE IF NOT EXISTS auths (
    public_key            VARCHAR PRIMARY KEY,
    hashed_password       VARCHAR,
    encrypted_private_key VARCHAR
)
