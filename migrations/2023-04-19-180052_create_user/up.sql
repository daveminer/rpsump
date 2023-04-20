CREATE TABLE user
(
  id INTEGER PRIMARY KEY,
  password_hash TEXT NOT NULL,
  email TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_on_email ON user (email);
CREATE INDEX idx_user_on_created_at ON user (created_at);
