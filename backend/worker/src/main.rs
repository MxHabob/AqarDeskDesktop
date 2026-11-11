#[tokio::main]
async fn main() {
    tracing::info!("Starting AqarDesk background worker");

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

    loop {
        interval.tick().await;
        tracing::info!("AqarDesk background worker heartbeat");
    }
}
