use clap::Parser;
use std::sync::Arc;
use tokio::sync::RwLock;

// CLI args only — what user passes on startup
#[derive(Parser, Debug)]
#[command(name = "jedis")]
#[command(about = "A Redis-compatible server built in Rust")]
struct CliArgs {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value = "6379")]
    port: u16,

    #[arg(long)]
    password: Option<String>,

    #[arg(long, default_value = "100")]
    max_connections: usize,

    #[arg(long, default_value = "info")]
    loglevel: String,

    #[arg(long, default_value = "100")]
    monitor_buffer: usize,

    // path to conf file
    #[arg(long, default_value = "jedis.conf")]
    conf: String,
}

// runtime config — this is what the server actually uses
#[derive(Debug, Clone)]
pub struct Config {
    // cannot change at runtime — set once at startup
    pub host: String,
    pub port: u16,
    pub conf_path: String,

    // can change at runtime via CONFIG SET
    pub password: Option<String>,
    pub auth_enabled: bool,
    pub max_connections: usize,
    pub loglevel: String,
    pub monitor_buffer: usize,
}

pub type SharedConfig = Arc<RwLock<Config>>;

impl Config {
    pub fn new() -> Result<Config, String> {
        // step 1 — load .env (lowest priority)
        dotenvy::dotenv().ok();

        // step 2 — parse CLI args once
        let cli = CliArgs::parse();

        // step 3 — load conf file if it exists
        // conf file overrides .env but NOT CLI args
        let mut from_conf = ConfFile::default();
        if let Ok(content) = std::fs::read_to_string(&cli.conf) {
            println!("Loading config from {}", cli.conf);
            from_conf = ConfFile::parse(&content)?;
        }

        // step 4 — merge in priority order
        // CLI > conf file > .env > defaults
        // clap already handled CLI > .env > defaults
        // we just need to handle conf file in between
        Ok(Config {
            // these never change at runtime
            host: cli.host,
            port: cli.port,
            conf_path: cli.conf,

            // these can change — CLI wins over conf file
            password: cli.password.or(from_conf.password),
            auth_enabled: from_conf.auth_enabled.unwrap_or(false),
            max_connections: if cli.max_connections != 100 {
                cli.max_connections // user explicitly set it
            } else {
                from_conf.max_connections.unwrap_or(100)
            },
            loglevel: if cli.loglevel != "info" {
                cli.loglevel // user explicitly set it
            } else {
                from_conf.loglevel.unwrap_or("info".to_string())
            },
            monitor_buffer: if cli.monitor_buffer != 100 {
                cli.monitor_buffer
            } else {
                from_conf.monitor_buffer.unwrap_or(100)
            },
        })
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn save_to_conf(&self) -> Result<(), String> {
        let content = format!(
            "# jedis configuration\n\
             # do not edit while server is running — use CONFIG SET + CONFIG REWRITE\n\
             \n\
             host {}\n\
             port {}\n\
             password {}\n\
             auth {}\n\
             maxconnections {}\n\
             loglevel {}\n\
             monitor_buffer {}\n",
            self.host,
            self.port,
            self.password.as_deref().unwrap_or(""),
            if self.auth_enabled { "on" } else { "off" },
            self.max_connections,
            self.loglevel,
            self.monitor_buffer,
        );

        std::fs::write(&self.conf_path, content)
            .map_err(|e| format!("failed to write {}: {}", self.conf_path, e))
    }
}

pub fn create_shared_config(config: Config) -> SharedConfig {
    Arc::new(RwLock::new(config))
}

// internal — parses jedis.conf into typed values
#[derive(Default)]
struct ConfFile {
    password: Option<String>,
    auth_enabled: Option<bool>,
    max_connections: Option<usize>,
    loglevel: Option<String>,
    monitor_buffer: Option<usize>,
}

impl ConfFile {
    fn parse(content: &str) -> Result<ConfFile, String> {
        let mut cf = ConfFile::default();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0];
            let val = parts[1].trim();

            match key {
                "password" => {
                    cf.password = if val.is_empty() {
                        None
                    } else {
                        Some(val.to_string())
                    };
                }
                "auth" => {
                    cf.auth_enabled = Some(match val {
                        "on" => true,
                        "off" => false,
                        _ => {
                            return Err(format!(
                                "invalid auth value '{}' -- must be on or off",
                                val
                            ));
                        }
                    });
                }
                "maxconnections" => {
                    cf.max_connections = Some(
                        val.parse()
                            .map_err(|_| format!("invalid maxconnections in conf: {}", val))?,
                    );
                }
                "loglevel" => {
                    validate_loglevel(val)?;
                    cf.loglevel = Some(val.to_string());
                }
                "monitor_buffer" => {
                    cf.monitor_buffer = Some(
                        val.parse()
                            .map_err(|_| format!("invalid monitor_buffer in conf: {}", val))?,
                    );
                }
                // host and port are read-only — parsed from CLI only
                "host" | "port" => {}
                _ => {
                    println!("Warning: unknown config key '{}' in conf file", key);
                }
            }
        }

        Ok(cf)
    }
}

pub fn validate_loglevel(val: &str) -> Result<(), String> {
    match val {
        "debug" | "info" | "warn" | "error" => Ok(()),
        _ => Err(format!(
            "invalid loglevel '{}' — must be debug, info, warn or error",
            val
        )),
    }
}
