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

## Implementation Guidelines

When implementing container functionality, ensure compatibility with the OCI Runtime Specification. Key areas to focus on:
- Container lifecycle management
- Filesystem operations
- Process execution
- Resource management
- Security features (namespaces, cgroups, capabilities)
