# Redis Clone in Rust - JEDIS

A high-performance Redis clone implementation written in Rust, featuring comprehensive Redis protocol support, master-slave replication, and persistent storage.

There are few features which will be added in future all of them haven't been integrated yet 

## Features

### Core Redis Commands
- **String Operations**: `SET`, `GET`, `DEL`, `TTL`, `INCR`, `DECR`, `INCRBY`, `DECRBY`
- **Hash Operation**: `HSET`, `HGET`, `HDEL` , `HTTL`
- **List Operations**: <!---`LPUSH`, `RPUSH`, `LPOP`, `LRANGE`, `LLEN`, `BLPOP` --> coming soon
- **Stream Operations**: <!---`XADD`, `XRANGE`, `XREAD` --> coming soon
- **Sorted Set Operations**: <!---`ZADD`, `ZRANGE`, `ZRANK`, `ZCARD`, `ZSCORE`, `ZREM` --> coming soon
- **Server Operations**: `PING`, `ECHO`, `INFO`, `CONFIG`, `KEYS`, `SAVE`, `TYPE`, `SHUTDOWN`, `MONITER`
- **Pub/Sub**: <!---`SUBSCRIBE`, `UNSUBSCRIBE`, `PUBLISH` --> coming soon
- **Transactions**: <!---`MULTI`, `EXEC`, `DISCARD` --> coming soon

### Advanced Features
<!--- **Master-Slave Replication**: Full replication support with `REPLCONF`, `PSYNC`-->
<!--- **RDB Persistence**: Binary file format for data persistence-->
<!--- **Blocking Operations**: Non-blocking I/O with support for blocking list operations-->
- **SELF RESP CLI SUPPORT**: Have it own RESP protocol CLI
- **Expiration Support**: TTL functionality for keys
- **Asynchronous Architecture**: Built on Tokio for high concurrency

## Architecture

### Core Components

- **`server.rs`**: Main entry point with TCP server and connection handling
- **`commands/mod.rs`**: Implementation of all Redis commands
<!--- **`utils/mod.rs`**: Utility functions, data structures, and protocol encoding/decoding-->
<!--- **Integration Tests**: Comprehensive test suite-->

### Data Structures
```rust
pub enum RedisValue {
    String(String),
    Hash(HashMap<String, String>),
    // List(Vec<String>),
}

pub struct Entry {
    pub value: RedisValue,
    pub expires_at: Option<Instant>,
}
```

## Getting Started

### Prerequisites

- Rust 1.90+ 
- Tokio runtime
- Dependencies: `rand`, `tokio`, `clap`

### Installation

```
git clone <repository-url>
cd redis-clone
cargo build --release
```

### Usage

#### Start as Master (default)
```cargo run -- --port 6379```


#### Start as Replica
```cargo run -- --port 6380 --replicaof "127.0.0 6379"```


#### With Persistence
```cargo run -- --port 6379 --dir /tmp/redis-data --dbfilename dump.rdb```


### Command Line Options

- `--port`: Port number to listen on (default: 6379)
<!--- `--dir`: Directory for RDB files
- `--dbfilename`: Name of the RDB file
- `--replicaof`: Master server address for replication (format: "host port")-->

## Protocol Support

The implementation follows the Redis Serialization Protocol (RESP):

- **Simple Strings**: `+OK\r\n`
- **Errors**: `-Error message\r\n`
- **Integers**: `:1000\r\n`
- **Bulk Strings**: `$6\r\nfoobar\r\n`
- **Arrays**: `*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n`

## Replication

The server supports master-slave replication with:

<!--- Automatic replica discovery and registration
- Command propagation from master to replicas
- Replica acknowledgment tracking
- RDB file synchronization-->

COMING SOON ...

### Replication Flow
1. Replica connects to master
2. Handshake with `PING`, `REPLCONF` commands
3. Full synchronization with `PSYNC`
4. Continuous command replication

## Persistence

### RDB Format Support
- Header with Redis version information
- Metadata sections for configuration
- Database sections with key-value pairs
- Timestamp support for key expiration
- CRC64 checksum for data integrity

### File Operations
- `SAVE`: Create RDB snapshot
- Automatic loading on startup
- Support for multiple databases

## Testing

Run the integration tests:
```cargo test```

The test suite (currently)includes:
- Basic PING/PONG functionality
- Connection handling
- Command parsing and execution

## Performance Features

- **Async/Await**: Non-blocking I/O operations
- **Connection Pooling**: Multiple concurrent client connections
- **Memory Efficient**: Zero-copy string parsing where possible
- **Lock-Free Operations**: Minimized contention with Arc<Mutex<>> patterns

## Supported Clients

coming soon...

## Limitations

- Partial Redis command set (continuously expanding)
- Single-threaded per connection (but multi-connection)
- Limited clustering support
- Basic pub/sub implementation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License.

## Acknowledgments

- Redis Labs for the original Redis implementation and protocol specification
- Tokio team for the excellent async runtime
- Rust community for the robust ecosystem
