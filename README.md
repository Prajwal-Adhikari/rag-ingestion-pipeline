# rag-ingestion-pipeline

A Rust-powered document ingestion engine for RAG systems. Documents are submitted via a Telegram bot, saved to disk, and queued in PostgreSQL for downstream processing (chunking, embedding, vector storage).

---

## Architecture

```
Telegram Bot
    │
    ▼
Receive document → Save to ./downloads/ → Insert job (status=pending) into PostgreSQL
                                                    │
                                                    ▼
                                            Worker (coming soon)
                                                    │
                                          ┌─────────┴──────────┐
                                          ▼                     ▼
                                   Extract text           Redis cache
                                   + Chunk                     │
                                          │               cache miss
                                          ▼                     ▼
                                   pgvector store        OpenAI / ONNX
```

---

## Prerequisites

- Rust (stable)
- PostgreSQL 14+
- A Telegram bot token from [@BotFather](https://t.me/botfather)

---

## Setup

### 1. Clone and enter the project

```bash
git clone <your-repo>
cd rag-ingestion-pipeline
```

### 2. Configure environment

Create a `.env` file in the project root:

```env
TELOXIDE_TOKEN=your_telegram_bot_token
DATABASE_URL=postgres://user:password@localhost:5432/yourdb
```

### 3. Create the database schema

Connect to your PostgreSQL instance and run:

```sql
CREATE TABLE jobs (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    file_name   TEXT        NOT NULL,
    file_path   TEXT        NOT NULL,
    status      TEXT        NOT NULL DEFAULT 'pending',
    retries     INT         NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### 4. Create the downloads directory

```bash
mkdir downloads
```

### 5. Run the bot

```bash
RUST_LOG=info cargo run
```

---

## Usage

1. Open Telegram and find your bot by username
2. Send `/start` to initialize the chat
3. Send any document (PDF, txt, etc.)
4. The bot saves the file to `./downloads/` and inserts a `pending` job into PostgreSQL

---

## Project Structure

```
rag-ingestion-pipeline/
├── src/
│   └── main.rs          # Telegram bot + file download + job insert
├── downloads/           # Saved documents (gitignored)
├── .env                 # Local secrets (gitignored)
├── Cargo.toml
└── README.md
```

---

## Cargo.toml Dependencies

```toml
[dependencies]
teloxide   = { version = "0.17", features = ["macros"] }
tokio      = { version = "1",    features = ["rt-multi-thread", "macros", "fs"] }
sqlx       = { version = "0.8",  features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
uuid       = { version = "1",    features = ["v4"] }
chrono     = { version = "0.4",  features = ["serde"] }
dotenvy    = "0.15"
log        = "0.4"
pretty_env_logger = "0.5"
```

---

## Roadmap

- [x] Telegram bot receives documents
- [x] Files saved to local `./downloads/` directory
- [x] Job record inserted into PostgreSQL on upload
- [ ] Worker: poll `jobs` table with `FOR UPDATE SKIP LOCKED`
- [ ] Text extraction from PDF/HTML
- [ ] Semantic chunking
- [ ] Redis embedding cache
- [ ] OpenAI / local ONNX embedding generation
- [ ] pgvector storage
- [ ] OpenTelemetry tracing
- [ ] Docker Compose setup
- [ ] Benchmark suite

---

## Security Notes

- Restrict the bot to your own Telegram user ID in the message handler to prevent strangers from uploading files
- Rotate your bot token via BotFather if it is ever exposed publicly (`/mybots` → select bot → API Token → Revoke)
- Never hardcode secrets in source code — always use environment variables

---
