CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE event (
    id uuid DEFAULT uuid_generate_v4(),
    last_modified_date TIMESTAMP NOT NULL default now(),
    url VARCHAR,
    title VARCHAR,
    description VARCHAR,
    start_date TIMESTAMP,
    end_date TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE pin (
    id uuid DEFAULT uuid_generate_v4(),
    last_modified_date TIMESTAMP NOT NULL default now(),
    url VARCHAR,
    title VARCHAR,
    description VARCHAR,
    PRIMARY KEY (id)
);

CREATE TABLE faq (
    id uuid DEFAULT uuid_generate_v4(),
    last_modified_date TIMESTAMP NOT NULL default now(),
    question VARCHAR,
    answer VARCHAR,
    PRIMARY KEY (id)
);

