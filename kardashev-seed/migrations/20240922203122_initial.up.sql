CREATE TABLE star (
    id INTEGER NOT NULL PRIMARY KEY,
    position_x REAL NOT NULL,
    position_y REAL NOT NULL,
    position_z REAL NOT NULL,
    t_eff REAL NOT NULL,
    absolute_magnitude REAL NOT NULL,
    luminosity REAL NOT NULL,
    radius REAL NOT NULL,
    mass REAL NOT NULL,
    age REAL NOT NULL,
    type INTEGER,
    name TEXT
);
