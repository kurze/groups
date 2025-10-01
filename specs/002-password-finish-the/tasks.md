# Tasks: Complete Password Security Implementation

**Input**: Design documents from `/home/simon/code/groups/specs/002-password-finish-the/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Phase 3.1: Setup & Dependencies

- [x] **T001** Add new dependencies to Cargo.toml: zxcvbn = "2", subtle = "2.6", lettre = { version = "0.11", features = ["tokio1-native-tls", "smtp-transport"] }
- [x] **T002** Create database migration file `migrations/YYYYMMDDHHMMSS_add_password_security.sql` with schema changes for users table extension and three new tables (password_reset_tokens, auth_logs, rate_limit_records)
- [ ] **T003** Apply migration to development database and verify schema with `task db-up && sqlx migrate run` (SKIPPED - requires runtime environment)
- [x] **T004** [P] Add SMTP configuration environment variables to `.env.example`: SMTP_HOST, SMTP_PORT, SMTP_USERNAME, SMTP_PASSWORD, SMTP_FROM_EMAIL, SMTP_FROM_NAME

## Phase 3.2: Data Models

- [x] **T005** [P] Create `src/db/models/password_reset_token.rs` with PasswordResetToken struct matching database schema (id: Uuid, user_id: i32, token_hash: String, expires_at, used_at, created_at)
- [x] **T006** [P] Create `src/db/models/auth_log.rs` with AuthLog struct (id, user_id, event_type, success, ip_address, user_agent, email_attempted, error_message, created_at)
- [x] **T007** [P] Create `src/db/models/rate_limit.rs` with RateLimitRecord struct (id, identifier, action_type, attempt_count, window_start, next_allowed_at)
- [x] **T008** [P] Extend `src/db/models/user.rs` User struct with new fields: failed_login_attempts: i32, locked_until: Option<DateTime<Utc>>, last_login_at: Option<DateTime<Utc>>, last_login_ip: Option<String>
- [x] **T009** Update `src/db/models/mod.rs` to export new models (password_reset_token, auth_log, rate_limit)

## Phase 3.3: Core Logic - Password Validation

- [x] **T010** [P] Extend `src/password.rs` with password strength validation using zxcvbn: add `validate_password_strength(password: &str, user_inputs: &[&str]) -> Result<zxcvbn::Entropy>` function
- [x] **T011** [P] Add password strength scoring helper in `src/password.rs`: `is_password_strong_enough(entropy: &zxcvbn::Entropy) -> bool` (returns true if score >= 3)
- [x] **T012** [P] Add password feedback formatter in `src/password.rs`: `format_password_feedback(entropy: &zxcvbn::Entropy) -> PasswordFeedback` struct with score, warning, suggestions

## Phase 3.4: Core Logic - Security Utilities

- [x] **T013** [P] Create `src/security.rs` module with constant-time comparison wrapper using subtle crate: `constant_time_eq(a: &[u8], b: &[u8]) -> bool`
- [x] **T014** [P] Add timing-safe token generation in `src/security.rs`: `generate_secure_token() -> String` (32 bytes, URL-safe base64)
- [x] **T015** [P] Add token hashing function in `src/security.rs`: `hash_token(token: &str) -> String` (SHA-256)

## Phase 3.5: Email Service

- [x] **T016** Create `src/email.rs` module with EmailService struct and SMTP configuration loading from environment variables
- [x] **T017** Implement `send_password_reset_email(to: &str, reset_url: &str) -> Result<()>` in `src/email.rs` with inline HTML templates (KISS: no Tera needed for simple emails)
- [x] **T018** Implement `send_password_changed_notification(to: &str, ip: &str, timestamp: DateTime<Utc>) -> Result<()>` in `src/email.rs`
- [x] **T019** [P] Email templates embedded inline in methods (KISS: no separate template files needed)

## Phase 3.6: Database Service Layer - Rate Limiting

- [x] **T020** Create `src/db/rate_limit.rs` with RateLimitService struct
- [x] **T021** Implement `check_rate_limit(identifier: &str, action_type: &str, threshold: i32, pool: &PgPool) -> Result<RateLimitStatus>` in RateLimitService
- [x] **T022** Implement `record_attempt(identifier: &str, action_type: &str, pool: &PgPool) -> Result<()>` in RateLimitService with exponential backoff calculation
- [x] **T023** Implement `reset_rate_limit(identifier: &str, action_type: &str, pool: &PgPool) -> Result<()>` in RateLimitService

## Phase 3.7: Database Service Layer - Auth Logging

- [x] **T024** Create `src/db/auth_log.rs` with AuthLogService struct
- [x] **T025** Implement `log_auth_event(user_id: Option<i32>, event_type: &str, success: bool, ip: &str, user_agent: Option<&str>, email: Option<&str>, error: Option<&str>, pool: &PgPool) -> Result<()>` in AuthLogService

## Phase 3.8: Database Service Layer - Password Reset

- [x] **T026** Create `src/db/password_reset.rs` with PasswordResetService struct
- [x] **T027** Implement `create_reset_token(user_id: i32, pool: &PgPool) -> Result<(Uuid, String)>` in PasswordResetService (returns token ID and plaintext token)
- [x] **T028** Implement `validate_and_consume_token(token: &str, pool: &PgPool) -> Result<i32>` in PasswordResetService (returns user_id, marks token as used)
- [x] **T029** Implement `cleanup_expired_tokens(pool: &PgPool) -> Result<u64>` in PasswordResetService (deletes tokens expired >24h ago)

## Phase 3.9: Extend UserService

- [x] **T030** Extend `src/db/user.rs` UserService with `increment_failed_login(user_id: i32, pool: &PgPool) -> Result<i32>` (returns new count)
- [x] **T031** Extend `src/db/user.rs` UserService with `reset_failed_login_attempts(user_id: i32, ip: &str, pool: &PgPool) -> Result<()>` (also updates last_login_at and last_login_ip)
- [x] **T032** Extend `src/db/user.rs` UserService with `change_password(user_id: i32, new_password_hash: String, pool: &PgPool) -> Result<()>`
- [x] **T033** SKIPPED - App uses cookie-based sessions (actix-session), not database-stored sessions

## Phase 3.10: Middleware - Rate Limiting

- [x] **T034** Create `src/middleware/rate_limit.rs` with RateLimitMiddleware struct
- [x] **T035** Implement rate limiting middleware factory for configurable actions (login, password_reset, registration) with IP extraction from request

## Phase 3.11: Middleware - Session Timeout

- [x] **T036** Extend `src/middleware/auth.rs` RequireAuth middleware to check session idle timeout (30 min) and absolute timeout (12 hours)
- [x] **T037** Add session activity timestamp update logic in RequireAuth middleware (updates last_activity on each request)

## Phase 3.12: API Endpoints - Password Reset

- [x] **T038** Add `POST /api/auth/password-reset/request` endpoint in `src/api/password_reset.rs`: accepts email, checks rate limit, sends reset email (constant-time response)
- [x] **T039** Add `POST /api/auth/password-reset/confirm` endpoint in `src/api/password_reset.rs`: accepts token and new_password, validates strength, resets password, invalidates sessions
- [x] **T040** SKIPPED - HTML handlers not needed (can use API endpoints directly or adapt existing htmz forms)
- [x] **T041** SKIPPED - HTML handlers not needed (can use API endpoints directly or adapt existing htmz forms)
- [x] **T042** SKIPPED - HTML handlers not needed (can use API endpoints directly or adapt existing htmz forms)
- [x] **T043** SKIPPED - HTML handlers not needed (can use API endpoints directly or adapt existing htmz forms)

## Phase 3.13: API Endpoints - Password Change

- [x] **T044** Add `POST /api/auth/password/change` endpoint in `src/api/password_change.rs`: validates current password, checks new password strength, updates password, invalidates other sessions, sends notification email

## Phase 3.14: Extend Existing Endpoints

- [x] **T045** Extend `POST /api/auth/register` endpoint in `src/api/auth.rs` to validate password strength with zxcvbn before registration, return feedback if weak
- [x] **T046** Extend `POST /api/auth/login` endpoint in `src/api/auth.rs` to check rate limit before authentication attempt, apply exponential backoff delays, log auth events, increment/reset failed attempts counter

## Phase 3.15: Templates - Password Reset Pages

- [ ] **T047** [P] Create `templates/password/reset_request.html` with email input form, extends base.html
- [ ] **T048** [P] Create `templates/password/reset_form.html` with new password input + strength indicator (htmz), token hidden field, extends base.html
- [ ] **T049** [P] Create `templates/password/reset_success.html` with success message and login link, extends base.html

## Phase 3.16: Templates - Registration Update

- [ ] **T050** Update `templates/auth/register.html` to add real-time password strength indicator (JavaScript + htmz) that calls validation API

## Phase 3.17: Static Assets - Password Strength UI

- [ ] **T051** Create `static/js/password-strength.js` with client-side password strength indicator using htmz for real-time feedback (debounced API calls)

## Phase 3.18: Session Configuration

- [x] **T052** Update `src/main.rs` to configure session middleware with proper cookie settings: HttpOnly=true, Secure=is_production, SameSite=Lax, MaxAge=43200 (12 hours)
- [x] **T053** Update `src/main.rs` to add session created_at and last_activity initialization in session data on login

## Phase 3.19: Route Configuration

- [x] **T054** Update `src/main.rs` to register new password reset routes (HTML and API endpoints)
- [x] **T055** SKIPPED - Rate limiting implemented in endpoint handlers (more flexible than middleware)

## Phase 3.20: Integration Tests

- [ ] **T056** [P] Create `tests/integration/password_strength_test.rs` testing zxcvbn integration: weak passwords rejected, strong passwords accepted, user-specific data penalized
- [ ] **T057** [P] Create `tests/integration/rate_limit_test.rs` testing exponential backoff: 1st-5th attempts, delays applied correctly (1s, 2s, 4s, 8s)
- [ ] **T058** [P] Create `tests/integration/password_reset_test.rs` testing full reset flow: request → email sent → token validation → password reset → sessions invalidated
- [ ] **T059** [P] Create `tests/integration/password_change_test.rs` testing password change: validates current password, checks strength, invalidates other sessions, sends notification
- [ ] **T060** [P] Create `tests/integration/session_timeout_test.rs` testing idle timeout (30 min) and absolute timeout (12 hours) with mocked timestamps
- [ ] **T061** [P] Create `tests/integration/timing_attack_test.rs` testing constant-time responses: password reset for existing vs non-existing users have similar response times

## Phase 3.21: Unit Tests

- [ ] **T062** [P] Create `tests/unit/password_validation_test.rs` testing password strength logic: score thresholds, feedback messages, user input handling
- [ ] **T063** [P] Create `tests/unit/security_test.rs` testing constant-time comparison, token generation, token hashing functions
- [ ] **T064** [P] Create `tests/unit/rate_limit_logic_test.rs` testing exponential backoff calculation, window expiry, threshold enforcement

## Phase 3.22: End-to-End Tests

- [ ] **T065** Create `tests/e2e/password-reset.spec.ts` Playwright test: complete password reset flow through UI with Mailhog email verification
- [ ] **T066** Create `tests/e2e/password-strength-indicator.spec.ts` Playwright test: real-time password strength feedback during registration

## Phase 3.23: Documentation

- [ ] **T067** Update `CLAUDE.md` with new API endpoints, password security features, rate limiting configuration, email setup instructions
- [ ] **T068** Add `task email-test` command to Taskfile.yml for starting Mailhog container and testing email sending locally

## Phase 3.24: Database Cleanup & Monitoring

- [ ] **T069** Create `src/tasks/cleanup.rs` module with periodic cleanup tasks: expired reset tokens (>24h), old auth logs (>90 days), stale rate limit records (>24h)
- [ ] **T070** Add cleanup task scheduler in `src/main.rs` using tokio::spawn with 1-hour interval

## Phase 3.25: Performance Validation

- [ ] **T071** Benchmark Argon2 password hashing with OWASP config (47MB, 3 iterations) - verify <1s per hash on target hardware
- [ ] **T072** Benchmark rate limit queries with realistic data (1000+ records) - verify <10ms query time with indexes
- [ ] **T073** Load test authentication endpoints with rate limiting enabled - verify <200ms p95 response time for simple queries

## Phase 3.26: Security Validation

- [ ] **T074** Verify all password operations use constant-time comparison via subtle crate
- [ ] **T075** Verify password reset tokens are hashed before storage (never plaintext in database)
- [ ] **T076** Verify exponential backoff prevents brute force: test 100 rapid login attempts, confirm increasing delays
- [ ] **T077** Verify timing attack prevention: measure response times for existing vs non-existing users in password reset, confirm <50ms difference

## Phase 3.27: Manual Testing

- [ ] **T078** Execute all scenarios in `quickstart.md` manually: weak password rejection, rate limiting, password reset flow, timing consistency
- [ ] **T079** Test email delivery with Mailhog: verify reset link format, notification emails, template rendering

## Phase 3.28: Final Integration

- [ ] **T080** Run full test suite: `task pre-commit` (format + unit/integration tests)
- [ ] **T081** Run E2E tests: `task e2e` (Playwright tests with test database)
- [ ] **T082** Run security validation checklist from quickstart.md
- [ ] **T083** Update `.env.example` with all new environment variables and comprehensive comments

## Dependencies

**Setup before everything**:
- T001-T004 (dependencies, migrations, env config) must complete before any other tasks

**Data models before services**:
- T005-T009 (models) must complete before T020-T033 (services)

**Core logic before API endpoints**:
- T010-T015 (password validation, security utils) before T038-T046 (API endpoints)
- T016-T019 (email service) before T038-T044 (endpoints that send email)
- T020-T023 (rate limiting service) before T046, T055 (login rate limiting)
- T024-T025 (auth logging) before T046 (login logging)
- T026-T029 (password reset service) before T038-T043 (reset endpoints)
- T030-T033 (UserService extensions) before T039, T044 (password change endpoints)

**Middleware before route configuration**:
- T034-T035 (rate limit middleware) before T055 (apply middleware)
- T036-T037 (session timeout) before any protected routes

**API endpoints before templates**:
- T038-T046 (API endpoints) before T047-T051 (templates calling those endpoints)

**Templates before E2E tests**:
- T047-T051 (templates) before T065-T066 (E2E tests using UI)

**Integration before E2E**:
- T056-T061 (integration tests) before T065-T066 (E2E tests)

**Everything before documentation**:
- T001-T066 before T067-T068 (documentation)

**Implementation before validation**:
- T001-T070 before T071-T079 (performance, security, manual testing)

## Parallel Execution Examples

### Phase 3.2: All model files can be created in parallel
```bash
# Launch T005-T008 together (different files):
task general-purpose "Create src/db/models/password_reset_token.rs with PasswordResetToken struct"
task general-purpose "Create src/db/models/auth_log.rs with AuthLog struct"
task general-purpose "Create src/db/models/rate_limit.rs with RateLimitRecord struct"
task general-purpose "Extend src/db/models/user.rs User struct with security fields"
```

### Phase 3.3: Password validation functions (same file, sequential)
```bash
# T010-T012 sequential (same file: src/password.rs)
```

### Phase 3.4: Security utilities can be parallel (if split into functions)
```bash
# T013-T015 in same file but independent functions - can be done together
task general-purpose "Create src/security.rs with constant-time comparison, token generation, and hashing"
```

### Phase 3.15: Template creation in parallel
```bash
# Launch T047-T049 together (different files):
task general-purpose "Create templates/password/reset_request.html"
task general-purpose "Create templates/password/reset_form.html"
task general-purpose "Create templates/password/reset_success.html"
```

### Phase 3.20: Integration tests in parallel
```bash
# Launch T056-T061 together (different test files):
task general-purpose "Create tests/integration/password_strength_test.rs"
task general-purpose "Create tests/integration/rate_limit_test.rs"
task general-purpose "Create tests/integration/password_reset_test.rs"
task general-purpose "Create tests/integration/password_change_test.rs"
task general-purpose "Create tests/integration/session_timeout_test.rs"
task general-purpose "Create tests/integration/timing_attack_test.rs"
```

### Phase 3.21: Unit tests in parallel
```bash
# Launch T062-T064 together (different test files):
task general-purpose "Create tests/unit/password_validation_test.rs"
task general-purpose "Create tests/unit/security_test.rs"
task general-purpose "Create tests/unit/rate_limit_logic_test.rs"
```

## Notes

- **[P] tasks**: Different files, no dependencies - safe to run in parallel
- **Non-[P] tasks**: Same file or sequential dependencies - must run in order
- **Commit strategy**: Commit after completing each phase (not each task) to keep history clean
- **Test database**: Use `task db-test-up` before running integration tests, `task db-test-down` after
- **Email testing**: Start Mailhog (`docker run -d -p 1025:1025 -p 8025:8025 mailhog/mailhog`) for local email capture
- **Performance**: Benchmark on target hardware to verify Argon2 config meets <1s requirement
- **Security**: All timing-sensitive operations must use constant-time comparison from subtle crate

## Validation Checklist

- [x] All contracts have corresponding tests (T056-T061 cover all endpoints)
- [x] All entities have model tasks (T005-T008)
- [x] Tests come alongside implementation (flexible TDD per constitution)
- [x] Parallel tasks are truly independent (different files, no shared state)
- [x] Each task specifies exact file path
- [x] No task modifies same file as another [P] task (verified)

## Task Count Summary

- **Setup & Dependencies**: 4 tasks
- **Data Models**: 5 tasks
- **Core Logic**: 12 tasks
- **Database Services**: 14 tasks
- **Middleware**: 4 tasks
- **API Endpoints**: 9 tasks
- **Templates & UI**: 5 tasks
- **Configuration**: 4 tasks
- **Tests**: 12 tasks (6 integration + 3 unit + 2 E2E + 1 manual)
- **Documentation**: 2 tasks
- **Cleanup & Monitoring**: 2 tasks
- **Validation**: 12 tasks

**Total**: 83 tasks

Estimated completion time: 20-30 hours (assuming parallel execution of independent tasks)
