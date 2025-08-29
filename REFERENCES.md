# rastOS Project References

## Core Documentation

1. **AGENTS.md** - Development plan and task tracking
   - Project overview and components
   - Development phases and milestones
   - Building and testing instructions

2. **ARCHITECTURE.md** - System architecture and design
   - Container-first architecture
   - OpenStack integration
   - Security model and data flow

## Technical References

3. **Open Container Initiative (OCI) Runtime Specification**
   - [Latest Documentation](https://docs.rs/oci-spec/latest/oci_spec/)
   - Container configuration and execution
   - Filesystem and process management
   - Linux-specific configurations
   - Windows-specific configurations
   - Solaris-specific configurations

4. **btrfsutil** - Btrfs filesystem utility bindings
   - [GitHub Repository](https://github.com/cezarmathe/btrfsutil-rs)
   - Btrfs subvolume and snapshot management
   - Filesystem operations and queries
   - Integration with container storage

5. **ALPM (Arch Linux Package Management)**
   - [crates.io](https://crates.io/crates/alpm)
   - [Documentation](https://docs.rs/alpm/)
   - Native Rust bindings for libalpm
   - Package database management
   - Dependency resolution and transaction handling

6. **rust-aur - AUR Client Library**
   - [GitHub Repository](https://github.com/hsdcc/rust-aur)
   - Pure Rust implementation of AUR client functionality
   - Direct Git operations with AUR packages
   - Supports package searching and metadata retrieval
   - Lightweight alternative to traditional AUR helpers

7. **Paru - AUR Helper**
   - [GitHub Repository](https://github.com/Morganamilo/paru)
   - [Documentation](https://github.com/Morganamilo/paru/wiki)
   - Feature-rich AUR helper with minimal dependencies
   - Can be used as a library or executed as a command
   - Supports AUR package searching, building, and installation
   - Handles AUR dependencies and PKGBUILD parsing

8. **qcow2-rs - QCOW2 Disk Image Format**
   - [crates.io](https://crates.io/crates/qcow2-rs)
   - Pure Rust implementation of QCOW2 (QEMU Copy-On-Write) format
   - Supports reading and writing QCOW2 disk images
   - Handles cluster allocation and management
   - Snapshot support
   - Used for virtual machine disk images

9. **rinit - Service Manager**
   - [GitHub Repository](https://github.com/rinit-org/rinit)
   - Modern, fast, and secure init system and service manager
   - Written in Rust for safety and performance
   - Lightweight alternative to systemd
   - Process supervision and service management
   - Dependency-based service startup

10. **CNI Bridge Plugin**
    - [GitHub Source](https://github.com/containernetworking/plugins/blob/main/plugins/main/bridge/bridge.go)
    - Reference implementation of the CNI bridge plugin
    - Handles container network interface creation
    - Manages Linux bridge configuration
    - Implements basic network connectivity between containers
    - Supports IP address management (IPAM)
    - Key component for container networking in CNI

11. **nano-vectordb-rs**
    - [GitHub Repository](https://github.com/VectorInstitute/nano-vectordb-rs)
    - Simple, easy-to-hack vector database in Rust
    - Lightweight with minimal abstractions
    - Ideal for ML pipelines and semantic search
    - Supports vector upsert, query, and metadata

12. **tinyvector**
    - [GitHub Repository](https://github.com/m1guelpf/tinyvector)
    - Tiny embedding database in pure Rust (~600 LOC)
    - Fast in-memory indices for quick queries
    - Easy to customize and extend
    - Suitable for small to medium datasets

13. **SahomeDB**
    - [Documentation](https://docs.rs/sahomedb)
    - SQLite-inspired embedded vector database
    - Uses Sled for persistence
    - Implements HNSW indexing for fast vector search
    - Supports incremental operations and metadata storage

## GUI Frameworks

### Smithay
- **Purpose**: Building Wayland compositors in Rust
- **Features**:
  - Modular architecture
  - Wayland protocol implementation
  - Custom compositor development
  - Hardware acceleration support
- **Repository**: [Smithay GitHub](https://github.com/Smithay/smithay)
- **Documentation**: [Smithay Docs](https://smithay.github.io/smithay/)

### Tauri
- **Purpose**: Build desktop applications with web technologies
- **Features**:
  - System webview integration
  - Secure IPC
  - Plugin system
  - Cross-platform support
- **Website**: [Tauri](https://tauri.app/)
- **Documentation**: [Tauri v2 Docs](https://v2.tauri.app/)
- **Sidecar Pattern**: [Sidecar Documentation](https://v2.tauri.app/develop/sidecar/)

### Integration Notes
- **Smithay + Tauri**:
  - Smithay provides the Wayland compositor foundation
  - Tauri apps run as Wayland clients
  - Secure communication via Wayland protocols
  - Shared security model

## PTY/TTY State Management

### Key Concepts
- **PTY State**: Line discipline, termios settings, window size, and process group
- **State Persistence**: Maintained by kernel while PTY remains open
- **Session Management**: Similar to terminal multiplexers (tmux/screen)

### Implementation Resources
- [Linux TTY Documentation](https://www.kernel.org/doc/html/v6.3/driver-api/tty/tty_struct.html)
- [Termion Raw Mode](https://docs.rs/termion/latest/termion/raw/index.html)
- [TTY/PTY Deep Dive](https://kevroletin.github.io/terminal/2022/01/27/how-tty-works-stty.html)
- [Terminal Applications in Rust](https://ticki.github.io/blog/making-terminal-applications-in-rust-with-termion/)

## Terminal Handling Libraries

### Comparison of Main Options

#### Crossterm
- **Platform Support**: Cross-platform (Linux, macOS, Windows)
- **Async Support**: Good (non-blocking I/O)
- **Features**:
  - Terminal manipulation
  - Styling and colors
  - Input handling
  - Raw mode support
  - Alternative screen support
- **Performance**: Good
- **Dependencies**: Moderate
- **Maintenance**: Actively developed
- **Use Case**: General-purpose terminal applications needing cross-platform support
- [GitHub](https://github.com/crossterm-rs/crossterm)

#### Termion
- **Platform Support**: Unix-like only (Linux, macOS)
- **Async Support**: Basic (via threads)
- **Features**:
  - Terminal manipulation
  - Styling and colors
  - Input handling
  - Raw mode support
- **Performance**: Excellent
- **Dependencies**: Minimal
- **Maintenance**: Less active
- **Use Case**: Unix-only applications where minimal dependencies are crucial
- [GitHub](https://github.com/redox-os/termion)

#### UI Frameworks (Ratatui/tui-rs)
- **Platform Support**: Depends on backend (Crossterm/Termion)
- **Async Support**: Varies by backend
- **Features**:
  - Widget-based UI
  - Layout management
  - Event handling
  - Theming
- **Performance**: Good (depends on backend)
- **Dependencies**: Higher (includes backend)
- **Maintenance**: Active
- **Use Case**: Rich terminal UIs with complex layouts
- [Ratatui GitHub](https://github.com/ratatui-org/ratatui)
- [tui-rs GitHub](https://github.com/fdehau/tui-rs)

### Selection Criteria for rastOS
1. **Unix-like support** (Linux/macOS)
2. **Performance** (low latency, efficient rendering)
3. **Minimal dependencies** (lightweight implementation)
4. **Feature completeness** (styling, input, etc.)
5. **Maintenance status** (stable, well-tested)

## Terminal Emulation

### Rust PTY Libraries

#### portable-pty
- **Pros**:
  - Cross-platform support (Unix/Windows)
  - Used in production by WezTerm
  - Actively maintained
  - Full-featured API
- **Cons**:
  - Larger dependency footprint
  - More complex API surface
- [GitHub](https://github.com/wez/wezterm/tree/main/pty)

#### ptyprocess
- **Pros**:
  - Simple, focused API
  - Lighter weight
  - Good for basic PTY needs
- **Cons**:
  - Fewer advanced features
  - Less battle-tested
- [Crates.io](https://crates.io/crates/ptyprocess)

#### alacritty_terminal
- **Pros**:
  - Highly optimized for performance
  - Battle-tested in Alacritty
  - Excellent terminal emulation
- **Cons**:
  - Tighter coupling with Alacritty
  - May be overkill for simple use cases
- [GitHub](https://github.com/alacritty/alacritty/tree/master/alacritty_terminal)

- **xterm.js**
  - Web-based terminal emulator
  - Full xterm compatibility
  - Extensible addons
  - [GitHub](https://github.com/xtermjs/xterm.js)

### Tauri Integration
- **tauri-plugin-shell**
  - Process management in Tauri
  - I/O streaming
  - [Documentation](https://v2.tauri.app/plugin/shell/)

## Implementation Guidelines

### Container and Network Management
When implementing container functionality, ensure compatibility with the OCI Runtime Specification and CNI networking. Key areas to focus on:
- Container lifecycle management
- Network interface configuration using CNI plugins
- Vector database integration for configuration management
- Filesystem operations
- Process execution
- Resource management
- Security features (namespaces, cgroups, capabilities, network policies)

### Vector Database Integration
When implementing the vector database for configuration management:
- **Data Modeling**:
  - Design effective embedding strategies for configuration items
  - Define metadata schema for operational context
  - Plan for versioning and historical tracking

- **Performance Optimization**:
  - Implement efficient indexing strategies
  - Design caching layers for frequently accessed data
  - Optimize vector dimensions and similarity metrics

- **Operational Excellence**:
  - Set up comprehensive monitoring and alerting
  - Implement robust backup and recovery procedures
  - Plan for horizontal scaling as the configuration space grows

## Vector Database Selection Criteria

When choosing a vector database implementation, consider:
1. **Performance Characteristics**:
   - Query latency and throughput
   - Resource utilization (CPU, memory, disk I/O)
   - Scaling behavior with growing configuration sets

2. **Operational Requirements**:
   - Persistence and durability guarantees
   - Backup and recovery capabilities
   - Monitoring and observability features

3. **Integration Factors**:
   - Rust API quality and documentation
   - Community support and maintenance status
   - License compatibility with project requirements

4. **Feature Set**:
   - Support for required index types (HNSW, IVF, etc.)
   - Metadata filtering capabilities
   - Built-in versioning and history support
