CREATE TABLE IF NOT EXISTS pastes (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    paste TEXT NOT NULL,
    date INTEGER NOT NULL
)