-- Your SQL goes here
CREATE TABLE user_delivery_address (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users ON DELETE CASCADE,
    administrative_area_level_1 VARCHAR,
    administrative_area_level_2 VARCHAR,
    country VARCHAR NOT NULL,
    locality VARCHAR,
    political VARCHAR,
    postal_code VARCHAR NOT NULL,
    route VARCHAR,
    street_number VARCHAR,
    address VARCHAR,
    is_priority BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX user_delivery_address_id_idx ON user_delivery_address (id);

SELECT diesel_manage_updated_at('user_delivery_address');
