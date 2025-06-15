# Groups

A Rust web application for group management - a toy project to learn Rust with goals to create a Meetup equivalent. The application uses Actix-web as the HTTP server with PostgreSQL as the database and Tera templating for HTML rendering.

## Tech Stack
 * [Actix Web](https://actix.rs/docs) - Web framework for Rust
 * [PostgreSQL](https://www.postgresql.org/) - Database with Docker support
 * [SQLx](https://docs.rs/sqlx/latest/sqlx/) - Async PostgreSQL driver with compile-time checked queries
 * [Tera](https://tera.netlify.app/) - Template engine (Jinja2-like)
 * [Argon2](https://docs.rs/argon2/latest/argon2/) - Password hashing library
 * [htmz](https://leanrada.com/htmz/) - Minimalist HTML microframework (166 bytes)
 * [Task](https://taskfile.dev/) - Task runner/build tool

## Development

To install Task (not mandatory), please refer to the [installation guide](https://taskfile.dev/installation/).

### Quick Start

1. Copy environment configuration:
```bash
cp .env.example .env
```

2. Start PostgreSQL database:
```bash
task db-up
```

3. Run the application:
```bash
task dev
```

The application will be available at http://127.0.0.1:8080

### Development Commands

```bash
# Build the project
task build

# Build for production
task build-release

# Run the service locally
task run

# Format code
task fmt

# Run tests
task test

# Run pre-commit checks (format and test)
task pre-commit
```

### Docker Support

```bash
# Start full stack (database + application)
task docker-up

# View logs
task docker-logs

# Stop the stack
task docker-down
```

### Project Structure
- `src/api/` - REST endpoints and HTML handlers
- `src/db/` - Database layer with business logic and models
- `src/middleware/` - Authentication middleware
- `src/templates/` - Tera HTML templates
- `src/static/` - CSS, JS assets
- `migrations/` - PostgreSQL schema migrations
- `tests/` - Rust integration tests and Playwright E2E tests