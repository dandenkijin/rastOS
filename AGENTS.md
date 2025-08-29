# rastOS Development Plan

## Project Overview
rastOS is a modern, safe implementation of a Linux distribution with snapshot capabilities, built entirely in Rust. The project features a modular architecture with a focus on security, performance, and developer experience.

## Core Components

### 1. CLI Interface (`src/bin/`)
- [x] Command-line interface for kernel building
- [x] Support for multiple build profiles
- [x] Parallel build support
- [x] Debug output options
- [x] Custom configuration support

### 2. Installer Module (`src/installer/`)
- Virtual disk image creation and management
  - [ ] QCOW2 image support (using qcow2-rs)
  - [ ] Raw disk image support
  - [ ] Filesystem management within images
- Base system installation
  - [ ] Package installation
  - [ ] System configuration
  - [ ] Bootloader setup (if applicable)
- User configuration
  - [ ] Default user creation
  - [ ] SSH key management
  - [ ] System-wide configurations

### 3. Package Management (`src/package/`)
- [x] Declarative package lists (TOML format)
- [x] Support for official and AUR packages
- [ ] Transaction management
- [ ] Dependency resolution
- [ ] Rollback support

### 4. Kernel & Runtime (`src/kernel/`) - Arch Linux Compatible
- [x] Kernel builder implementation
  - [x] Basic build system integration
  - [x] Configuration management
  - [x] Parallel compilation support
  - [x] Progress reporting
- [x] Kernel configuration profiles
  - [x] Container-optimized profile
  - [x] Development profile (with debugging)
  - [x] Production profile (with hardening)
- [x] Testing
  - [x] Test with mainline kernel sources (Verified on Arch Linux)
  - [x] Verify container runtime compatibility (Basic verification complete)
  - [ ] Performance benchmarking
  - [x] Arch Linux compatibility verified
- [ ] Container runtime integration
  - [x] Basic OCI runtime interface
  - [x] Container state management
  - [x] Basic container lifecycle operations
  - [ ] Integrate crun (primary)
  - [ ] Integrate youki (Rust alternative)
  - [ ] Resource limits and isolation
  - [ ] Network namespace support

### 4. Snapshot System (`src/snapshot/`)
- [ ] BTRFS snapshot management
- [ ] Snapshot tree structure
- Rollback functionality
- Snapshot deployment

### 4. System Utilities (`src/system/`)
- System information gathering
- Service management with rinit
  - [ ] rinit service files and configuration
  - [ ] Service supervision and monitoring
  - [ ] Dependency-based service startup
  - [ ] Logging integration
  - [ ] TTY device management
    - [ ] Virtual terminal allocation
    - [ ] TTY session tracking
    - [ ] Resource cleanup on session end
- **Networking**:
  - [ ] Native network namespace management
  - [ ] Integrated CNI plugin framework
  - [ ] Network policy enforcement engine
  - [ ] Service discovery and load balancing
  - [ ] Network security and encryption
- User management

## Vector Database Implementation

### Core Components
- **Storage Layer**:
  - [ ] Implement vector storage backend (nano-vectordb-rs/tinyvector/SahomeDB)
  - [ ] Design embedding strategy for configuration items
  - [ ] Implement versioning and rollback support
  - [ ] Set up backup and recovery mechanisms

### Integration Points
- **Configuration Management**:
  - [ ] Store and retrieve system configurations with semantic search
  - [ ] Handle network policies and rules with relationship mapping
  - [ ] Integrate with operational metrics and telemetry
  - [ ] Implement access control and encryption

### Performance Optimization
- [ ] Benchmark and optimize query performance
  - [ ] Test with different embedding dimensions
  - [ ] Evaluate indexing strategies (HNSW, IVF, etc.)
  - [ ] Measure impact of vector similarity thresholds
- [ ] Implement caching strategies
  - [ ] In-memory cache for hot configurations
  - [ ] Query result caching
  - [ ] Vector index optimization

### Operational Considerations
- [ ] Monitoring and alerting
  - [ ] Track query performance metrics
  - [ ] Monitor resource usage
  - [ ] Set up alerts for anomalies
- [ ] Data consistency
  - [ ] Implement distributed locking
  - [ ] Handle concurrent updates
  - [ ] Ensure data durability

## Development Phases

### Phase 1: Core Infrastructure
- [x] Project setup and structure
- [x] Basic error handling and logging
- [ ] Configuration management
- [x] Filesystem operations
  - [x] File operations (create, read, write, copy, move, delete)
  - [x] Directory operations (create, list, remove)
  - [x] Filesystem utilities (temp files, glob patterns)
  - [x] BTRFS subvolume management
  - [x] Basic snapshot operations

### Phase 2: Vector Database Integration
- [ ] Evaluate and select vector database implementation
  - [ ] nano-vectordb-rs for lightweight ML pipelines
  - [ ] tinyvector for in-memory performance
  - [ ] SahomeDB for HNSW indexing and persistence
- [ ] Implement core storage interface
  - [ ] CRUD operations for configurations
  - [ ] Versioning and history tracking
  - [ ] Backup and restore functionality
- [ ] Integration with existing systems
  - [ ] Network configuration management
  - [ ] System state tracking
  - [ ] Policy and rule storage

### Phase 3: Package Management
- [ ] ALPM integration (libalpm)
  - [ ] Initialize ALPM handle
  - [ ] Configure repositories
  - [ ] Package database operations
- [ ] Package installation/removal
  - [ ] Transaction handling
  - [ ] Conflict resolution
  - [ ] Callback implementation
- [ ] Dependency resolution
  - [ ] Dependency calculation
  - [ ] Transaction dependency resolution
- [ ] AUR support
  - [ ] Primary: rust-aur integration
    - [ ] Direct Git operations with AUR packages
    - [ ] Package searching and metadata retrieval
    - [ ] Build and install workflows
    - [ ] Dependency resolution for AUR packages
  - [ ] Fallback: Paru integration
    - [ ] Library integration for AUR operations
    - [ ] Build and install workflows
    - [ ] Dependency resolution for AUR packages
  - [ ] Native implementation (optional, future)
    - [ ] PKGBUILD parsing
    - [ ] AUR API integration
    - [ ] Build system integration

### Phase 3: Snapshot System
- [x] BTRFS snapshot operations (basic implementation)
- [ ] Snapshot tree structure
- [ ] Rollback functionality
- [ ] Snapshot deployment

### Phase 4: Installer
- [ ] Disk partitioning
- [ ] Filesystem creation
- [ ] Base system installation
- [ ] Bootloader configuration

### Phase 5: Network Integration
- [ ] Core Networking
  - [ ] Implement native network namespace management
  - [ ] Integrate CNI plugins for container networking
  - [ ] Set up network policy enforcement
  - [ ] Configure service discovery
  - [ ] Implement load balancing
- [ ] Security & Performance
  - [ ] Network encryption (TLS, WireGuard)
  - [ ] Traffic shaping and QoS
  - [ ] Network monitoring and metrics
  - [ ] Bandwidth management

### Phase 6: Orchestration Layer
- [ ] Container orchestration
  - [ ] Multi-container application support
  - [ ] Service discovery and load balancing
  - [ ] Health monitoring and self-healing
  - [ ] Rolling updates and rollbacks
  - [ ] Configuration and secret management
- [ ] Scheduling
  - [ ] Resource-aware scheduling
  - [ ] Affinity/anti-affinity rules
  - [ ] Taints and tolerations
  - [ ] Custom scheduling policies

### Phase 7: GUI Development
- [ ] **Core Display Server (Smithay)**
  - [ ] Basic Wayland compositor
  - [ ] Window management
  - [ ] Input handling
  - [ ] Hardware acceleration
  - [ ] Multi-monitor support

- [ ] **Application Framework (Tauri)**
  - [ ] Tauri core integration
  - [ ] System webview setup
  - [ ] Secure IPC channels
  - [ ] Plugin system
  - [ ] Application lifecycle management

- [ ] **Terminal Integration**
  - [ ] PTY State Management
    - [ ] Implement PTY state preservation
    - [ ] Handle TUI to GUI transitions
    - [ ] Manage terminal session persistence
    - [ ] Implement terminal multiplexing support
  - [ ] Evaluate terminal handling libraries (Unix-focused):
    - [ ] `Termion` (Unix-only, lightweight)
      - [ ] Performance benchmarks
      - [ ] Feature set evaluation
      - [ ] UI framework compatibility (Ratatui/tui-rs)
    - [ ] `tui-rs` with Termion backend
      - [ ] Widget system evaluation
      - [ ] Performance impact
      - [ ] Ease of customization
  - [ ] Evaluate PTY backend options:
    - [ ] `portable-pty` (cross-platform, used by WezTerm)
    - [ ] `ptyprocess` (simpler API, good for basic use cases)
    - [ ] `alacritty_terminal` (high-performance, battle-tested in Alacritty)
  - [ ] Implement selected PTY backend
    - [ ] Process spawning and management
    - [ ] Terminal I/O handling
    - [ ] Signal forwarding
  - [ ] xterm.js frontend
    - [ ] Terminal emulation
    - [ ] Input/output handling
    - [ ] Terminal theming
  - [ ] Tauri integration
    - [ ] WebSocket communication
    - [ ] Terminal window management
    - [ ] Multiple terminal sessions

- [ ] **System Integration**
  - [ ] Session management
  - [ ] Display configuration
  - [ ] Input method support
  - [ ] Accessibility features
  - [ ] Power management

### Phase 8: CLI and Tools
- [ ] Command-line interface
- [ ] Interactive TUI
- [ ] System maintenance tools
- [ ] Documentation

## Networking Implementation

### Core Networking Components
- **Network Stack**:
  - Native Linux networking stack integration
  - Custom network drivers for optimized performance
  - Support for multiple network interfaces
  - IPv4/IPv6 dual-stack support

### Network Plugins (CNI)
- **Standard Plugins**:
  - **Bridge**: Container networking with Linux bridge
  - **Host-local**: Efficient IP address management (IPAM)
  - **Portmap**: Seamless host port mapping
  - **Bandwidth**: Traffic shaping and QoS controls
  - **Tuning**: Network interface optimization

### Advanced Networking Features
- **Network Policies**:
  - Fine-grained traffic control
  - Namespace isolation
  - Egress/ingress filtering
  - DNS-based service discovery
  - Network segmentation

### Security & Performance
- **Encryption**:
  - mTLS for service-to-service communication
  - WireGuard for secure tunnels
  - Certificate management
- **Monitoring**:
  - Real-time traffic analysis
  - Flow monitoring
  - Anomaly detection
  - Performance metrics collection

### Service Discovery
- **CoreDNS** integration for service discovery
- **DNS-based** service resolution
- **Headless services** support
- **ExternalName** service type

## GUI Development Guidelines

### Architecture
- **Display Server**: Smithay-based Wayland compositor
- **UI Framework**: Tauri with system webview
- **Frontend**: Web technologies (HTML/CSS/JavaScript/TypeScript)
- **Backend**: Rust services with Tauri API

### Security Considerations
- All applications run in isolated Wayland clients
- Fine-grained permissions for system access
- Secure IPC between frontend and backend
- Sandboxed webview processes
- Memory-safe Rust backend

## Building and Testing

### Prerequisites
- Rust 1.70 or later
- Cargo
- Development libraries for BTRFS and Wayland
- System packages:
  - btrfs-progs
  - util-linux
  - wayland-protocols
  - webkit2gtk (for Tauri)
  - libgtk-3-dev (for system webview)

### Building
```bash
# Build in release mode
cargo build --release

# Build with all features
cargo build --all-features
```

### Testing
```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

## Contributing
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License
GNU General Public License v3.0

## Contact
Project Link: [https://github.com/yourusername/astos-rs](https://github.com/yourusername/astos-rs)
