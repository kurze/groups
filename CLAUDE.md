# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust web application for group management - a toy project to learn Rust with goals to create a Meetup equivalent. The application uses Actix-web as the HTTP server with PostgreSQL as the database and Tera templating for HTML rendering.

## Architecture

The codebase follows a layered architecture:

- **API Layer** (`src/api/`): REST endpoints and HTML handlers
  - `groups_api.rs` - CRUD operations for groups (`/groups` endpoints)
  - `groups_html.rs` - HTML API endpoints
  - `hello.rs` - Demo counter service
- **Database Layer** (`src/db/`): Business logic and data access
  - `group.rs` - GroupService with business logic
  - `user.rs` - UserService with business logic
  - `models/` - Data models with versioning (Group V1, User V0->V1)
- **Main Application** (`src/main.rs`): HTTP server setup, routes, template handlers

The application serves both REST API endpoints and rendered HTML templates. Static assets are served from `/static` and include `htmz.js` for frontend interactivity.

## Database Models

Uses PostgreSQL with SQLx and migration support:

- **Users**: id (primary), email (unique), name, password_hash, timestamps, soft delete
- **Groups**: id (primary), name, timestamps, soft delete

Database schema is managed through SQL migrations in the `migrations/` directory.

## Development Commands

The project uses Task (taskfile.dev) for build automation:

```bash
# Development workflow
task run              # Run development server (cargo run)
task build            # Debug build (cargo build)
task build-release    # Release build (cargo build --release)

# Code quality
task fmt              # Format code (cargo fmt)
task test             # Run tests (cargo test)
task pre-commit       # Format + test

# Direct cargo commands also work
cargo run             # Runs server on 127.0.0.1:8080
cargo test            # Unit tests
cargo fmt             # Code formatting
```

## API Endpoints

REST API for groups management:

- `GET /groups` - List active groups
- `GET /groups/{id}` - Get specific group
- `POST /groups` - Create group
- `PUT /groups/{id}` - Update group  
- `DELETE /groups/{id}` - Soft delete group
- `GET /groups/search?name=` - Search by name

## Key Dependencies

- `actix-web` (~4) - Web framework
- `sqlx` (0.8) - Async PostgreSQL driver with compile-time checked queries
- `tokio` (1) - Async runtime
- `tera` (1) - Template engine (Jinja2-like)
- `serde` - JSON serialization
- `chrono` - Date/time handling
- `argon2` - Password hashing (currently unused)

## Testing

Comprehensive test coverage exists for API endpoints. Run tests with `task test` or `cargo test`.

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
