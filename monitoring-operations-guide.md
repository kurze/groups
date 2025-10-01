# Monitoring Operations Guide

## Overview

This document provides operational guidance for the monitoring capabilities in the Groups Rust web application. 

**Important Note**: No "ultrathink" monitoring system was found in the codebase. This guide covers the existing monitoring infrastructure, which consists of basic health checks, logging, and operational tools.

## Current Monitoring Architecture

The application uses a simple monitoring approach focused on development and basic operations:

- **Health Checks**: Database connectivity verification and application startup validation
- **Logging**: Basic HTTP request logging and application logging via `env_logger`
- **Docker Monitoring**: Container health checks with automatic restart capabilities
- **Task-based Operations**: Comprehensive status checking and log access via Taskfile

## Daily Operations

### Quick System Health Check

```bash
# Check overall system status
task status
```

This command provides:
- Docker services status
- Database connectivity health
- Application HTTP response status

### Database Health Monitoring

```bash
# Check database connectivity
task db-health

# Alternative direct check
docker exec groups_postgres pg_isready -U groups_user -d groups_dev
```

### Application Logs Access

```bash
# View all service logs
task logs-all

# View specific service logs
task docker-logs        # Application container logs
task db-logs           # Database container logs

# Real-time log following
docker compose logs -f groups_app
docker compose logs -f postgres
```

## Health Check Details

### Database Health Check

**Location**: `src/db/connection.rs:61-68`

```rust
pub async fn health_check(pool: &DbPool) -> DbResult<bool> {
    let row = sqlx::query("SELECT 1 as health_check")
        .fetch_one(pool)
        .await?;
    let result: i32 = row.get("health_check");
    Ok(result == 1)
}
```

**Usage**: Automatically executed on application startup. Application exits with code 1 if database is unhealthy.

### Docker Health Check

**Configuration**: Dockerfile includes automatic health monitoring:

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/ || exit 1
```

**Behavior**: 
- Checks every 30 seconds
- 3-second timeout
- 5-second startup grace period
- Restarts container after 3 consecutive failures

## Application Startup Monitoring

The application performs database health verification on startup:

**Location**: `src/main.rs:32-43`

```rust
match db::health_check(&pool).await {
    Ok(true) => println!("Database connection: OK"),
    Ok(false) => {
        eprintln!("Database health check failed");
        std::process::exit(1);
    }
    Err(e) => {
        eprintln!("Database connection error: {}", e);
        std::process::exit(1);
    }
}
```

## Troubleshooting Procedures

### Application Won't Start

1. **Check database connectivity**:
   ```bash
   task db-health
   ```

2. **Check Docker services**:
   ```bash
   docker compose ps
   ```

3. **View application logs**:
   ```bash
   task docker-logs
   ```

### Database Connection Issues

1. **Verify PostgreSQL is running**:
   ```bash
   docker compose ps postgres
   ```

2. **Check database health**:
   ```bash
   docker exec groups_postgres pg_isready -U groups_user -d groups_dev
   ```

3. **Review database logs**:
   ```bash
   task db-logs
   ```

### High Response Times or Errors

1. **Check application logs**:
   ```bash
   docker compose logs -f groups_app
   ```

2. **Monitor HTTP requests** (logs include Actix-web middleware logging)

3. **Verify Docker container health**:
   ```bash
   docker inspect groups_app | grep -A 5 '"Health"'
   ```

## CI/CD Monitoring Integration

### GitHub Actions Health Checks

**Docker Build Pipeline**: `.github/workflows/docker-build.yml:144-149`
- Tests health endpoint availability after container startup
- Validates application responds correctly

**Integration Testing**: `.github/workflows/rust.yml:194-197`
- Automated health checks in CI pipeline
- Database connectivity validation

### Security Monitoring

- **Trivy vulnerability scanning** integrated in CI pipeline
- **Dependency auditing** via cargo audit

## Available Task Commands

```bash
# Core monitoring commands
task status          # Overall system health
task db-health       # Database connectivity check
task logs-all        # View all service logs
task docker-logs     # Application container logs

# Development and debugging
task run             # Start development server
task docker-run      # Start via Docker Compose
task docker-stop     # Stop all services
task docker-clean    # Clean up containers and volumes
```

## Logging Configuration

### Application Logging

- **Framework**: `env_logger` with `RUST_LOG=info`
- **HTTP Logging**: Actix-web Logger middleware enabled
- **Location**: Accessible via Docker container logs

### Log Levels

```bash
# Set log level (development)
export RUST_LOG=debug
cargo run

# Docker environment logging
RUST_LOG=info docker compose up
```

## Monitoring Limitations

### What's Missing

1. **No Dedicated Monitoring Endpoints**:
   - No `/health` endpoint for external monitoring
   - No `/metrics` endpoint for Prometheus integration
   - No `/status` or `/ready` endpoints

2. **No Application Metrics**:
   - No performance metrics (response times, throughput)
   - No business metrics (user registrations, group creation rates)
   - No error rate tracking beyond basic logging

3. **No Observability Infrastructure**:
   - No Prometheus metrics collection
   - No Grafana dashboards
   - No distributed tracing
   - No APM tool integration

4. **No Alerting System**:
   - No proactive notifications
   - No SLI/SLO monitoring
   - No automated incident detection

### Recommendations for Production

1. **Add Health Endpoints**: Implement `/health`, `/ready`, and `/metrics` endpoints
2. **Metrics Collection**: Integrate Prometheus for application metrics
3. **Observability Stack**: Consider adding Grafana for visualization
4. **Structured Logging**: Implement structured JSON logging for better parsing
5. **Error Tracking**: Add error tracking service integration
6. **Alerting**: Implement alerting for critical system failures

## Emergency Procedures

### Application Down

1. **Check Docker services**:
   ```bash
   task status
   ```

2. **Restart services**:
   ```bash
   docker compose restart
   ```

3. **Full restart if needed**:
   ```bash
   task docker-stop
   task docker-run
   ```

### Database Issues

1. **Check database logs**:
   ```bash
   task db-logs
   ```

2. **Restart database**:
   ```bash
   docker compose restart postgres
   ```

3. **Verify data integrity** after restart:
   ```bash
   task db-health
   ```

### Performance Issues

1. **Check resource usage**:
   ```bash
   docker stats
   ```

2. **Review application logs** for errors or warnings:
   ```bash
   task docker-logs
   ```

3. **Monitor HTTP request patterns** in logs

## Summary

The current monitoring system provides basic operational capabilities suitable for development and simple deployments. While it lacks advanced observability features, it covers essential health checking and log access needed for day-to-day operations.

For production use, consider implementing additional monitoring endpoints, metrics collection, and alerting capabilities as outlined in the recommendations section.