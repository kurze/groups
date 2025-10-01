# Quickstart: Complete Password Security Implementation

**Feature**: 002-password-finish-the
**Date**: 2025-10-01
**Purpose**: Manual testing guide and integration test scenarios

## Prerequisites

- Development environment running (`task dev`)
- Database migrations applied
- Test database available (`task db-test-up`)
- SMTP server configured (Mailhog on port 1025 for dev)

## Test Scenarios

### Scenario 1: User Registration with Password Strength Validation

**Objective**: Verify password strength validation using zxcvbn during registration

**Steps**:

1. **Attempt registration with weak password**:
   ```bash
   curl -X POST http://localhost:8080/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{
       "email": "test@example.com",
       "name": "Test User",
       "password": "password123"
     }'
   ```

   **Expected**: 400 Bad Request
   ```json
   {
     "error": "Password is too weak",
     "field": "password",
     "feedback": {
       "score": 0,
       "warning": "This is a very common password",
       "suggestions": [
         "Add another word or two",
         "Uncommon words are better"
       ]
     }
   }
   ```

2. **Attempt registration with password containing email**:
   ```bash
   curl -X POST http://localhost:8080/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{
       "email": "alice@example.com",
       "name": "Alice",
       "password": "alice@example.com123"
     }'
   ```

   **Expected**: 400 Bad Request (password contains user info)

3. **Register with strong password**:
   ```bash
   curl -X POST http://localhost:8080/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{
       "email": "alice@example.com",
       "name": "Alice Smith",
       "password": "correct-horse-battery-staple-2024"
     }'
   ```

   **Expected**: 201 Created
   ```json
   {
     "id": 1,
     "email": "alice@example.com",
     "name": "Alice Smith",
     "created_at": "2025-10-01T12:00:00Z"
   }
   ```

**Verification**:
- Database: `SELECT * FROM users WHERE email = 'alice@example.com'`
- Verify `password_hash` is set
- Verify `failed_login_attempts = 0`

---

### Scenario 2: Login with Rate Limiting and Exponential Backoff

**Objective**: Verify rate limiting prevents brute force attacks with exponential delays

**Steps**:

1. **First failed login** (immediate response):
   ```bash
   time curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "alice@example.com", "password": "wrong1"}'
   ```

   **Expected**: 401 Unauthorized, response time ~200ms

2. **Second failed login** (immediate response):
   ```bash
   time curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "alice@example.com", "password": "wrong2"}'
   ```

   **Expected**: 401 Unauthorized, response time ~200ms

3. **Third failed login** (1 second delay):
   ```bash
   time curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "alice@example.com", "password": "wrong3"}'
   ```

   **Expected**: 401 Unauthorized, response time ~1.2 seconds (1s backoff + processing)

4. **Fourth failed login** (2 second delay):
   ```bash
   time curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "alice@example.com", "password": "wrong4"}'
   ```

   **Expected**: 401 Unauthorized, response time ~2.2 seconds

5. **Fifth failed login** (4 second delay):
   ```bash
   time curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "alice@example.com", "password": "wrong5"}'
   ```

   **Expected**: 401 Unauthorized, response time ~4.2 seconds

6. **Successful login** (resets counter):
   ```bash
   curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -c cookies.txt \
     -d '{"email": "alice@example.com", "password": "correct-horse-battery-staple-2024"}'
   ```

   **Expected**: 200 OK, session cookie set, `failed_login_attempts` reset to 0

**Verification**:
- Database: `SELECT * FROM rate_limit_records WHERE identifier = '127.0.0.1' AND action_type = 'login'`
- Database: `SELECT * FROM auth_logs WHERE email_attempted = 'alice@example.com' ORDER BY created_at DESC LIMIT 6`
- Verify exponential backoff delays were applied
- Verify counter reset after successful login

---

### Scenario 3: Password Reset Flow (Full Journey)

**Objective**: Complete password reset from request to confirmation

**Setup**: Start Mailhog for email capture
```bash
docker run -d -p 1025:1025 -p 8025:8025 mailhog/mailhog
```

**Steps**:

1. **Request password reset**:
   ```bash
   curl -X POST http://localhost:8080/api/auth/password-reset/request \
     -H "Content-Type: application/json" \
     -d '{"email": "alice@example.com"}'
   ```

   **Expected**: 200 OK
   ```json
   {
     "message": "If this email exists in our system, a password reset link has been sent."
   }
   ```

2. **Check email** (Mailhog UI at http://localhost:8025):
   - Verify email received with reset link
   - Extract token from URL: `http://localhost:8080/password-reset/confirm?token=<TOKEN>`

3. **Attempt reset with weak password**:
   ```bash
   curl -X POST http://localhost:8080/api/auth/password-reset/confirm \
     -H "Content-Type: application/json" \
     -d '{
       "token": "<TOKEN>",
       "new_password": "weak123"
     }'
   ```

   **Expected**: 400 Bad Request (password too weak)

4. **Reset with strong password**:
   ```bash
   curl -X POST http://localhost:8080/api/auth/password-reset/confirm \
     -H "Content-Type: application/json" \
     -d '{
       "token": "<TOKEN>",
       "new_password": "my-new-secure-passphrase-2024"
     }'
   ```

   **Expected**: 200 OK
   ```json
   {
     "message": "Password reset successful. You can now log in with your new password."
   }
   ```

5. **Check email** (confirmation notification):
   - Verify second email received: "Your password was successfully changed"

6. **Verify old sessions invalidated**:
   - Previous session cookie should no longer work
   - Attempt to access protected endpoint with old cookie → 401 Unauthorized

7. **Login with new password**:
   ```bash
   curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -c cookies.txt \
     -d '{"email": "alice@example.com", "password": "my-new-secure-passphrase-2024"}'
   ```

   **Expected**: 200 OK

8. **Attempt token reuse** (should fail):
   ```bash
   curl -X POST http://localhost:8080/api/auth/password-reset/confirm \
     -H "Content-Type: application/json" \
     -d '{
       "token": "<SAME_TOKEN>",
       "new_password": "another-password"
     }'
   ```

   **Expected**: 410 Gone
   ```json
   {
     "error": "Password reset token has expired or already been used"
   }
   ```

**Verification**:
- Database: `SELECT * FROM password_reset_tokens WHERE user_id = 1`
- Verify `used_at` is set
- Database: `SELECT * FROM auth_logs WHERE user_id = 1 AND event_type LIKE 'password_reset%' ORDER BY created_at DESC`
- Verify log entries: request, success, (attempted reuse)

---

### Scenario 4: Password Reset Rate Limiting

**Objective**: Prevent password reset abuse through rate limiting

**Steps**:

1. **First reset request** (allowed):
   ```bash
   curl -X POST http://localhost:8080/api/auth/password-reset/request \
     -H "Content-Type: application/json" \
     -d '{"email": "alice@example.com"}'
   ```

   **Expected**: 200 OK

2. **Second reset request** (allowed):
   ```bash
   curl -X POST http://localhost:8080/api/auth/password-reset/request \
     -H "Content-Type: application/json" \
     -d '{"email": "alice@example.com"}'
   ```

   **Expected**: 200 OK

3. **Third reset request** (allowed):
   ```bash
   curl -X POST http://localhost:8080/api/auth/password-reset/request \
     -H "Content-Type: application/json" \
     -d '{"email": "alice@example.com"}'
   ```

   **Expected**: 200 OK

4. **Fourth reset request** (rate limited):
   ```bash
   curl -X POST http://localhost:8080/api/auth/password-reset/request \
     -H "Content-Type: application/json" \
     -d '{"email": "alice@example.com"}'
   ```

   **Expected**: 429 Too Many Requests
   ```json
   {
     "error": "Too many password reset requests. Please try again later.",
     "retry_after_seconds": 1800
   }
   ```

**Verification**:
- Database: `SELECT * FROM rate_limit_records WHERE identifier = 'alice@example.com' AND action_type = 'password_reset'`
- Verify `attempt_count = 4`, `next_allowed_at` set to future timestamp

---

### Scenario 5: Password Change (Authenticated User)

**Objective**: Change password while logged in, invalidating other sessions

**Setup**:
1. Login and save session cookie:
   ```bash
   curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -c session1.txt \
     -d '{"email": "alice@example.com", "password": "my-new-secure-passphrase-2024"}'
   ```

2. Login again in a "second device" (different cookie file):
   ```bash
   curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -c session2.txt \
     -d '{"email": "alice@example.com", "password": "my-new-secure-passphrase-2024"}'
   ```

**Steps**:

1. **Verify both sessions work**:
   ```bash
   curl -X GET http://localhost:8080/api/groups -b session1.txt  # Should work
   curl -X GET http://localhost:8080/api/groups -b session2.txt  # Should work
   ```

2. **Attempt password change with wrong current password**:
   ```bash
   curl -X POST http://localhost:8080/api/auth/password/change \
     -H "Content-Type: application/json" \
     -b session1.txt \
     -d '{
       "current_password": "wrong-password",
       "new_password": "another-secure-passphrase"
     }'
   ```

   **Expected**: 401 Unauthorized

3. **Change password with correct current password**:
   ```bash
   curl -X POST http://localhost:8080/api/auth/password/change \
     -H "Content-Type: application/json" \
     -b session1.txt \
     -d '{
       "current_password": "my-new-secure-passphrase-2024",
       "new_password": "final-secure-passphrase-2024"
     }'
   ```

   **Expected**: 200 OK

4. **Verify session1 still works** (current session):
   ```bash
   curl -X GET http://localhost:8080/api/groups -b session1.txt
   ```

   **Expected**: 200 OK (current session preserved)

5. **Verify session2 invalidated** (other session):
   ```bash
   curl -X GET http://localhost:8080/api/groups -b session2.txt
   ```

   **Expected**: 401 Unauthorized (session invalidated)

6. **Check email** (notification):
   - Verify email received: "Your password was changed from IP 127.0.0.1 at [timestamp]"

**Verification**:
- Database: `SELECT * FROM auth_logs WHERE user_id = 1 AND event_type = 'password_changed'`
- Mailhog: Verify password change notification email

---

### Scenario 6: Session Timeout (Idle and Absolute)

**Objective**: Verify session idle timeout (30 min) and absolute timeout (12 hours)

**Setup**:
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -c session.txt \
  -d '{"email": "alice@example.com", "password": "final-secure-passphrase-2024"}'
```

**Idle Timeout Test**:

1. **Access protected endpoint** (resets idle timer):
   ```bash
   curl -X GET http://localhost:8080/api/groups -b session.txt
   ```

   **Expected**: 200 OK

2. **Wait 31 minutes** (or mock time in test):
   ```bash
   # In integration test: Mock session.last_activity to 31 minutes ago
   ```

3. **Access protected endpoint** (should be timed out):
   ```bash
   curl -X GET http://localhost:8080/api/groups -b session.txt
   ```

   **Expected**: 401 Unauthorized
   ```json
   {
     "error": "Session expired due to inactivity. Please log in again."
   }
   ```

**Absolute Timeout Test**:

1. **Create session and mock created_at** to 13 hours ago
2. **Access endpoint repeatedly** (simulating activity):
   ```bash
   curl -X GET http://localhost:8080/api/groups -b session.txt
   ```

   **Expected**: 401 Unauthorized (absolute timeout exceeded)

---

### Scenario 7: Timing Attack Prevention

**Objective**: Verify constant-time responses prevent user enumeration

**Steps**:

1. **Request password reset for existing user**:
   ```bash
   time curl -X POST http://localhost:8080/api/auth/password-reset/request \
     -H "Content-Type: application/json" \
     -d '{"email": "alice@example.com"}'
   ```

   **Record response time**: ~200-300ms

2. **Request password reset for non-existent user**:
   ```bash
   time curl -X POST http://localhost:8080/api/auth/password-reset/request \
     -H "Content-Type: application/json" \
     -d '{"email": "nonexistent@example.com"}'
   ```

   **Record response time**: Should be within ±50ms of step 1 (constant time)

3. **Repeat 10 times for each** and compare distributions:
   ```bash
   for i in {1..10}; do
     time curl -X POST http://localhost:8080/api/auth/password-reset/request \
       -H "Content-Type: application/json" \
       -d '{"email": "alice@example.com"}' -o /dev/null -s
   done

   for i in {1..10}; do
     time curl -X POST http://localhost:8080/api/auth/password-reset/request \
       -H "Content-Type: application/json" \
       -d '{"email": "nonexistent@example.com"}' -o /dev/null -s
   done
   ```

   **Expected**: No statistically significant difference in response times

**Verification**:
- Response times should overlap significantly
- No way to distinguish existing vs. non-existing users via timing
- Both requests return identical 200 OK responses

---

## Integration Test Checklist

- [ ] Weak passwords rejected during registration
- [ ] Strong passwords accepted during registration
- [ ] Failed login attempts trigger exponential backoff (1s, 2s, 4s)
- [ ] Successful login resets failed attempt counter
- [ ] Password reset request sends email (if user exists)
- [ ] Password reset token expires after 15 minutes
- [ ] Password reset token is single-use
- [ ] Password reset invalidates all active sessions
- [ ] Password change invalidates other sessions (not current)
- [ ] Password change sends notification email
- [ ] Rate limiting prevents password reset abuse (3 per hour)
- [ ] Rate limiting prevents registration abuse (5 per hour per IP)
- [ ] Session idle timeout works (30 minutes)
- [ ] Session absolute timeout works (12 hours)
- [ ] Constant-time responses prevent user enumeration
- [ ] Authentication events logged to auth_logs table
- [ ] zxcvbn provides actionable password strength feedback

## End-to-End Test (Playwright)

**File**: `tests/e2e/password-reset.spec.ts`

**Scenarios**:
1. Complete password reset flow via UI
2. Real-time password strength indicator during registration
3. Rate limiting error messages displayed correctly
4. Session timeout redirects to login page

---

## Cleanup

After testing:
```bash
# Stop Mailhog
docker stop $(docker ps -q --filter ancestor=mailhog/mailhog)

# Reset test database
task db-test-down
task db-test-up

# Clear rate limit records
psql $DATABASE_URL -c "DELETE FROM rate_limit_records;"
```
