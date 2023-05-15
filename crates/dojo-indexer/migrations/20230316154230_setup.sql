CREATE TABLE indexer (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    head BIGINT DEFAULT 0
);

INSERT INTO indexer (head) VALUES (0);

CREATE TABLE components (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT,
    properties TEXT,
    address TEXT NOT NULL,
    class_hash TEXT NOT NULL,
    transaction_hash TEXT NOT NULL
);

CREATE TABLE system_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    data TEXT,
    transaction_hash TEXT NOT NULL,
    system_id TEXT NOT NULL,
    FOREIGN KEY (system_id) REFERENCES systems(id)
);

CREATE TABLE systems (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT,
    address TEXT NOT NULL,
    class_hash TEXT NOT NULL,
    transaction_hash TEXT NOT NULL
);

CREATE TABLE entities (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT,
    transaction_hash TEXT NOT NULL,
    partition_id TEXT NOT NULL,
    keys TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_entities_partition_id ON entities (partition_id);
CREATE INDEX idx_entities_keys ON entities (keys);

CREATE TABLE entity_state_updates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id TEXT NOT NULL,
    component_id TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    data TEXT,
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (component_id) REFERENCES components(id)
);
    
CREATE TABLE entity_states (
    entity_id TEXT NOT NULL,
    component_id TEXT NOT NULL,
    data TEXT,
    FOREIGN KEY (entity_id) REFERENCES entities(id),
    FOREIGN KEY (component_id) REFERENCES components(id),
    UNIQUE (entity_id, component_id)
);

CREATE TABLE events (
    id TEXT NOT NULL PRIMARY KEY,
    system_call_id INTEGER NOT NULL,
    keys TEXT NOT NULL,
    data TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (system_call_id) REFERENCES system_calls(id)
);

CREATE INDEX idx_events_keys ON events (keys);
