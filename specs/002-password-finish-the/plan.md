# Implementation Plan: Complete Password Security Implementation

**Branch**: `002-password-finish-the` | **Date**: 2025-10-01 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/home/simon/code/groups/specs/002-password-finish-the/spec.md`

## Summary

Implement secure password management features including OWASP 2024-compliant Argon2id hashing, zxcvbn-based realistic password strength validation, exponential backoff rate limiting, password reset with cryptographically secure tokens, session management with timeouts, and comprehensive security logging. The implementation prioritizes simplicity and directness—achieving security through well-designed code and efficient algorithms rather than adding caching layers or external infrastructure.

## Technical Context

**Language/Version**: Rust 1.85 (edition 2024)
**Primary Dependencies**:
- Existing: actix-web (~4), sqlx (0.8 with PostgreSQL), argon2 (0.5.3), actix-session (0.10), tera (1)
- New: zxcvbn (password strength), subtle (constant-time ops), lettre/smtp (email notifications)

**Storage**: PostgreSQL with SQLx migrations
**Testing**: cargo test (unit/integration), Playwright (E2E)
**Target Platform**: Linux server (Docker-compatible)
**Project Type**: single (web application with backend + templates)
**Performance Goals**:
- API response <200ms p95 (simple), <500ms p95 (complex)
- Password hashing <1s per operation
- No caching layers—performance through efficient queries

**Constraints**:
- Constitutional: Simplicity over optimization, no Redis/caching unless measured necessity
- Security: Timing-attack prevention, constant-time comparisons
- UX: Actionable error messages, real-time password strength feedback

**Scale/Scope**: Learning project, modest traffic (hundreds of users)

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Principle I: Code Quality & Maintainability**
- ✅ **KISS**: Password validation uses zxcvbn library (established, simple API) rather than custom complexity rules
- ✅ **YAGNI**: Implementing only Phase 1 & 2 from password management plan—no MFA, SSO, or WebAuthn yet
- ✅ **DRY**: Reuse existing password module, extend rather than duplicate
- ✅ **Single Responsibility**: Each module handles one concern (validation, hashing, rate limiting, password reset)
- ✅ **Explicit Over Implicit**: All security functions return Result types with explicit error handling
- ✅ **Module Boundaries**: Clear separation: API layer → UserService → database access
- ✅ **Minimal Dependencies**: New deps justified (zxcvbn=strength, subtle=timing safety, lettre=email)
- ✅ **Document Why**: Comments explain security rationale (constant-time, exponential backoff)

**Principle II: Testability & Test Coverage**
- ✅ **Testable Design**: Rate limiting state in database (testable), password validation pure functions
- ✅ **Test Coverage**: Integration tests for auth flows, unit tests for password strength, E2E for reset flow
- ✅ **Test Isolation**: Use test database on port 5433, clean state between tests
- ✅ **Flexible TDD**: Write tests alongside implementation—tests encouraged, not mandated first

**Principle III: User Experience Consistency**
- ✅ **Template Consistency**: Reuse existing Tera base templates for password reset pages
- ✅ **Error Handling**: Generic messages ("Invalid email or password") prevent user enumeration
- ✅ **Form Validation**: Real-time strength feedback via JavaScript + htmz, server-side authoritative
- ✅ **Session Management**: Leverage existing actix-session, add timeout enforcement
- ✅ **Responsive Design**: Password reset forms follow existing mobile/desktop patterns

**Principle IV: Performance Through Simplicity**
- ✅ **No Premature Optimization**: NO Redis for rate limiting—use PostgreSQL with efficient queries and indexes
- ✅ **Simplicity First**: Rate limit tracking via database table, not distributed cache
- ✅ **Baseline Targets**: Argon2 tuned for <1s hash time, database queries optimized with indexes
- ✅ **No Caching Layers**: Email sending async but direct—no queue infrastructure
- ✅ **Perceived Performance**: Async operations + optimistic UI feedback for password strength

**GATE RESULT**: ✅ PASS - No constitutional violations

## Project Structure

### Documentation (this feature)
```
specs/002-password-finish-the/
├── plan.md              # This file (/plan command output)
├── spec.md              # Feature specification
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
│   ├── password-reset-request.yaml
│   ├── password-reset-confirm.yaml
│   ├── password-change.yaml
│   └── registration-with-strength.yaml
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
src/
├── api/
│   ├── auth.rs          # Extend with password reset endpoints
│   └── mod.rs
├── db/
│   ├── models/
│   │   ├── user.rs      # Extend with reset token fields, failed attempts
│   │   ├── password_reset_token.rs  # NEW
│   │   ├── auth_log.rs  # NEW
│   │   └── rate_limit.rs  # NEW
│   ├── user.rs          # Extend UserService with password reset, rate limit logic
│   └── connection.rs    # Database pool, migrations
├── middleware/
│   ├── auth.rs          # Extend with session timeout checks
│   └── rate_limit.rs    # NEW - rate limiting middleware
├── password.rs          # Extend with strength validation (zxcvbn)
├── email.rs             # NEW - email sending for notifications
└── main.rs              # Configure session timeouts

templates/
├── password/
│   ├── reset_request.html  # NEW
│   ├── reset_form.html     # NEW
│   └── reset_success.html  # NEW
└── auth/
    └── register.html    # UPDATE - add strength indicator

static/
└── js/
    └── password-strength.js  # NEW - real-time strength feedback

migrations/
└── YYYYMMDDHHMMSS_add_password_security.sql  # NEW

tests/
├── integration/
│   ├── password_reset_test.rs  # NEW
│   ├── rate_limit_test.rs      # NEW
│   └── password_strength_test.rs  # NEW
└── unit/
    └── password_validation_test.rs  # NEW

tests/e2e/
└── password-reset.spec.ts  # NEW - Playwright test
```

**Structure Decision**: Single project structure. This is a web application with backend (Rust/Actix) and server-rendered HTML templates. All source in `src/`, tests in `tests/`, templates in `templates/`. No frontend/backend split needed—server renders HTML with htmz for interactivity.

## Phase 0: Outline & Research

**No NEEDS CLARIFICATION markers found in spec**—all requirements are well-defined. Research focuses on best practices for chosen technologies.

### Research Topics

1. **Argon2 OWASP 2024 Configuration**
   - Decision: Argon2id with 47MB memory (48,000 KB), 3 iterations, parallelism 1
   - Rationale: OWASP 2024 recommendations balance security and performance
   - Verification: Benchmark on target hardware to ensure <1s hash time

2. **zxcvbn Password Strength Library**
   - Decision: Use `zxcvbn` Rust crate for realistic password strength estimation
   - Rationale: Industry-standard algorithm, evaluates patterns/dictionary/sequences, provides actionable feedback
   - Integration: Pure Rust library, no external services required

3. **Constant-Time Comparison**
   - Decision: Use `subtle` crate for ConstantTimeEq trait
   - Rationale: Prevents timing attacks during authentication
   - Application: Password verification, token validation, user lookups

4. **Rate Limiting Without Redis**
   - Decision: PostgreSQL table with indexes on (identifier, action_type, timestamp)
   - Rationale: Simpler than distributed cache, sufficient for single-instance deployment
   - Cleanup: Periodic deletion of old records via scheduled job or lazy cleanup

5. **Email Sending**
   - Decision: `lettre` crate with SMTP backend (configurable via env vars)
   - Rationale: Established Rust email library, supports async
   - Configuration: SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASS in `.env`

6. **Session Timeout Implementation**
   - Decision: Store last_activity timestamp in session, check on each request
   - Rationale: Works with existing actix-session cookie-based storage
   - Cleanup: Sessions auto-expire via cookie MaxAge + server-side validation

**Output**: research.md (to be generated in Phase 0 execution)

## Phase 1: Design & Contracts

### Data Model

**Entities** (detailed in data-model.md):

1. **User** (extend existing)
   - Add fields: `failed_login_attempts INT DEFAULT 0`, `locked_until TIMESTAMP NULL`, `last_login_at TIMESTAMP NULL`

2. **PasswordResetToken** (new table)
   - `id` (UUID, primary key)
   - `user_id` (INT, foreign key to users)
   - `token_hash` (VARCHAR(255), indexed, hashed token for security)
   - `expires_at` (TIMESTAMP, indexed)
   - `used_at` (TIMESTAMP NULL)
   - `created_at` (TIMESTAMP)

3. **AuthLog** (new table)
   - `id` (SERIAL, primary key)
   - `user_id` (INT NULL, foreign key to users)
   - `event_type` (VARCHAR(50), e.g., "login_success", "login_failure", "password_reset")
   - `ip_address` (VARCHAR(45))
   - `user_agent` (TEXT NULL)
   - `created_at` (TIMESTAMP, indexed)

4. **RateLimitRecord** (new table)
   - `id` (SERIAL, primary key)
   - `identifier` (VARCHAR(255), indexed, e.g., IP or email)
   - `action_type` (VARCHAR(50), e.g., "login", "password_reset", "registration")
   - `attempt_count` (INT DEFAULT 1)
   - `window_start` (TIMESTAMP, indexed)
   - `next_allowed_at` (TIMESTAMP NULL)
   - Unique constraint on (identifier, action_type)

### API Contracts

**REST Endpoints** (detailed in contracts/):

1. **POST /api/auth/password-reset/request**
   - Request: `{ "email": "string" }`
   - Response: `{ "message": "If this email exists, a reset link has been sent" }` (constant time)
   - Status: 200 (always, prevents enumeration)

2. **POST /api/auth/password-reset/confirm**
   - Request: `{ "token": "string", "new_password": "string" }`
   - Response: `{ "message": "Password reset successful" }` or error
   - Status: 200 OK, 400 Bad Request (invalid token/password), 410 Gone (expired token)

3. **POST /api/auth/password/change**
   - Request: `{ "current_password": "string", "new_password": "string" }`
   - Response: `{ "message": "Password changed successfully" }`
   - Status: 200 OK, 401 Unauthorized, 400 Bad Request
   - Side effect: Invalidate all sessions except current

4. **POST /api/auth/register** (extend existing)
   - Add password strength validation
   - Response includes strength feedback if rejected

5. **POST /api/auth/login** (extend existing)
   - Add rate limiting logic
   - Add exponential backoff delays
   - Log authentication attempts

### HTML Endpoints

1. **GET /password-reset/request** - Form to request password reset
2. **GET /password-reset/confirm?token=xxx** - Form to set new password
3. **POST /password-reset/request** - Submit email
4. **POST /password-reset/confirm** - Submit new password

### Quickstart Test Scenarios

1. User registers with weak password → receives strength feedback
2. User registers with strong password → success
3. User fails login 3 times → experiences delays (1s, 2s, 4s)
4. User requests password reset → receives email (if exists)
5. User clicks reset link, sets new password → sessions invalidated
6. User changes password → all other sessions logged out

**Output**:
- data-model.md
- contracts/*.yaml (OpenAPI specs)
- quickstart.md
- Integration test scenarios

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
1. **Setup Tasks**: Add dependencies (zxcvbn, subtle, lettre), create migrations
2. **Data Layer**: Create new models (PasswordResetToken, AuthLog, RateLimitRecord)
3. **Core Logic**: Implement password strength validation, rate limiting, reset token generation
4. **Service Layer**: Extend UserService with password reset, rate limit checking
5. **API Layer**: Add password reset endpoints, extend auth endpoints
6. **Middleware**: Create rate limit middleware, extend auth middleware for timeouts
7. **Templates**: Create password reset HTML pages, update registration with strength indicator
8. **Email**: Implement email sending for notifications
9. **Tests**: Integration tests for each flow, unit tests for validation logic, E2E for full reset journey
10. **Documentation**: Update CLAUDE.md, add Taskfile commands for email testing

**Ordering Strategy**:
- Migrations before models
- Models before services
- Services before API endpoints
- Tests alongside each layer (flexible TDD)
- Mark [P] for parallel tasks (different files)

**Estimated Output**: 35-40 numbered tasks in tasks.md

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)
**Phase 4**: Implementation (execute tasks.md following constitutional principles)
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking

*No constitutional violations—table remains empty*

## Progress Tracking

**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved (none present)
- [x] Complexity deviations documented (none)

**Phase 1 Artifacts Generated**:
- ✅ research.md - Technical research and decisions
- ✅ data-model.md - Database schema and entity definitions
- ✅ contracts/ - 4 OpenAPI specifications for API endpoints
- ✅ quickstart.md - Manual testing guide and integration scenarios
- ✅ CLAUDE.md - Updated with new technical context

---
*Based on Constitution v1.0.0 - See `.specify/memory/constitution.md`*
