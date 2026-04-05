use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast;

pub async fn run_monitor(
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    monitor_tx: &Arc<broadcast::Sender<String>>,
) {
    let mut rx = monitor_tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(msg) => {
                if writer
                    .write_all(format!("{}\n", msg).as_bytes())
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
