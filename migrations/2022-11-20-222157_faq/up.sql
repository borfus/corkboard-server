CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE faq (
    id uuid DEFAULT uuid_generate_v4(),
    last_modified_date TIMESTAMP,
    question VARCHAR,
    answer VARCHAR,
    PRIMARY KEY (id)
)

