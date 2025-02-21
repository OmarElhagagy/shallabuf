# Shallabuf

## Architecture

Shallabuf follows a modular, service-oriented architecture where independent services communicate through a combination of **PostgreSQL events** and **NATS messaging**. The system is designed for scalability, leveraging **WebAssembly (WASM)** for secure and efficient execution of tasks.

### Backend Services

The backend is built in Rust and consists of several key services:

- **API Service (`api/`)**: Handles client requests and exposes RESTful endpoints.
- **Worker Service (`worker/`)**:
  - Accepts messages from the scheduler.
  - Executes tasks inside **WebAssembly (WASM)** modules using the **Wasmtime runtime**.
  - Uses **WASI (WebAssembly System Interface)** for secure interaction with the host environment.
  - Provides a lightweight, sandboxed execution environment for isolated tasks.
- **Scheduler Service (`scheduler/`)**:
  - Manages scheduled jobs and dispatches tasks to worker services.
  - Ensures efficient task distribution based on workload.
- **Event Bridge (`event-bridge/`)**:
  - Listens to **PostgreSQL events** (e.g., inserts, updates, deletes).
  - Dispatches these events to other services using **NATS**, a high-performance messaging system.
  - Enables real-time event-driven communication between system components.

### Database & Storage

- **PostgreSQL (`db/`)**: The primary relational database for structured application data.
- **MinIO**: Provides object storage for large files and unstructured data. (S3 compatible)

### Infrastructure

- **Docker & Docker Compose**: Used for containerization and service orchestration.
- **SQLx (`.sqlx/`)**: A Rust-based database toolkit for schema management and queries.
- **NATS**: A lightweight, high-performance messaging system for service communication.
- **Loki (`loki-config.yaml`)** & **Prometheus (`prometheus.yaml`)**: Used for logging, monitoring, and observability.

### Frontend

- **Next.js (`web/`)**: A React-based frontend framework for the user interface.
- **Bun**: A fast JavaScript runtime used for dependency management and development tooling.

By utilizing **PostgreSQL events and NATS**, Shallabuf achieves a highly efficient **event-driven architecture**, ensuring **real-time updates, decoupled communication, and scalable task execution**.

```mermaid
graph TD;
    subgraph Frontend
        NextJS["Next.js (web)"]
    end

    subgraph Backend
        API["API Service"]
        Worker["Worker Service (WASM)"]
        Scheduler["Scheduler Service"]
        EventBridge["Event Bridge"]
    end

    subgraph Infrastructure
        DB["PostgreSQL"]
        MinIO["MinIO (Object Storage)"]
        NATS["NATS (Message Broker)"]
        Loki["Loki (Logging)"]
        Prometheus["Prometheus (Monitoring)"]
    end

    NextJS -->|HTTP Requests| API
    API -->|Reads/Writes| DB
    DB -->|Triggers Events| EventBridge
    EventBridge -->|Dispatches Events| NATS
    NATS -->|Publishes Events| API
    NATS -->|Publishes Events| Scheduler
    Scheduler -->|Dispatch Jobs| Worker
    Worker -->|Executes WASM Tasks| NATS
    Worker -->|Stores Processed Data| MinIO
    Worker -->|Stores Metadata| DB
    NATS -->|Message Queue| Worker
    Prometheus -->|Monitors| Backend
    Loki -->|Logs| Backend
```

### Explanation

- **Frontend**: The Next.js app interacts with the backend via API calls.
- **API Service**: Communicates with the database and publishes events when changes occur.
- **Event Bridge**: Listens for PostgreSQL events and publishes them via NATS.
- **Scheduler**: Dispatches jobs to worker services via NATS.
- **Worker Service**: Executes WebAssembly (WASM) modules securely using Wasmtime.
- **Infrastructure**: Includes PostgreSQL (for data), MinIO (for object storage), NATS (for messaging), and monitoring/logging tools like Loki and Prometheus.

This provides a **clear, structured view** of your architecture in **Mermaid format**. Let me know if you want any modifications! 🚀

## Development

### Prerequisites

- rust ^1.84
- bun ^1.1
- docker ^27.4

### Setup

- Start all services

```sh
docker compose up --build
```

- Prepare sqlx

```sh
cargo install sqlx-cli
cargo sqlx prepare --workspace  --database-url="postgres://shallabuf:secret@localhost:5432/shallabuf"
```

- During active development you can compile and run desired services like so:

```sh
# Optinally install cargo-watch
cargo install cargo-watch
cargo watch -x "run --bin service_name" # worker || api || scheduler etc...
```

Note:
for running services on the host machine make sure you have correct .env file in your root directory
you can simple copy/paste .env.docker although you will need to replace service names with localhost.
e.g. replace `minio:9000` with `localhost:9000` and `postgresql:5432` with `localhost:5432`

- Frontend is written in Next.js

```sh
cd web && bun dev
```

### Prepare your environment

For MinIO `MINIO_ACCESS_KEY` and `MINIO_SECRET_KEY` you will need to login to MinIO console
use `admin` as username and `admin` as password

For the frontend simply copy/paste `web/.env.example` to `web/.env`
For the frontend simply copy/paste `web/.env.example` to `web/.env`

Test Users:

- <alex@mail.com> with password: alexpass
- <bob@mail.com> with password: bobpass
