// +OK\r\n
pub fn simple_string(msg: &str) -> String {
    format!("+{}\r\n", msg)
}

// -ERR something\r\n
pub fn error(msg: &str) -> String {
    format!("-ERR {}\r\n", msg)
}

// :42\r\n
pub fn integer(n: i64) -> String {
    format!(":{}\r\n", n)
}

// $6\r\nAditya\r\n
pub fn bulk_string(value: &str) -> String {
    format!("${}\r\n{}\r\n", value.len(), value)
}

// $-1\r\n  (nil — key not found)
pub fn nil() -> String {
    "$-1\r\n".to_string()
}

// *3\r\n$3\r\nfoo\r\n...  (array of bulk strings)
pub fn array(items: &[String]) -> String {
    let mut result = format!("*{}\r\n", items.len());
    for item in items {
        result.push_str(&bulk_string(item));
    }
    result
}

// *0\r\n  (empty array)
pub fn empty_array() -> String {
    "*0\r\n".to_string()
}
