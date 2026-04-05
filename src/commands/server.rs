use crate::store::server::{data_type, flush_db, get_db_size, get_keys, server_info};
use crate::store::{Db, Stats};

pub async fn handle(parts: &[&str], db: &Db, stats: &Stats, port: u16) -> String {
    let command = parts[0].to_uppercase();
    match command.as_str() {
        "PING" => "PONG\n".to_string(),
        "ECHO" => match parts {
            [_, msg] => format!("{}\n", msg).to_string(),
            _ => "ERR wrong number of arguments for 'ECHO'\n".to_string(),
        },
        "DBSIZE" => {
            let len = get_db_size(db).await;
            format!("{}\n", len)
        }
        "FLUSHDB" => flush_db(db).await,
        "TYPE" => match parts {
            [_, data] => data_type(db, data).await,
            _ => "ERR wrong number of arguments for 'TYPE'\n".to_string(),
        },
        "KEYS" => match parts {
            [_, pattern] => get_keys(db, pattern).await,
            _ => "ERR wrong number of arguments for 'KEYS'\n".to_string(),
        },
        "INFO" => match parts {
            [_] => server_info(db, stats, port).await,
            _ => "ERR wrong number of arguments for 'INFO'\n".to_string(),
        },
        _ => "UNKNOWN SERVER COMMAND\n".to_string(),
    }
}
