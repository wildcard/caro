# Web API Service

A high-performance REST API built with Rust and Actix-web.

## Features

- Fast and efficient HTTP server
- PostgreSQL database integration
- Redis caching layer
- JWT authentication
- Rate limiting
- Comprehensive logging

## Quick Start

```bash
# Install dependencies
cargo build

# Run migrations
./scripts/migrate.sh

# Start server
cargo run

# Run tests
cargo test
```

## API Endpoints

- `GET /` - Health check
- `GET /users` - List all users
- `POST /users` - Create new user
- `GET /users/:id` - Get user by ID
- `PUT /users/:id` - Update user
- `DELETE /users/:id` - Delete user

## Configuration

Set environment variables in `config/.env`:

- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `API_PORT` - Server port (default: 8080)
- `LOG_LEVEL` - Logging level (info, debug, warn, error)

## Deployment

```bash
./scripts/deploy.sh
```

## License

MIT
