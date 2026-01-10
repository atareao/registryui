# RegistryUI

This project is a full-stack web application featuring a Rust backend with Axum and a React frontend built with Vite. It appears to be designed as a user interface for a registry system.


## Technologies

*   **Frontend:** React, Vite, TypeScript, Node.js v22
*   **Backend:** Rust, Axum, Tokio
*   **Containerization:** Docker

## Project Structure

The project is divided into two main directories:

*   `frontend/`: Contains the React application.
*   `backend/`: Contains the Rust application.

## Setup and Installation

### Prerequisites

*   Docker
*   Node.js (version 22 or higher recommended)
*   pnpm (package manager for frontend)
*   Rust toolchain

### Backend Configuration

The backend relies on environment variables, likely defined in a `.env` file. A sample `.env` file might be needed for local development.

### Building and Running

The project uses `just` for development and building.

1.  **Install Dependencies:**
    *   Frontend: `cd frontend && pnpm install`
    *   Backend: `cd backend && cargo build` (or `just backend` / `just watch` will handle this)

2.  **Development:**
    *   To run the frontend development server:
        ```bash
        just frontend
        ```
    *   To run the backend development server:
        ```bash
        just backend
        ```
    *   To run the backend with hot-reloading:
        ```bash
        just watch
        ```
    *   To run both frontend and backend with build steps for the frontend:
        ```bash
        just dev
        ```

3.  **Docker:**
    *   To build the Docker image:
        ```bash
        just build
        ```
    *   To push the Docker image (requires authentication):
        ```bash
        just push
        ```

## Database Migrations

The project uses `sqlx` for database migrations. To revert migrations:

```bash
cd backend
just revert
```

### Token de registry

```bash
# Definir variables
USER="tu_usuario"
PASS="tu_password"

# Generar el encoded (opción con printf es la más robusta)
ENCODED=$(printf "%s:%s" "$USER" "$PASS" | base64)

echo "El valor para el header es: Basic $ENCODED"
```
