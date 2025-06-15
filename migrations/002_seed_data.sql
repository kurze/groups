-- Seed data for development
INSERT INTO users (email, name, password_hash) VALUES 
('alice@wonderla.nd', 'Alice', NULL),
('admin@groups.dev', 'Admin User', '$argon2id$v=19$m=65536,t=3,p=4$example$hash')
ON CONFLICT (email) DO NOTHING;

INSERT INTO groups (name) VALUES 
('Rust Developers'),
('Web Developers'),
('Database Enthusiasts');