use sqlx::sqlite::SqliteConnection;
use sqlx::{query, Connection, SqlitePool, query_as};
// use sqlx::sql
use std::env;
use std::future::Future;
use std::task::Poll;

#[derive(Debug)]
struct Document {
    itemId: i64,
    title: String
}
enum ItemFiledIDs {
    Title = 1,
}
async fn get_docs(pool: &SqlitePool) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;
    let query = query_as!(
        Document,
        r#"
    SELECT items.itemID as itemId, itemDataValues.value as "title!: String"
        FROM items, itemData, itemDataValues
        WHERE
            itemData.itemID = items.itemID AND
            itemData.fieldID = ? AND
            itemDataValues.valueID = itemData.valueID
"#,
        ItemFiledIDs::Title as i32
    )
    .fetch_all(pool)
    .await?;
    dbg!(query);

    Ok(())
}
async fn do_test() -> anyhow::Result<()> {
    let url = env::var("DATABASE_URL")?;
    let pool = SqlitePool::connect(&url).await?;
    get_docs(&pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        dotenv::dotenv().ok();
        tokio_test::block_on(do_test()).unwrap();
    }
}
