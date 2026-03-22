use common::create_pool;
use teloxide::{net::Download, prelude::*};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    log::info!("Starting doctor...");

    let bot = Bot::from_env();

    let pool = create_pool().await?;
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let pool = pool.clone();
        async move {
            if let Some(doc) = msg.document() {
                let file_name = doc
                    .file_name
                    .clone()
                    .unwrap_or_else(|| "unkown".to_string());
                let file = bot.get_file(doc.file.id.clone()).await.unwrap();
                let mut bytes = Vec::new();
                bot.download_file(&file.path, &mut bytes).await.unwrap();

                let save_path = format!("./downloads/{}", file_name);
                let mut out = File::create(&save_path).await.unwrap();
                out.write_all(&bytes).await.unwrap();

                sqlx::query!(
                    r#"
                        INSERT INTO jobs (file_name, file_path, status)
                        VALUES ($1, $2, 'pending')
                        "#,
                    file_name,
                    save_path
                )
                .execute(&pool)
                .await
                .unwrap();

                println!("Saved {} and inserted job", save_path);
            } else {
                println!("Got something else: {:?}", msg.text());
            }
            respond(())
        }
    })
    .await;
    Ok(())
}
