# Supervit - gRPC Supervisor Client

A high-performance gRPC client for connecting actrix nodes to the centralized actrix-supervisor management platform.

## Features

- **gRPC Communication**: Uses HTTP/2 and Protocol Buffers for efficient bidirectional communication
- **Status Reporting**: Automatic periodic system metrics and service status reporting
- **Configuration Management**: Receive and apply configuration updates from supervisor
- **Tenant Operations**: Remote tenant CRUD operations
- **Health Checks**: Built-in health check and heartbeat mechanism

## Architecture

```
┌─────────────────┐         gRPC/HTTP2          ┌─────────────────┐
│  actrix-node    │◄──────────────────────────►│ actrix-supervisor│
│  (SupervitClient)│    Bidirectional Stream    │  (gRPC Server)  │
└─────────────────┘                             └─────────────────┘
```

## Configuration

Add to your `config.toml`:

```toml
[supervisor]
node_id = "actrix-node-01"
server_addr = "http://supervisor.example.com:50051"
connect_timeout_secs = 30
status_report_interval_secs = 60
health_check_interval_secs = 30
enable_tls = false
```

For TLS connections:

```toml
[supervisor]
node_id = "actrix-node-01"
server_addr = "https://supervisor.example.com:50051"
enable_tls = true
tls_domain = "supervisor.example.com"
```

## Usage

```rust
use supervit::{SupervitClient, SupervitConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = SupervitConfig {
        node_id: "actrix-01".to_string(),
        server_addr: "http://localhost:50051".to_string(),
        ..Default::default()
    };

    // Create and connect client
    let mut client = SupervitClient::new(config)?;
    client.connect().await?;

    // Start automatic status reporting
    client.start_status_reporting().await?;

    // Perform health check
    let health = client.health_check().await?;
    println!("Health check: {:?}", health);

    Ok(())
}
```

## Protocol

The communication protocol is defined in `proto/supervisor.proto`. The protocol supports:

### Services

- **StreamStatus**: Bidirectional streaming for continuous status reporting
- **UpdateConfig**: Configuration updates pushed from supervisor
- **ManageTenant**: Tenant management operations (CRUD)
- **HealthCheck**: Health checks and heartbeat

### Message Types

- `StatusReport`: System metrics and service status
- `StatusAck`: Acknowledgment of status reports
- `ConfigUpdateRequest/Response`: Configuration management
- `TenantOperation/Response`: Tenant CRUD operations
- `HealthCheckRequest/Response`: Health checks

## Building

The protocol is automatically compiled during build using `tonic-build`:

```bash
cargo build -p supervit
```

Generated code will be in `src/generated/`.

## Testing

```bash
cargo test -p supervit
```

## Comparison with WebSocket

| Feature | gRPC | WebSocket (Old) |
|---------|------|-----------------|
| Code Generation | ✅ Automatic | ❌ Manual |
| Type Safety | ✅ Compile-time | ⚠️ Runtime |
| Monitoring | ✅ Built-in | ❌ Custom |
| Load Balancing | ✅ Native | ⚠️ Complex |
| Debugging Tools | ✅ Rich (grpcurl, grpcui) | ⚠️ Limited |
| Development Time | ⚡ Fast | 🐢 Slow |

## License

Apache 2.0
