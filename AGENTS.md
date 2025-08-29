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
- Network configuration
- User management

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

### Phase 2: Package Management
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

### Phase 5: Container Integration
- [ ] OCI runtime integration
- [ ] Container lifecycle management
- [ ] Resource isolation
- [ ] Network configuration

### Phase 5: CLI and Tools
- [ ] Command-line interface
- [ ] Interactive TUI
- [ ] System maintenance tools
- [ ] Documentation

## Building and Testing

### Prerequisites
- Rust 1.70 or later
- Cargo
- Development libraries for BTRFS
- System packages: btrfs-progs, util-linux

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
