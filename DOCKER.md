# Docker Setup for Groups Application

This document describes how to run the Groups application using Docker.

## Architecture

The application consists of two main services:
- **PostgreSQL Database**: Persistent data storage
- **Web Application**: Rust/Actix-web server

## Quick Start

### 1. Database Only (for local development)
```bash
# Start PostgreSQL database
task db-up

# Run application locally
task dev
```

### 2. Full Docker Stack
```bash
# Build the application image
task docker-build

# Start complete stack (database + application)
task docker-up

# View logs
task docker-logs

# Stop the stack
task docker-down
```

## Available Docker Commands

| Command | Description |
|---------|-------------|
| `task docker-build` | Build Docker image for the application |
| `task docker-up` | Start full application stack |
| `task docker-down` | Stop full application stack |
| `task docker-logs` | Show application logs |
| `task docker-logs-db` | Show database logs |
| `task docker-restart` | Restart the application container |
| `task docker-shell` | Open shell in application container |
| `task docker-rebuild` | Rebuild and restart the application |
| `task docker-clean` | Clean up Docker containers and volumes |

## Docker Compose Services

### Application Service (`app`)
- **Image**: Built from local Dockerfile
- **Port**: 8080 (mapped to host 8080)
- **Environment**: Production configuration
- **Profile**: `app` (not started by default)
- **Dependencies**: Waits for PostgreSQL to be healthy

### PostgreSQL Service (`postgres`)
- **Image**: postgres:16-alpine
- **Port**: 5432 (mapped to host 5432)
- **Database**: groups_dev
- **Credentials**: groups_user/groups_password
- **Volume**: Persistent data storage

### Test PostgreSQL Service (`postgres_test`)
- **Image**: postgres:16-alpine
- **Port**: 5433 (mapped to host 5433)
- **Database**: groups_test
- **Profile**: `test` (started only when testing)

## Environment Variables

The application uses the following environment variables in Docker:

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | postgres://... | PostgreSQL connection string |
| `RUST_LOG` | info | Logging level |
| `HOST` | 0.0.0.0 | Server bind address |
| `PORT` | 8080 | Server port |
| `SESSION_SECRET_KEY` | (required) | Session encryption key (64+ chars) |

## Development vs Production

### Development (Local)
- Run `task dev` for local development
- Uses `.env` file for configuration
- PostgreSQL runs in Docker, application runs locally
- Hot reload with `cargo run`

### Production (Docker)
- Run `task docker-up` for containerized deployment
- Uses `.env.production` configuration
- Both database and application run in Docker
- Optimized release build

## Troubleshooting

### Build Issues
- Ensure Docker has sufficient memory (4GB+ recommended)
- Build takes 5-10 minutes on first run due to Rust compilation
- Use `task docker-clean` to reset if builds fail

### Database Connection Issues
- Verify PostgreSQL is healthy: `docker compose ps`
- Check database logs: `task docker-logs-db`
- Ensure `.env.production` has correct DATABASE_URL

### Application Issues
- Check application logs: `task docker-logs`
- Verify environment variables are set correctly
- Use `task docker-shell` to debug inside container

## Security Notes

- Change `SESSION_SECRET_KEY` in production
- Use proper PostgreSQL credentials in production
- Consider using Docker secrets for sensitive data
- The application runs as non-root user (appuser) inside container