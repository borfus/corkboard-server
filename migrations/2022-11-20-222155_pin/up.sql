CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE pin (
    id uuid DEFAULT uuid_generate_v4(),
    last_modified_date TIMESTAMP,
    url VARCHAR,
    title VARCHAR,
    description VARCHAR,
    PRIMARY KEY (id)
)

