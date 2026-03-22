use common::{create_pool, Job};
use sqlx::{types::uuid, PgPool};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::chunker::SemanticChunker;
mod chunker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    log::info!("Worker Starting");

    //testing python sidecar sentence splitter
    let mut chunker = SemanticChunker::new().await?;
    let sentences = chunker
        .chunk("Dr. Smith went to the store. He bought apples. The weather was nice. This is a localhost -> 127.0.0.1:8080")
        .await?;
    println!("Total: {} sentences", sentences.len());

    let pool = create_pool().await?;
    loop {
        match pick_job(&pool).await {
            Some(job) => {
                log::info!("Processing job: {} ({})", job.id, job.file_name);
                match process_job(&pool, &job).await {
                    Ok(_) => {
                        mark_done(&pool, job.id).await;
                        log::info!("Job {} done", job.id);
                    }
                    Err(e) => {
                        mark_failed(&pool, job.id).await;
                        log::error!("Job {} failed: {}", job.id, e);
                    }
                }
            }
            None => {
                log::info!("No pending jobs, sleeping...");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn pick_job(pool: &PgPool) -> Option<Job> {
    sqlx::query_as!(
        Job,
        r#"
            SELECT id, file_name, file_path, status, retries
            FROM jobs
            WHERE status = 'pending'
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        "#
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

async fn process_job(pool: &PgPool, job: &Job) -> Result<(), Box<dyn std::error::Error>> {
    //Mark as processing
    sqlx::query!(
        "UPDATE jobs SET status = 'processing' WHERE id = $1",
        job.id
    )
    .execute(pool)
    .await?;

    let text = extract_text(&job.file_path)?;
    log::info!("Extracted {} characters from {}", text.len(), job.file_name);
    println!("{}", &text[..500.min(text.len())]);

    Ok(())
}

async fn mark_done(pool: &PgPool, id: Uuid) {
    sqlx::query!(
        "UPDATE jobs SET status = 'done', updated_at = now() WHERE id = $1",
        id
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn mark_failed(pool: &PgPool, id: Uuid) {
    sqlx::query!(
        "UPDATE jobs SET status = 'failed', retries = retries + 1, updated_at = now() WHERE id = $1",
        id
    )
    .execute(pool)
    .await
    .unwrap();
}

fn extract_text(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "pdf" => extract_pdf(file_path),
        "txt" => extract_txt(file_path),
        "json" => extract_json(file_path),
        other => Err(format!("Unsupported file type: {}", other).into()),
    }
}

fn extract_pdf(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(file_path)?;
    let text = pdf_extract::extract_text_from_mem(&bytes)?;
    Ok(text)
}
fn extract_txt(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(file_path)?;
    Ok(text)
}

fn extract_json(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(file_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(parsed.to_string())
}
