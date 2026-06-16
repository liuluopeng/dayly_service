use common::front_can_do::get_url_from_mhtml::{get_subject_from_mhtml, get_url_from_mhtml};
use sqlx::PgPool;
use std::env;
use tokio;

#[derive(Debug)]
struct NoteRecord {
    id: uuid::Uuid,
    filename: Option<String>,
    raw_content: Option<Vec<u8>>,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://dayly_db:dayly_db@0.0.0.0:5432/dayly_db".to_string());

    let pool = PgPool::connect(&database_url).await?;

    println!("数据库连接成功");

    let records = sqlx::query_as!(
        NoteRecord,
        r#"
        SELECT id, filename, raw_content
        FROM notes
        WHERE filename LIKE '%.mhtml'
        AND raw_content IS NOT NULL
        
        "#
    )
    .fetch_all(&pool)
    .await?;

    println!("\n找到 {} 个MHTML文件\n", records.len());

    let mut url_updated_count = 0;
    let mut subject_updated_count = 0;
    let mut skipped_count = 0;

    for record in records {
        println!("ID: {}", record.id);
        println!(
            "文件名: {}",
            record.filename.unwrap_or_else(|| "未知".to_string())
        );

        if let Some(content) = record.raw_content {
            let content_str = String::from_utf8_lossy(&content);
            let lines: Vec<&str> = content_str.lines().take(100).collect();
            let header_content = lines.join("\n");

            println!("前10行内容:");
            for (i, line) in lines.iter().enumerate() {
                println!("  {}: {}", i + 1, line);
            }

            // 提取并更新URL
            if let Some(url) = get_url_from_mhtml(&header_content) {
                println!("找到URL: {}", url);

                sqlx::query!(
                    r#"
                    UPDATE notes
                    SET url = $1
                    WHERE id = $2
                    "#,
                    url,
                    record.id
                )
                .execute(&pool)
                .await?;

                url_updated_count += 1;
                println!("✓ URL已更新到数据库");
            } else {
                println!("⚠ 未找到URL");
            }

            // 提取并更新Subject
            if let Some(subject) = get_subject_from_mhtml(&header_content) {
                println!("找到Subject: {}", subject);

                sqlx::query!(
                    r#"
                    UPDATE notes
                    SET subject = $1
                    WHERE id = $2
                    "#,
                    subject,
                    record.id
                )
                .execute(&pool)
                .await?;

                subject_updated_count += 1;
                println!("✓ Subject已更新到数据库");
            } else {
                println!("⚠ 未找到Subject");
            }
        } else {
            println!("  (无内容)");
            skipped_count += 1;
        }

        println!();
    }

    println!("\n处理完成!");
    println!("更新了 {} 个URL", url_updated_count);
    println!("更新了 {} 个Subject", subject_updated_count);
    println!("跳过了 {} 个记录", skipped_count);

    Ok(())
}
