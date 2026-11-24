# Demo AI Sales Voice - Backend

High-performance AI Sales & Customer Service Voice Assistant backend service built in **Rust** using **Axum** & **Tokio**.

## 🏗️ Architecture

Built following **Domain-Driven Design (DDD)** and **Hexagonal Architecture (Ports and Adapters)**:
- **Domain Layer**:
  - Entities: `CallSession`
  - Value Objects: `SessionId`, `CallStatus`, `TranscriptEntry`, `Speaker`
  - Ports: `SessionRepository` trait
- **Application Layer**:
  - Use Cases / Services: `start_call`, `end_call`, `process_chat`, `get_session`
  - DTOs for structured request/response payload
- **Infrastructure Layer**:
  - In-memory repository adapter with `tokio::sync::RwLock`
  - Axum HTTP Router & CORS middleware
  - Comprehensive unit tests covering business logic and state transitions

## 🚀 Getting Started

```bash
# Run backend server (default port 8080)
cargo run

# Run with custom port
PORT=8080 cargo run

# Run unit tests
cargo test
```

## 📡 API Endpoints

- `GET /api/health` - Health check
- `POST /api/call/start` - Initiate a new call session
- `POST /api/call/{id}/end` - End an active call session
- `POST /api/call/{id}/chat` - Process conversational chat messages
- `GET /api/call/{id}` - Retrieve session state and full transcripts
