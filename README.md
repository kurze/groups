# Groups

Toy project to learn Rust
The goal is to make a Meetup equivalent for our own group at first  and then may be open up more broadly

## Tools reference
 * [Native DB](https://github.com/native-db/native-db) - Embedded database for Rust
 * [Actix Web](https://actix.rs/docs) - Web framework for Rust
 * [Argon2](https://docs.rs/argon2/latest/argon2/) - Password hashing library
 * [Thiserror](https://docs.rs/thiserror/latest/thiserror/) - Error handling library
 * [Serde](https://serde.rs/) - Serialization/deserialization framework
 * [Task](https://taskfile.dev/) - Task runner/build tool

## Development

To install Task (not mandatory), please refer to the [installation guide](https://taskfile.dev/installation/).

### Setup and Commands

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

### Project Structure
- `src/db/` - Database interface and models
- `src/password.rs` - Password hashing utilities
- `src/main.rs` - Main application entry point