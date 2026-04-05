pub struct Config {
    pub host: String,
    pub port: u16,
    pub _password: Option<String>,
    pub _max_connection: usize,
}

impl Config {
    pub fn new() -> Config {
        Config {
            host: std::env::var("HOST").unwrap_or("127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .unwrap_or("6379".to_string())
                .parse()
                .unwrap_or(6379),
            _password: std::env::var("PASSWORD").ok(),
            _max_connection: std::env::var("MAX_CONNECTIONS")
                .unwrap_or("100".to_string())
                .parse()
                .unwrap_or(100),
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port).to_string()
    }
}
