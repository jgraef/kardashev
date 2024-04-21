-- returns current time in utc
CREATE OR REPLACE FUNCTION utc_now() RETURNS TIMESTAMPTZ AS $$
        BEGIN
                RETURN NOW() AT TIME ZONE 'utc';
        END;
$$ LANGUAGE plpgsql;



CREATE TYPE spherical_coordinates AS (
    radius DOUBLE PRECISION,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION
);

CREATE TYPE rga AS (
    red REAL,
    green REAL,
    blue REAL
);


CREATE TABLE users (
    user_id UUID NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    auth_secret TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    last_login TIMESTAMP NOT NULL,
    god_mode BOOLEAN NOT NULL DEFAULT FALSE,
    public_text TEXT
);

CREATE INDEX index_users_name ON users(name);


CREATE TABLE factions (
    faction_id UUID NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    public_text TEXT,
    private_text TEXT
);

CREATE TABLE faction_users (
    faction_id UUID NOT NULL REFERENCES factions(faction_id),
    user_id UUID NOT NULL REFERENCES users(user_id),
    role faction_user_role NOT NULL,
    UNIQUE (faction_id, user_id)
);

CREATE TYPE faction_user_role AS ENUM ('owner', 'member');

CREATE INDEX index_faction_users_by_faction_id ON faction_users(faction_id);
CREATE INDEX index_faction_users_by_user_id ON faction_users(user_id);
CREATE INDEX index_faction_users_by_faction_id_user_id ON faction_users(faction_id, user_id);

CREATE TABLE systems (
    system_id UUID NOT NULL PRIMARY KEY,
    position spherical_coordinates NOT NULL,
    magnitude REAL NOT NULL,
    t_eff REAL,
    sphere_of_influence DOUBLE PRECISION NOT NULL,
    root_node UUID NOT NULL REFERENCES system_nodes(node_id),
    created_at TIMESTAMP NOT NULL,
    last_updated BIGINT NOT NULL
);

CREATE INDEX index_systems_by_magnitude ON systems(magnitude);

CREATE TABLE system_nodes (
    node_id UUID NOT NULL PRIMARY KEY,
    system_id UUID NOT NULL REFERENCES systems(system_id),
    parent UUID REFERENCES system_nodes(node_id),
    type system_node_type NOT NULL,
    orbit orbital_elements,
    lagrange lagrange_elements
);

CREATE INDEX index_system_nodes_by_system_id ON system_nodes(system_id);

CREATE TYPE system_node_type AS ENUM ('barycenter', 'body', 'lagrange');

CREATE TYPE orbital_elements AS (
    eccentricity DOUBLE PRECISION,
    semi_major_axis DOUBLE PRECISION,
    inclination DOUBLE PRECISION,
    longitude_of_ascending_node DOUBLE PRECISION,
    argument_of_periapsis DOUBLE PRECISION,
    true_anomaly DOUBLE PRECISION,
    period BIGINT
);

CREATE TYPE lagrange_elements AS (
    id SMALLINT,
    orbital_displacement DOUBLE PRECISION,
    radial_displacement DOUBLE PRECISION
);

CREATE TABLE system_bodies (
    body_id UUID NOT NULL PRIMARY KEY,
    system_id UUID NOT NULL REFERENCES systems(system_id),
    node_id UUID NOT NULL REFERENCES system_nodes(node_id),
    sphere_of_influence DOUBLE PRECISION NOT NULL,
    type system_body_type NOT NULL,
    properties JSONB NOT NULL
);

CREATE TYPE system_body_type AS ENUM ('star', 'planet', 'moon', 'asteroid');

CREATE INDEX index_system_bodies_by_system_id ON system_bodies(system_id);
CREATE INDEX index_system_bodies_by_node_id ON system_bodies(node_id);


CREATE TABLE vessel (
    vessel_id UUID NOT NULL PRIMARY KEY,
    faction_id UUID NOT NULL REFERENCES factions(faction_id),
    
    name TEXT,
    public_text TEXT,
    private_text TEXT,

    interstellar_position spherical_coordinates,
    -- todo: interstellar velocity

    system_id UUID REFERENCES systems(system_id),
    system_node UUID REFERENCES system_nodes(node_id),
    system_orbit orbital_elements,

    -- todo: park in hangar

    -- todo: orientation, acceleration
    
    last_updated BIGINT NOT NULL
);

CREATE INDEX index_vessels_by_system_id ON vessels(system_id);
CREATE INDEX index_vessels_by_faction_id ON vessels(faction_id);
CREATE INDEX index_vessels_by_name ON vessels(name);

CREATE TABLE settlements (
    settlement_id UUID NOT NULL PRIMARY KEY,
    faction_id UUID NOT NULL REFERENCES factions(faction_id),

    name TEXT,
    public_text TEXT,
    private_text TEXT,

    body_id UUID REFERENCES system_bodies(body_id),

    last_updated BIGINT NOT NULL
);

CREATE INDEX index_settlements_by_system_id ON settlements(system_id);
CREATE INDEX index_settlements_by_faction_id ON settlements(faction_id);
CREATE INDEX index_settlements_by_name ON settlements(name);
