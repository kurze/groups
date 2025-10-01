# Database Schema Analysis Report

## Executive Summary

The current database implementation provides a basic foundation for password management but is missing most of the advanced security features outlined in the password management plan. While basic authentication is functional, significant database schema enhancements are needed to support enterprise-grade password management.

## Current Database Schema

### Existing Tables

#### 1. Users Table (Implemented)
```sql
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL DEFAULT '',
    password_hash VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE
);
```

**Indexes:**
- `idx_users_email` - Performance optimization for login lookups
- `idx_users_deleted_at` - Soft delete performance

#### 2. Groups Table (Implemented)
```sql
CREATE TABLE IF NOT EXISTS groups (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE
);
```

#### 3. Database Extensions (Implemented)
```sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "citext";
SET timezone = 'UTC';
```

## Current Implementation Status

### ✅ What's Already Implemented

1. **Basic Password Storage**
   - `password_hash` field exists in users table
   - Argon2 password hashing implemented
   - Password verification working
   - Optional password hash (supports legacy users)

2. **User Management**
   - User creation and retrieval by email/ID
   - Soft delete functionality
   - Basic user model with timestamps

3. **Authentication Flow**
   - Login/logout endpoints
   - Registration with password hashing
   - Session-based authentication
   - Authentication middleware

4. **Database Infrastructure**
   - PostgreSQL with proper extensions
   - Connection pooling via SQLx
   - Migration system in place
   - Proper indexing for performance

### ❌ What's Missing for Password Management Plan

#### Critical Missing Database Fields (Phase 1)
```sql
-- Required additions to users table:
ALTER TABLE users ADD COLUMN password_changed_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE users ADD COLUMN password_expires_at TIMESTAMP WITH TIME ZONE NULL;
ALTER TABLE users ADD COLUMN failed_login_attempts INT DEFAULT 0;
ALTER TABLE users ADD COLUMN locked_until TIMESTAMP WITH TIME ZONE NULL;
ALTER TABLE users ADD COLUMN last_login_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE users ADD COLUMN last_login_ip VARCHAR(45);
ALTER TABLE users ADD COLUMN last_login_device TEXT;
ALTER TABLE users ADD COLUMN created_ip VARCHAR(45);
ALTER TABLE users ADD COLUMN password_history TEXT; -- JSON array of previous hashes
ALTER TABLE users ADD COLUMN risk_score INT DEFAULT 0;
```

#### Missing Tables (Phase 1 & 2)

1. **Sessions Table (Critical)**
```sql
CREATE TABLE sessions (
    id VARCHAR(64) PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    device_fingerprint VARCHAR(64),
    is_remembered BOOLEAN DEFAULT FALSE,
    last_activity TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
```

2. **Authentication Logs Table (High Priority)**
```sql
CREATE TABLE auth_logs (
    id SERIAL PRIMARY KEY,
    user_id INT NULL REFERENCES users(id) ON DELETE SET NULL,
    event_type VARCHAR(50) NOT NULL, -- login, logout, register, password_change, etc.
    success BOOLEAN NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    error_message TEXT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_auth_logs_user_id ON auth_logs(user_id);
CREATE INDEX idx_auth_logs_created_at ON auth_logs(created_at);
CREATE INDEX idx_auth_logs_event_type ON auth_logs(event_type);
```

3. **Login Attempts Table (Brute Force Protection)**
```sql
CREATE TABLE login_attempts (
    id SERIAL PRIMARY KEY,
    identifier VARCHAR(255) NOT NULL, -- email or username
    ip_address VARCHAR(45) NOT NULL,
    attempted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    success BOOLEAN NOT NULL,
    user_agent TEXT
);

CREATE INDEX idx_login_attempts_identifier ON login_attempts(identifier);
CREATE INDEX idx_login_attempts_ip ON login_attempts(ip_address);
CREATE INDEX idx_login_attempts_attempted_at ON login_attempts(attempted_at);
```

#### Advanced Security Tables (Phase 2 & 3)

4. **Password Reset Tokens**
```sql
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    used_at TIMESTAMP WITH TIME ZONE NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(45),
    user_agent TEXT
);

CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_expires_at ON password_reset_tokens(expires_at);
```

5. **MFA Settings (Phase 3)**
```sql
CREATE TABLE user_mfa (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mfa_type VARCHAR(20) NOT NULL, -- totp, sms, email, webauthn
    secret_encrypted TEXT, -- Encrypted TOTP secret or device key
    backup_codes_encrypted TEXT, -- Encrypted JSON array
    is_active BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(user_id, mfa_type)
);
```

6. **Device Tracking (Phase 2)**
```sql
CREATE TABLE user_devices (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_fingerprint VARCHAR(64) NOT NULL,
    device_name VARCHAR(255),
    first_seen_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    is_trusted BOOLEAN DEFAULT FALSE,
    ip_address VARCHAR(45),
    user_agent TEXT,
    location_country VARCHAR(2),
    location_city VARCHAR(100)
);

CREATE INDEX idx_user_devices_user_id ON user_devices(user_id);
CREATE INDEX idx_user_devices_fingerprint ON user_devices(device_fingerprint);
```

## Schema Migration Strategy

### Phase 1: Core Security (Immediate - Next 2 weeks)
1. Add missing fields to users table
2. Create sessions table
3. Create auth_logs table  
4. Create login_attempts table
5. Update User model to include new fields
6. Implement proper session management

### Phase 2: Advanced Authentication (Month 2)
1. Add password_reset_tokens table
2. Add user_devices table
3. Implement device fingerprinting
4. Add risk-based authentication

### Phase 3: MFA & Enterprise (Month 3+)
1. Add user_mfa table
2. Implement TOTP support
3. Add WebAuthn tables if needed
4. Enterprise SSO tables

## Database Model Updates Required

### Current User Model Issues
```rust
// Current - Missing many security fields
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
```

### Required User Model Enhancement
```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromRow)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub password_hash: Option<String>,
    pub password_changed_at: Option<DateTime<Utc>>,
    pub password_expires_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub last_login_device: Option<String>,
    pub created_ip: Option<String>,
    pub password_history: Option<String>, // JSON
    pub risk_score: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
```

## Performance Considerations

### Current Indexing Status: ✅ Good
- Email lookup optimized
- Soft delete queries optimized
- Primary keys properly defined

### Additional Indexes Needed
```sql
-- For session cleanup
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

-- For brute force protection
CREATE INDEX idx_login_attempts_ip_time ON login_attempts(ip_address, attempted_at);

-- For audit queries
CREATE INDEX idx_auth_logs_user_time ON auth_logs(user_id, created_at);

-- For device tracking
CREATE INDEX idx_user_devices_user_last_seen ON user_devices(user_id, last_seen_at);
```

## Security Gaps Analysis

### Critical Security Issues (Immediate Fix Required)
1. **No Session Persistence** - Sessions only exist in memory
2. **No Brute Force Protection** - Unlimited login attempts
3. **No Audit Trail** - No logging of authentication events
4. **No Account Lockout** - Failed attempts not tracked
5. **Basic Argon2 Config** - Using defaults instead of OWASP 2024 recommendations

### High Priority Issues (Fix within 1 month)
1. **No Password History** - Users can reuse passwords
2. **No Device Tracking** - Can't detect unusual login locations
3. **No Password Expiry** - No forced password rotation
4. **No Rate Limiting** - IP-based attacks possible

### Medium Priority Issues (Fix within 3 months)
1. **No MFA Support** - Single factor authentication only
2. **No Password Strength Policy** - Accepts weak passwords
3. **No Breach Detection** - No HaveIBeenPwned integration
4. **No Risk Scoring** - No adaptive authentication

## Recommendations

### Immediate Actions (Week 1-2)
1. **Create comprehensive migration** for Phase 1 database changes
2. **Implement session table** and persistent session management
3. **Add authentication logging** for security monitoring
4. **Update User model** to include security fields
5. **Implement basic rate limiting** using login_attempts table

### Short-term Actions (Month 1)
1. **Add account lockout logic** with exponential backoff
2. **Implement password history** to prevent reuse
3. **Add device fingerprinting** for security
4. **Optimize Argon2 parameters** per OWASP 2024 guidelines
5. **Create password reset** secure flow

### Medium-term Actions (Month 2-3)
1. **Implement TOTP MFA** with backup codes
2. **Add password strength validation** with zxcvbn
3. **Integrate breach detection** with HaveIBeenPwned
4. **Implement risk-based authentication**

## Migration Files Needed

1. `003_add_user_security_fields.sql` - Add missing user table fields
2. `004_create_sessions_table.sql` - Persistent session management
3. `005_create_auth_logging.sql` - Authentication audit trails
4. `006_create_login_attempts.sql` - Brute force protection
5. `007_create_password_reset.sql` - Secure password reset
6. `008_create_device_tracking.sql` - Device fingerprinting
7. `009_create_mfa_support.sql` - Multi-factor authentication

## Conclusion

The current database schema provides a solid foundation but requires significant enhancement to meet the password management plan requirements. The implementation is currently at approximately **20%** of the full plan scope. Critical security features like persistent sessions, brute force protection, and audit logging should be prioritized for immediate implementation.

The good news is that the existing architecture (PostgreSQL, SQLx, Argon2) is well-suited for these enhancements, and the migration system is already in place to support incremental rollout of security features.