-- Add migration script here
CREATE TABLE learning_records
(
    id               UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    user_id          UUID        NOT NULL,
    title            TEXT        NOT NULL,
    content          TEXT,
    duration_minutes INT4        NOT NULL DEFAULT 0,
    subject          TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_learning_records_user
        FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE INDEX idx_learning_records_user_id ON learning_records (user_id);
CREATE INDEX idx_learning_records_user_created ON learning_records (user_id, created_at DESC);
