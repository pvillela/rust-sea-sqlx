CREATE TABLE users (
                       id SERIAL PRIMARY KEY,
                       name VARCHAR(100) NOT NULL,
                       email VARCHAR(100) NOT NULL UNIQUE,
                       age INT,
                       created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
