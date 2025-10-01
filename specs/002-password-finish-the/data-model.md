# Data Model: Complete Password Security Implementation

**Feature**: 002-password-finish-the
**Date**: 2025-10-01

## Overview

This document defines the data entities required for secure password management including password reset tokens, authentication logs, and rate limiting records. The model extends the existing `users` table and adds three new tables to support the security features.

## Entity Definitions

### 1. User (Extended)

**Purpose**: Existing entity extended with password security fields

**Schema Changes** (extend existing `users` table):

```sql
ALTER TABLE users
ADD COLUMN IF NOT EXISTS failed_login_attempts INT DEFAULT 0 NOT NULL,
ADD COLUMN IF NOT EXISTS locked_until TIMESTAMP NULL,
ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMP NULL,
ADD COLUMN IF NOT EXISTS last_login_ip VARCHAR(45) NULL;
```

**Fields**:
- `id` (INT, primary key) - Existing user identifier
- `email` (VARCHAR(255), unique, not null) - Existing
- `name` (VARCHAR(255), not null) - Existing
- `password_hash` (VARCHAR(255), nullable) - Existing (nullable for OAuth users)
- `created_at` (TIMESTAMP, not null) - Existing
- `updated_at` (TIMESTAMP, not null) - Existing
- `deleted_at` (TIMESTAMP, nullable) - Existing (soft delete)
- `failed_login_attempts` (INT, default 0) - **NEW**: Count of consecutive failed logins
- `locked_until` (TIMESTAMP, nullable) - **NEW**: Temporary lockout expiration (for future use)
- `last_login_at` (TIMESTAMP, nullable) - **NEW**: Last successful login timestamp
- `last_login_ip` (VARCHAR(45), nullable) - **NEW**: IP address of last successful login

**Relationships**:
- One user → Many password_reset_tokens
- One user → Many auth_logs

**Validation Rules**:
- `failed_login_attempts` must be >= 0
- `locked_until` must be in the future if set
- Reset `failed_login_attempts` to 0 on successful login

**State Transitions**:
```
Login Failed → failed_login_attempts++
Login Success → failed_login_attempts = 0, last_login_at = now(), last_login_ip = request_ip
Password Reset → failed_login_attempts = 0, locked_until = NULL
```

**Indexes**:
- Existing: PRIMARY KEY (id), UNIQUE (email)
- New: INDEX idx_users_locked_until ON users(locked_until) WHERE locked_until IS NOT NULL

---

### 2. PasswordResetToken (New Table)

**Purpose**: Store time-limited, single-use tokens for password reset flow

**Schema**:

```sql
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    used_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_token_expiry CHECK (expires_at > created_at)
);

CREATE INDEX idx_password_reset_tokens_token_hash ON password_reset_tokens(token_hash);
CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_expires_at ON password_reset_tokens(expires_at);
```

**Fields**:
- `id` (UUID, primary key) - Unique token record identifier
- `user_id` (INT, foreign key → users.id, NOT NULL) - Associated user
- `token_hash` (VARCHAR(255), NOT NULL, indexed) - Hashed token value (security: never store plaintext)
- `expires_at` (TIMESTAMP, NOT NULL, indexed) - Expiration time (15 minutes from creation)
- `used_at` (TIMESTAMP, NULL) - Timestamp when token was used (NULL = unused)
- `created_at` (TIMESTAMP, NOT NULL) - Creation timestamp

**Relationships**:
- Many tokens → One user (user_id foreign key)
- Cascade delete: If user deleted, all their tokens are deleted

**Validation Rules**:
- `expires_at` must be after `created_at` (CHECK constraint)
- `used_at` must be after `created_at` if set
- Token lifetime: Exactly 15 minutes (enforced at creation)
- Token reuse prevention: `used_at IS NOT NULL` means token is consumed

**State Transitions**:
```
Created (unused) → used_at = NULL, expires_at = now() + 15min
Used → used_at = NOW(), no further changes allowed
Expired → expires_at < NOW(), rejected even if unused
```

**Security Considerations**:
- Token value hashed with SHA-256 before storage (prevents token theft if DB compromised)
- Original token sent via email, never logged or stored plaintext
- Lookup by token_hash (constant-time comparison)
- Expired/used tokens rejected during validation

**Cleanup Strategy**:
- Periodic job: Delete tokens where `expires_at < NOW() - INTERVAL '24 hours'`
- Or: Lazy cleanup on each request (delete expired tokens for that user)

---

### 3. AuthLog (New Table)

**Purpose**: Audit log of authentication events for security monitoring and forensics

**Schema**:

```sql
CREATE TABLE auth_logs (
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

CREATE INDEX idx_auth_logs_user_id ON auth_logs(user_id);
CREATE INDEX idx_auth_logs_created_at ON auth_logs(created_at);
CREATE INDEX idx_auth_logs_event_type ON auth_logs(event_type);
CREATE INDEX idx_auth_logs_ip_address ON auth_logs(ip_address);
```

**Fields**:
- `id` (SERIAL, primary key) - Log entry identifier
- `user_id` (INT, foreign key → users.id, NULL) - Associated user (NULL if user not found)
- `event_type` (VARCHAR(50), NOT NULL) - Event type (see enum below)
- `success` (BOOLEAN, NOT NULL) - Whether event succeeded
- `ip_address` (VARCHAR(45), NOT NULL) - Client IP address (IPv4 or IPv6)
- `user_agent` (TEXT, NULL) - HTTP User-Agent header
- `email_attempted` (VARCHAR(255), NULL) - Email used in attempt (even if user not found)
- `error_message` (TEXT, NULL) - Error details (never include sensitive data)
- `created_at` (TIMESTAMP, NOT NULL, indexed) - Event timestamp

**Event Types** (event_type values):
- `login_success` - Successful login
- `login_failure` - Failed login (wrong password or nonexistent user)
- `login_rate_limited` - Login attempt blocked by rate limiting
- `password_reset_requested` - Password reset email sent
- `password_reset_success` - Password successfully reset via token
- `password_reset_failure` - Invalid/expired token used
- `password_changed` - Password changed while logged in
- `session_expired` - Session timed out

**Relationships**:
- Many logs → One user (user_id foreign key, SET NULL on delete to preserve audit trail)

**Validation Rules**:
- `event_type` must be one of the defined types
- `success = false` logs should have `error_message` populated
- Never log passwords, tokens, or sensitive data in `error_message`

**Retention Policy**:
- Retain logs for 90 days
- Periodic cleanup: `DELETE FROM auth_logs WHERE created_at < NOW() - INTERVAL '90 days'`

**Query Patterns**:
- Find failed login attempts for user: `WHERE user_id = ? AND event_type = 'login_failure' AND created_at > ? ORDER BY created_at DESC`
- Detect brute force by IP: `WHERE ip_address = ? AND event_type = 'login_failure' AND created_at > NOW() - INTERVAL '1 hour' GROUP BY ip_address HAVING COUNT(*) > 10`

---

### 4. RateLimitRecord (New Table)

**Purpose**: Track rate-limited actions to prevent brute force and abuse

**Schema**:

```sql
CREATE TABLE rate_limit_records (
    id SERIAL PRIMARY KEY,
    identifier VARCHAR(255) NOT NULL,
    action_type VARCHAR(50) NOT NULL,
    attempt_count INT DEFAULT 1 NOT NULL,
    window_start TIMESTAMP NOT NULL DEFAULT NOW(),
    next_allowed_at TIMESTAMP NULL,
    UNIQUE(identifier, action_type)
);

CREATE INDEX idx_rate_limit_identifier_action ON rate_limit_records(identifier, action_type);
CREATE INDEX idx_rate_limit_window_start ON rate_limit_records(window_start);
```

**Fields**:
- `id` (SERIAL, primary key) - Record identifier
- `identifier` (VARCHAR(255), NOT NULL) - IP address or email being rate-limited
- `action_type` (VARCHAR(50), NOT NULL) - Action being rate-limited (see enum below)
- `attempt_count` (INT, DEFAULT 1, NOT NULL) - Number of attempts in current window
- `window_start` (TIMESTAMP, NOT NULL, indexed) - Start of current rate limit window
- `next_allowed_at` (TIMESTAMP, NULL) - When next attempt is allowed (NULL = no delay)

**Action Types** (action_type values):
- `login` - Login attempts (per IP or email)
- `password_reset` - Password reset requests (per email)
- `registration` - Registration attempts (per IP)
- `password_change` - Password change requests (per user session)

**Relationships**:
- None (standalone table, indexed for fast lookups)

**Unique Constraint**:
- `UNIQUE(identifier, action_type)` - One record per (identifier, action) pair
- Use `ON CONFLICT (identifier, action_type) DO UPDATE` for upserts

**Validation Rules**:
- `attempt_count` must be > 0
- `next_allowed_at` must be in the future if set
- `window_start` must be <= NOW()

**Rate Limit Logic**:

```
1. Query: SELECT * FROM rate_limit_records WHERE identifier = ? AND action_type = ?
2. If not found OR (NOW() - window_start) > 1 hour:
   - Reset: INSERT/UPDATE with attempt_count = 1, window_start = NOW(), next_allowed_at = NULL
   - Allow request
3. Else if next_allowed_at IS NOT NULL AND NOW() < next_allowed_at:
   - Reject request with error "Too many attempts, try again in X seconds"
4. Else if attempt_count < threshold:
   - Increment: UPDATE attempt_count = attempt_count + 1
   - Allow request
5. Else (attempt_count >= threshold):
   - Calculate exponential backoff: delay = 2^(attempt_count - threshold) seconds
   - Update: next_allowed_at = NOW() + delay, attempt_count = attempt_count + 1
   - Reject request with error
```

**Thresholds** (per action_type):
- `login`: 5 attempts → start backoff (1s, 2s, 4s, 8s...)
- `password_reset`: 3 attempts per hour
- `registration`: 5 attempts per hour per IP
- `password_change`: 10 attempts per hour per session

**Cleanup Strategy**:
- Periodic job: `DELETE FROM rate_limit_records WHERE window_start < NOW() - INTERVAL '24 hours'`
- Or: Lazy cleanup on each request (delete old records during query)

---

## Database Migration

**File**: `migrations/YYYYMMDDHHMMSS_add_password_security.sql`

```sql
-- Extend users table
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
```

## Rust Model Definitions

**Struct Definitions** (to be created in `src/db/models/`):

```rust
// src/db/models/user.rs (extend existing)
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    // NEW fields
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
}

// src/db/models/password_reset_token.rs (NEW)
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: i32,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// src/db/models/auth_log.rs (NEW)
pub struct AuthLog {
    pub id: i32,
    pub user_id: Option<i32>,
    pub event_type: String,
    pub success: bool,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub email_attempted: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// src/db/models/rate_limit.rs (NEW)
pub struct RateLimitRecord {
    pub id: i32,
    pub identifier: String,
    pub action_type: String,
    pub attempt_count: i32,
    pub window_start: DateTime<Utc>,
    pub next_allowed_at: Option<DateTime<Utc>>,
}
```

## Summary

**Total Tables**: 4 (1 extended, 3 new)
**Total New Fields**: 4 (users table)
**Total Indexes**: 15 (3 existing + 12 new)

**Key Design Decisions**:
1. Token hashing: Store hashed tokens, not plaintext (defense in depth)
2. Soft references: auth_logs.user_id SET NULL on delete (preserve audit trail)
3. Cascade delete: password_reset_tokens deleted with user (no orphaned tokens)
4. Rate limiting in PostgreSQL: Simple, testable, sufficient for scale
5. Timestamp indexes: Enable efficient cleanup queries and forensic analysis

**Next Step**: Define API contracts in OpenAPI format based on these entities.
