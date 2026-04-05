use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;

pub async fn parse_command(reader: &mut BufReader<OwnedReadHalf>) -> Option<Vec<String>> {
    let mut line = String::new();

    // read first line — must start with *
    reader.read_line(&mut line).await.ok()?;
    let line = line.trim();

    if !line.starts_with('*') {
        return None;
    }

    // how many elements in the array
    let count: usize = line[1..].parse().ok()?;
    let mut parts = Vec::with_capacity(count);

    for _ in 0..count {
        let mut len_line = String::new();
        reader.read_line(&mut len_line).await.ok()?;
        let len_line = len_line.trim();

        // must start with $ (bulk string)
        if !len_line.starts_with('$') {
            return None;
        }

        // how many bytes to read
        let len: usize = len_line[1..].parse().ok()?;

        // read exactly len bytes + \r\n
        let mut buf = vec![0u8; len + 2];
        reader.read_exact(&mut buf).await.ok()?;

        // strip the \r\n
        let value = String::from_utf8_lossy(&buf[..len]).to_string();
        parts.push(value);
    }

    Some(parts)
}
