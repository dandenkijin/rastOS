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

## Implementation Guidelines

When implementing container functionality, ensure compatibility with the OCI Runtime Specification. Key areas to focus on:
- Container lifecycle management
- Filesystem operations
- Process execution
- Resource management
- Security features (namespaces, cgroups, capabilities)
