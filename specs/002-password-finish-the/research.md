# Research: Complete Password Security Implementation

**Feature**: 002-password-finish-the
**Date**: 2025-10-01

## Research Findings

### 1. Argon2 OWASP 2024 Configuration

**Decision**: Configure Argon2id with 47MB memory (48,000 KB), 3 iterations, parallelism of 1

**Rationale**:
- OWASP 2024 Password Storage Cheat Sheet recommends these parameters for password hashing
- Argon2id (hybrid mode) provides best balance between side-channel resistance (Argon2i) and brute-force resistance (Argon2d)
- 47MB memory makes GPU-based attacks economically infeasible while remaining practical for server hardware
- 3 iterations provide additional security margin
- Parallelism of 1 ensures consistent performance across different CPU architectures

**Alternatives Considered**:
- bcrypt: Well-established but less memory-hard, more vulnerable to GPU attacks
- scrypt: Good memory-hardness but less well-studied than Argon2
- PBKDF2: Deprecated for password hashing, insufficient resistance to modern attacks
- Argon2i only: More resistant to side-channel but weaker against GPU attacks
- Argon2d only: Stronger against GPU but vulnerable to side-channel attacks

**Implementation Notes**:
- Use `argon2` crate (already in Cargo.toml at version 0.5.3)
- Configure via `argon2::Params::new(48_000, 3, 1, None)`
- Benchmark on target hardware to verify <1s hash time
- Store configuration in constants for easy tuning

**References**:
- OWASP Password Storage Cheat Sheet 2024
- RFC 9106: Argon2 Memory-Hard Function for Password Hashing
- Argon2 crate documentation: https://docs.rs/argon2/latest/argon2/

---

### 2. zxcvbn Password Strength Estimation

**Decision**: Use `zxcvbn` Rust crate for realistic password strength validation

**Rationale**:
- Industry-standard algorithm developed by Dropbox, widely adopted
- Evaluates actual password strength through pattern matching, dictionary attacks, common sequences
- Provides actionable feedback (e.g., "avoid dates", "add more unique characters") rather than arbitrary rules
- Pure Rust implementation, no external services or network calls required
- Returns strength score (0-4) and estimated crack time
- Supports user-specific inputs (email, name) to detect weak passwords containing personal info

**Alternatives Considered**:
- Arbitrary rules (12+ chars, uppercase, number, symbol): Users create weak passwords that technically comply (e.g., "Password123!")
- Entropy calculation only: Doesn't catch dictionary words or patterns
- External API (haveibeenpwned): Adds network dependency, latency, privacy concerns
- passwor d-strength crate: Less mature, fewer features than zxcvbn

**Implementation Notes**:
- Add `zxcvbn = "2"` to Cargo.toml
- Minimum acceptable score: 3 out of 4 (score 4 is "very strong", rarely achieved)
- Pass user email as user input to detect passwords containing email parts
- Display feedback.warning and feedback.suggestions to user
- Client-side validation for UX, server-side for security

**Example Usage**:
```rust
use zxcvbn::zxcvbn;

let entropy = zxcvbn("password123", &["user@example.com"]).unwrap();
// entropy.score() -> 0 (very weak)
// entropy.feedback() -> Suggestions to improve

let entropy = zxcvbn("correct-horse-battery-staple", &[]).unwrap();
// entropy.score() -> 4 (very strong)
```

**References**:
- zxcvbn paper: https://dropbox.tech/security/zxcvbn-realistic-password-strength-estimation
- zxcvbn Rust crate: https://docs.rs/zxcvbn/latest/zxcvbn/

---

### 3. Constant-Time Comparison for Timing Attack Prevention

**Decision**: Use `subtle` crate for constant-time comparisons

**Rationale**:
- Timing attacks can reveal which users exist by measuring response time differences
- Standard `==` operator short-circuits on first difference, creating timing side-channel
- `subtle::ConstantTimeEq` trait ensures all bytes are compared regardless of early mismatches
- Critical for: password verification, token validation, checking if email exists
- Zero-cost abstraction—compiled down to efficient constant-time code

**Alternatives Considered**:
- Manual constant-time implementation: Error-prone, compiler optimizations may break it
- Always sleep for fixed duration: Masks timing but adds unnecessary latency
- Compare hash outputs only: Still vulnerable if hash comparison isn't constant-time

**Implementation Notes**:
- Add `subtle = "2.6"` to Cargo.toml
- Use `ConstantTimeEq::ct_eq()` for all security-sensitive comparisons
- Apply to: password hash comparison, reset token validation, user existence checks
- Combine with equal-duration database queries (see rate limiting section)

**Example Usage**:
```rust
use subtle::ConstantTimeEq;

let hash1: &[u8] = stored_password_hash.as_bytes();
let hash2: &[u8] = computed_password_hash.as_bytes();

// Constant-time comparison
if hash1.ct_eq(hash2).into() {
    // Password correct
}
```

**References**:
- subtle crate documentation: https://docs.rs/subtle/latest/subtle/
- Timing attack overview: https://en.wikipedia.org/wiki/Timing_attack

---

### 4. Rate Limiting Without Redis (Constitutional Principle IV)

**Decision**: Implement rate limiting using PostgreSQL table with optimized indexes

**Rationale**:
- Constitution Principle IV: No caching layers unless measurements prove necessity
- PostgreSQL sufficient for single-instance deployment (target scale: hundreds of users)
- Simpler to implement, test, and maintain than distributed cache
- Leverages existing infrastructure—no new services to deploy
- Indexes on (identifier, action_type) and timestamp make queries fast enough

**Alternatives Considered**:
- Redis: Adds infrastructure complexity, requires separate service, increases failure modes
- In-memory HashMap: Lost on restart, not suitable for rate limiting (attackers just restart process)
- actix-web-governor: Uses in-memory state, same issues as HashMap
- No rate limiting: Unacceptable security risk

**Implementation Notes**:
```sql
CREATE TABLE rate_limit_records (
    id SERIAL PRIMARY KEY,
    identifier VARCHAR(255) NOT NULL,  -- IP address or email
    action_type VARCHAR(50) NOT NULL,  -- 'login', 'password_reset', 'registration'
    attempt_count INT DEFAULT 1,
    window_start TIMESTAMP NOT NULL,
    next_allowed_at TIMESTAMP,
    UNIQUE(identifier, action_type)
);

CREATE INDEX idx_rate_limit_identifier ON rate_limit_records(identifier, action_type);
CREATE INDEX idx_rate_limit_timestamp ON rate_limit_records(window_start);
```

**Rate Limit Logic**:
1. On each attempt, query: `SELECT * FROM rate_limit_records WHERE identifier = ? AND action_type = ?`
2. If not found or window expired: Create/reset record, allow request
3. If within window and under limit: Increment attempt_count, allow request
4. If over limit: Calculate exponential backoff delay, set next_allowed_at, reject request
5. Periodic cleanup: Delete records older than 1 hour (lazy cleanup on each check)

**Performance Considerations**:
- Index on (identifier, action_type) makes lookups O(log n)
- Typical query time: <5ms even with thousands of records
- Exponential backoff delays: 1s, 2s, 4s, 8s (prevent brute force without permanent lockout)

**References**:
- Constitution v1.0.0, Principle IV: Performance Through Simplicity
- PostgreSQL performance tuning: https://www.postgresql.org/docs/current/performance-tips.html

---

### 5. Email Sending with Lettre

**Decision**: Use `lettre` crate with SMTP backend for email notifications

**Rationale**:
- Established Rust email library, actively maintained
- Supports async/await (works with Tokio runtime)
- Flexible transport backends (SMTP, sendmail, file)
- No external API dependencies—standard SMTP protocol
- Configurable via environment variables for different environments (dev, staging, prod)

**Alternatives Considered**:
- sendgrid-rs/mailgun-rs: Vendor lock-in, requires API keys, adds external dependency
- tokio-smtp: Lower-level, more complex to use
- Manual SMTP implementation: Reinventing the wheel, error-prone
- Mock email in dev: Doesn't test actual email delivery

**Implementation Notes**:
- Add dependencies:
  ```toml
  lettre = { version = "0.11", features = ["tokio1-native-tls", "smtp-transport"] }
  ```
- Environment variables:
  ```
  SMTP_HOST=smtp.example.com
  SMTP_PORT=587
  SMTP_USERNAME=noreply@groups.com
  SMTP_PASSWORD=secret
  SMTP_FROM_EMAIL=noreply@groups.com
  SMTP_FROM_NAME=Groups Platform
  ```
- Dev environment: Use Mailhog or MailCatcher for local testing (Docker container on port 1025)
- Async sending to avoid blocking request threads
- Retry logic: Simple exponential backoff, 3 attempts max
- Template emails with Tera (reuse existing template engine)

**Email Templates**:
1. Password reset request: "Click this link to reset your password (expires in 15 min)"
2. Password reset success: "Your password was successfully changed"
3. Password change notification: "Your password was changed from IP x.x.x.x"

**Example Usage**:
```rust
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

async fn send_password_reset_email(to: &str, token: &str) -> Result<()> {
    let email = Message::builder()
        .from(env::var("SMTP_FROM_EMAIL")?.parse()?)
        .to(to.parse()?)
        .subject("Password Reset Request")
        .body(format!("Reset link: https://groups.com/password-reset/confirm?token={}", token))?;

    let creds = Credentials::new(
        env::var("SMTP_USERNAME")?,
        env::var("SMTP_PASSWORD")?
    );

    let mailer = SmtpTransport::relay(&env::var("SMTP_HOST")?)?
        .credentials(creds)
        .build();

    tokio::task::spawn_blocking(move || mailer.send(&email)).await??;
    Ok(())
}
```

**References**:
- lettre documentation: https://docs.rs/lettre/latest/lettre/
- Mailhog (dev SMTP server): https://github.com/mailhog/MailHog

---

### 6. Session Timeout Implementation

**Decision**: Store last_activity timestamp in session, validate on each request

**Rationale**:
- Works with existing actix-session cookie-based storage
- No database queries needed for timeout check (fast)
- Idle timeout (30 min) and absolute timeout (12 hours) both enforceable
- Middleware checks session validity on every protected route

**Alternatives Considered**:
- Database-backed sessions: Adds query overhead, violates simplicity principle
- JWT tokens: Stateless but can't be invalidated before expiry (security issue)
- Redis sessions: Requires Redis infrastructure, violates constitution

**Implementation Notes**:
```rust
// Session structure
struct SessionData {
    user_id: i32,
    user_email: String,
    created_at: DateTime<Utc>,   // For absolute timeout
    last_activity: DateTime<Utc>, // For idle timeout
}

// Middleware check
async fn check_session_timeout(session: Session) -> Result<(), AuthError> {
    let last_activity: DateTime<Utc> = session.get("last_activity")?.unwrap_or_default();
    let created_at: DateTime<Utc> = session.get("created_at")?.unwrap_or_default();
    let now = Utc::now();

    // Idle timeout: 30 minutes
    if now.signed_duration_since(last_activity).num_minutes() > 30 {
        session.purge();
        return Err(AuthError::SessionExpired);
    }

    // Absolute timeout: 12 hours
    if now.signed_duration_since(created_at).num_hours() > 12 {
        session.purge();
        return Err(AuthError::SessionExpired);
    }

    // Update last activity
    session.insert("last_activity", now)?;
    Ok(())
}
```

**Session Cookie Configuration**:
```rust
SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
    .cookie_name("groups_session")
    .cookie_secure(is_production)  // HTTPS only in prod
    .cookie_http_only(true)        // Prevent XSS access
    .cookie_same_site(SameSite::Lax)  // CSRF protection
    .cookie_max_age(Some(Duration::hours(12)))  // Absolute timeout
    .build()
```

**References**:
- actix-session documentation: https://docs.rs/actix-session/latest/actix_session/
- OWASP Session Management Cheat Sheet

---

## Summary of Technical Decisions

| Component | Technology | Justification |
|-----------|-----------|---------------|
| Password Hashing | Argon2id (47MB, 3 iter) | OWASP 2024 compliant, GPU-resistant |
| Password Strength | zxcvbn | Realistic estimation, actionable feedback |
| Timing Safety | subtle crate | Constant-time comparisons prevent attacks |
| Rate Limiting | PostgreSQL | Simple, sufficient for scale, no Redis needed |
| Email | lettre + SMTP | Standard protocol, async, no vendor lock-in |
| Session Timeout | actix-session cookies | Works with existing infra, fast validation |

**All decisions align with Constitution v1.0.0:**
- ✅ KISS: Established libraries, standard protocols
- ✅ YAGNI: Only implementing needed features (Phases 1-2)
- ✅ Performance Through Simplicity: No caching/queuing layers
- ✅ Minimal Dependencies: Each new dep justified

## Next Steps

Phase 1: Design detailed data models, API contracts, and test scenarios based on these research findings.
