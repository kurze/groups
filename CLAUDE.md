# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust web application for group management - a toy project to learn Rust with goals to create a Meetup equivalent. The application uses Actix-web as the HTTP server with PostgreSQL as the database and Tera templating for HTML rendering.

## Architecture

The codebase follows a layered architecture:

- **API Layer** (`src/api/`): REST endpoints and HTML handlers
  - `groups_api.rs` - CRUD operations for groups (`/groups` endpoints)
  - `groups_html.rs` - HTML API endpoints
  - `auth.rs` - Authentication endpoints (login, register, logout)
  - `hello.rs` - Demo counter service
- **Database Layer** (`src/db/`): Business logic and data access
  - `group.rs` - GroupService with business logic
  - `user.rs` - UserService with business logic
  - `connection.rs` - Database pool management, migrations, health checks
  - `models/` - Data models with versioning (Group V1, User V0->V1)
- **Middleware Layer** (`src/middleware/`): Request processing
  - `auth.rs` - RequireAuth middleware for protecting routes (redirects to /login if not authenticated)
- **Main Application** (`src/main.rs`): HTTP server setup, routes, template handlers, session management

The application serves both REST API endpoints and rendered HTML templates. Static assets are served from `/static` and include `htmz.js` for frontend interactivity.

### Session Management
- Uses actix-session with cookie-based storage
- Session data stores: user_id, user_email, user_name
- SESSION_SECRET_KEY must be 64+ characters in production
- RequireAuth middleware checks for user_id in session

## Database Models

Uses PostgreSQL with SQLx and migration support:

- **Users**: id (primary), email (unique), name, password_hash, timestamps, soft delete
- **Groups**: id (primary), name, timestamps, soft delete

Database schema is managed through SQL migrations in the `migrations/` directory.

## Development Commands

### Quick Start
```bash
cp .env.example .env     # Copy environment configuration
task dev                 # Start database and run application
```

The project uses Task (taskfile.dev) for build automation:

```bash
# Development workflow
task dev                 # Start PostgreSQL + run server (recommended)
task dev-clean           # Reset database + run server
task run                 # Run server only (cargo run)
task build               # Debug build (cargo build)
task build-release       # Release build (cargo build --release)

# Database management
task db-up               # Start PostgreSQL container
task db-down             # Stop PostgreSQL container
task db-reset            # Reset database (drops all data)
task db-shell            # Connect to database shell (psql)
task db-test-up          # Start test database (port 5433)
task db-test-down        # Stop test database
task db-health           # Check database connectivity

# Code quality
task fmt                 # Format code (cargo fmt)
task test                # Run tests (cargo test)
task test-docker         # Run tests with test database
task pre-commit          # Format + test
task e2e                 # Run Playwright end-to-end tests
task e2e-ui              # Run Playwright tests with UI

# Docker deployment
task docker-build        # Build application Docker image
task docker-up           # Start full stack (database + app)
task docker-down         # Stop full stack
task docker-logs         # Show application logs
task status              # Show status of all services
```

### Environment Variables
Required environment variables (see `.env.example`):
- `DATABASE_URL` - PostgreSQL connection string (dev: port 5432)
- `DATABASE_TEST_URL` - Test database connection (port 5433)
- `SESSION_SECRET_KEY` - Session encryption key (64+ chars, required for production)
- `RUST_LOG` - Logging level (info, debug, etc.)
- `HOST` / `PORT` - Server bind address (default: 127.0.0.1:8080)
- `ENVIRONMENT` - Set to "production" for production mode (affects cookie security)

**SMTP Configuration** (required for password reset emails):
- `SMTP_HOST` - SMTP server hostname (e.g., "localhost" for Mailhog)
- `SMTP_PORT` - SMTP server port (e.g., 1025 for Mailhog)
- `SMTP_USERNAME` - SMTP authentication username (optional for local dev)
- `SMTP_PASSWORD` - SMTP authentication password (optional for local dev)
- `SMTP_FROM_EMAIL` - Sender email address (e.g., "noreply@groups.local")
- `SMTP_FROM_NAME` - Sender display name (e.g., "Groups Platform")

**Local Email Testing with Mailhog**:
```bash
# Start Mailhog for local email capture
docker run -d -p 1025:1025 -p 8025:8025 mailhog/mailhog
# Access web UI at http://localhost:8025
```

## API Endpoints

### HTML Pages
- `GET /` - Home page
- `GET /login` - Login page
- `GET /register` - Registration page (with password strength validation)
- `GET /groups` - Groups listing page
- `GET /groups/new` - Create group page (protected)

### Authentication API
- `POST /auth/login` - Login with rate limiting and account lockout
  - Form data: email, password
  - Rate limit: 5 attempts per 5 minutes per IP
  - Account lockout: 5 failed attempts = 15 minute lock
  - Logs all attempts (success and failure)
- `POST /auth/register` - Register with password strength validation
  - Form data: email, password
  - Rate limit: 5 attempts per 5 minutes per IP
  - Validates password strength (zxcvbn score ≥ 3)
  - Returns detailed feedback for weak passwords
- `GET /logout` - Logout and clear session

### Password Security API
- `POST /api/auth/password-reset/request` - Request password reset
  - JSON: `{"email": "user@example.com"}`
  - Rate limit: 3 attempts per 5 minutes per IP
  - Sends email with reset link (15 minute expiry)
  - Constant-time response (prevents user enumeration)
- `POST /api/auth/password-reset/confirm` - Confirm password reset
  - JSON: `{"token": "...", "new_password": "..."}`
  - Validates token (expiry, single-use)
  - Validates password strength
  - Invalidates all user's reset tokens
  - Sends confirmation email
- `POST /api/auth/password/change` - Change password (authenticated)
  - JSON: `{"current_password": "...", "new_password": "..."}`
  - Requires valid session
  - Validates current password
  - Validates new password strength
  - Sends notification email

### Groups REST API
- `GET /api/groups` - List active groups (JSON)
- `GET /api/groups/{id}` - Get specific group
- `POST /api/groups` - Create group
- `PUT /api/groups/{id}` - Update group
- `DELETE /api/groups/{id}` - Soft delete group
- `GET /api/groups/search?name=` - Search by name

## Key Dependencies

- `actix-web` (~4) - Web framework
- `actix-session` (0.10) - Session middleware with cookie storage
- `actix-files` (0.6) - Static file serving
- `sqlx` (0.8) - Async PostgreSQL driver with compile-time checked queries
- `tokio` (1) - Async runtime
- `tera` (1) - Template engine (Jinja2-like)
- `serde` (~1) - JSON serialization
- `chrono` (~0.4) - Date/time handling
- `argon2` (0.5.3) - Password hashing with salt
- `zxcvbn` (2) - Password strength validation
- `subtle` (2.6) - Constant-time cryptographic operations
- `lettre` (0.11) - SMTP email client
- `sha2` (0.10) - SHA-256 token hashing
- `uuid` (1) - UUID generation

## Password Security Features

### Password Strength Validation
- **Algorithm**: zxcvbn (realistic password strength estimation)
- **Minimum Score**: 3 out of 4 (safely unguessable)
- **User Context**: Penalizes passwords containing user's email or name
- **Max Length**: 128 characters
- **Feedback**: Returns detailed suggestions for weak passwords

### Password Hashing
- **Algorithm**: Argon2id (OWASP 2024 recommended)
- **Parameters**: Default (19 MiB memory, 2 iterations, 1 parallelism)
- **Salt**: Random per-password using OsRng

### Rate Limiting
- **Backend**: PostgreSQL (no Redis required, KISS principle)
- **Implementation**: Sliding window with exponential backoff
- **Thresholds**:
  - Login: 5 attempts per 5 minutes
  - Password Reset: 3 attempts per 5 minutes
  - Registration: 5 attempts per 5 minutes
- **Backoff**: 1 min → 5 min → 15 min

### Account Lockout
- **Trigger**: 5 failed login attempts
- **Duration**: 15 minutes automatic lock
- **Auto-recovery**: Lock expires automatically
- **Reset**: Successful login resets counter

### Password Reset Flow
- **Token Generation**: 32-byte cryptographically secure random tokens
- **Token Storage**: SHA-256 hashed (never plaintext in database)
- **Token Expiry**: 15 minutes
- **Single-use**: Token marked as used after consumption
- **Email**: Sent via SMTP with reset link

### Session Security
- **Storage**: Cookie-based (actix-session)
- **Settings**: HttpOnly, Secure (production), SameSite=Lax
- **Idle Timeout**: 30 minutes of inactivity
- **Absolute Timeout**: 12 hours from creation
- **Timestamps**: created_at and last_activity tracked

### Security Logging
- **Events Logged**: Login success/failure, password changes, reset requests, registrations
- **Data Tracked**: User ID, IP address, user agent, timestamp, error details
- **Purpose**: Audit trail for security incidents

### Timing Attack Prevention
- **Constant-time comparison**: Uses `subtle` crate for all sensitive comparisons
- **User enumeration prevention**: Password reset returns same response for existing/non-existing users
- **Response timing**: Carefully designed to avoid leaking user existence

## Testing

Run tests with `task test` or `cargo test`. Integration tests are in the `tests/` directory.

For tests requiring a database:
```bash
task db-test-up          # Start test database
cargo test               # Run tests
task db-test-down        # Stop test database
```

Or use the combined command:
```bash
task test-docker         # Manages test database automatically
```

End-to-end tests use Playwright:
```bash
task e2e                 # Run Playwright tests
task e2e-ui              # Run with interactive UI
```

## Working with htmz

htmz is a minimalist HTML microframework (166 bytes) that enables interactive web interfaces using pure HTML. Key concepts:

- **How it works**: Uses a hidden iframe as a proxy to load HTML fragments and swap them into specific page elements
- **Basic usage**: Add `target=htmz` to links/forms along with a URL fragment selector

  ```html
  <a href="/partial.html#element-id" target=htmz>Load content</a>
  ```

- **Installation**: Include the htmz iframe snippet in your HTML:

  ```html
  <iframe hidden name=htmz onload="setTimeout(()=>document.querySelector(contentWindow.location.hash||null)?.replaceWith(...contentDocument.body.childNodes))"></iframe>
  ```

- **Benefits**: Zero dependencies, uses standard HTML, no special attributes or DSLs

Note: The templates currently show HTMX attributes (hx-get, hx-post) which suggests either a migration in progress or confusion between htmz and HTMX. The static/js/htmz.js file appears to be a build tool for installing htmz snippets rather than the library itself.

## Password Management

The application uses Argon2id for password hashing:
- Hash function: `password::hash_password(password: &[u8]) -> Result<String>`
- Verify function: `password::verify_password(password: &[u8], hash: &str) -> Result<bool>`
- Argon2id with default parameters (19 MiB memory, 2 iterations, 1 parallelism)
- Random salt generation per password

## Docker Configuration

Two database containers are available:
- **Development database**: `groups_postgres` on port 5432 (database: `groups_dev`)
- **Test database**: `groups_postgres_test` on port 5433 (database: `groups_test`)

Docker Compose profiles:
- Default: Only development database
- `--profile test`: Include test database
- `--profile app`: Include application container + databases

## Rules

- use as many subagents as you need
- core principles
  - Keep It Simple, Stupid (KISS)
  - You Aren't Gonna Need It (YAGNI)
  - Don't Repeat Yourself (DRY), But Not Obsessively
  - Modularity & Single Responsibility Principle
  - Use explicit function returns rather than side effects
  - Maintain clear boundaries between modules
  - Minimize external dependencies
  - Document "why" not "what" (the code should show what it does)
  - create a maintainable solution, not a sophisticated one
  - if a command is needed, consider adding it to the Task file
