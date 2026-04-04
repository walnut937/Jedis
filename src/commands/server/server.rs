use crate::store::Db;
use crate::store::server::{data_type, flush_db, get_db_size, get_keys};

pub async fn handle(parts: &[&str], db: &Db) -> String {
    match parts[0] {
        "PING" => "PONG\n".to_string(),
        "ECHO" => match parts {
            [_, msg] => format!("{}\n", msg).to_string(),
            _ => "Err Some err on ECHO\n".to_string(),
        },
        "DBSIZE" => {
            let len = get_db_size(db).await;
            format!("{}\n", len).to_string()
        }
        "FLUSHDB" => flush_db(db).await,
        "TYPE" => match parts {
            [_, data] => data_type(db, data).await,
            _ => "Err Some err on TYPE command\n".to_string(),
        },
        "KEYS" => match parts {
            [_, pattern] => get_keys(db, pattern).await,
            _ => "Err Some err on KEYS command\n".to_string(),
        },
        _ => "UNKNOWN SERVER COMMAND\n".to_string(),
    }
}
