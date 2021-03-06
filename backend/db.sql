DROP TABLE IF EXISTS messages;

CREATE TABLE messages (
  id VARCHAR(32) PRIMARY KEY,
  from_address VARCHAR NOT NULL,
  to_address VARCHAR NOT NULL,
  subject VARCHAR NOT NULL,
  signature VARCHAR,
  created DATE DEFAULT CURRENT_DATE
);
