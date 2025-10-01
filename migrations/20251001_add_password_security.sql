-- Add password security features
-- Extends users table and adds three new tables for password reset, auth logging, and rate limiting

-- Extend users table with security fields
ALTER TABLE users
ADD COLUMN IF NOT EXISTS failed_login_attempts INT DEFAULT 0 NOT NULL,
ADD COLUMN IF NOT EXISTS locked_until TIMESTAMP NULL,
ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMP NULL,
ADD COLUMN IF NOT EXISTS last_login_ip VARCHAR(45) NULL;

CREATE INDEX IF NOT EXISTS idx_users_locked_until ON users(locked_until)
WHERE locked_until IS NOT NULL;

-- Password reset tokens table
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    used_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_token_expiry CHECK (expires_at > created_at)
);

CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_token_hash
ON password_reset_tokens(token_hash);

CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_id
ON password_reset_tokens(user_id);

CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_expires_at
ON password_reset_tokens(expires_at);

-- Authentication logs table
CREATE TABLE IF NOT EXISTS auth_logs (
    id SERIAL PRIMARY KEY,
    user_id INT NULL REFERENCES users(id) ON DELETE SET NULL,
    event_type VARCHAR(50) NOT NULL,
    success BOOLEAN NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    user_agent TEXT NULL,
    email_attempted VARCHAR(255) NULL,
    error_message TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_auth_logs_user_id ON auth_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_auth_logs_created_at ON auth_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_auth_logs_event_type ON auth_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_auth_logs_ip_address ON auth_logs(ip_address);

-- Rate limiting table
CREATE TABLE IF NOT EXISTS rate_limit_records (
    id SERIAL PRIMARY KEY,
    identifier VARCHAR(255) NOT NULL,
    action_type VARCHAR(50) NOT NULL,
    attempt_count INT DEFAULT 1 NOT NULL,
    window_start TIMESTAMP NOT NULL DEFAULT NOW(),
    next_allowed_at TIMESTAMP NULL,
    UNIQUE(identifier, action_type)
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_identifier_action
ON rate_limit_records(identifier, action_type);

CREATE INDEX IF NOT EXISTS idx_rate_limit_window_start
ON rate_limit_records(window_start);
