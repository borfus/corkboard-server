CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE event (
    id uuid DEFAULT uuid_generate_v4(),
    last_modified_date TIMESTAMP,
    url VARCHAR,
    title VARCHAR,
    description VARCHAR,
    start_date DATE,
    end_date DATE,
    PRIMARY KEY (id)
)

