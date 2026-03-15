use anyhow::Result;

// Подключаем наш новый модуль, где будет вся логика
mod app;

#[tokio::main]
async fn main() -> Result<()> {
    app::run().await
}
