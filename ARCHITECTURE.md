# rastOS Architecture

## Core Design Principles

1. **Container-First**: Everything runs in containers by default
2. **Immutable Infrastructure**: System state managed through declarative configurations
3. **Minimal Host OS**: Bare minimum components in the host system
4. **Security by Default**: Strong isolation between containers and host
5. **Optimized Kernel**: Minimal Linux kernel with container-specific optimizations
6. **Lightweight Runtimes**: Using crun/youki for efficient container execution

## System Architecture

```
+------------------------------------------------+
|                  rastOS                        |
|  +----------------------------------------+   |
|  |  Container Runtime (Native)            |   |
|  |  +---------------+  +---------------+  |   |
|  |  |  Container 1  |  |  Container 2  |  |   |
|  |  +---------------+  +---------------+  |   |
|  |                                        |   |
|  |  +--------------------------------+   |   |
|  |  |  OpenStack Integration         |   |   |
|  |  +--------------------------------+   |   |
+------------------------------------------------+
```

## Key Components

### 1. rastOS Core with Native Container Runtime
- **Integrated Runtime**:
  - Built directly into rastOS kernel and userspace
  - No separate container runtime daemon
  - Direct system calls for container operations
- **Resource Management**:
  - Unified process and container scheduling
  - Native cgroups v2 integration
  - Direct memory and CPU allocation
- **Networking**:
  - Kernel-level network namespace management
  - Integrated CNI plugins
  - Direct network policy enforcement
- **Storage**:
  - Built-in container storage drivers
  - Native snapshot and layer management
  - Direct filesystem integration
- **Security**:
  - Compile-time security hardening
  - Minimal attack surface
  - Direct integration with Linux security modules

### 2. Configuration & State Management
- **Vector Database Core**: Central repository for configuration and management data
  - **Implementation Options**:
    - **nano-vectordb-rs**: Lightweight, easy-to-hack vector database for ML pipelines and semantic search
    - **tinyvector**: Pure Rust implementation with fast in-memory indices
    - **SahomeDB**: SQLite-inspired embedded vector database with HNSW indexing

- **Key Advantages**:
  - **Semantic Search**: Find similar configurations and detect anomalies
  - **Scalability**: Efficiently handle growing numbers of devices and configurations
  - **Unified Data Model**: Store diverse configuration types with rich metadata
  - **Flexible Schema**: Adapt to new configuration types without rigid structures

- **Implementation Considerations**:
  - **Embedding Strategy**: Consistent representation of configuration items
  - **Update Management**: Handle rapid configuration changes efficiently
  - **Data Consistency**: Ensure reliable state across distributed systems
  - **Backup & Recovery**: Robust mechanisms for configuration persistence
  - **Security**: Access control and encryption for sensitive configuration data

- **Use Cases**:
  - Network configuration versioning and rollback
  - System state tracking and health monitoring
  - Policy and rule management with semantic relationships
  - Anomaly detection in operational metrics
  - Unified logging and telemetry analysis

### 3. Host OS Integration
- **Built-in Container Management**:
  - Direct container lifecycle management
  - Native process scheduling and resource allocation
  - Integrated security policies and isolation
  - Systemd integration for service management
- **Configuration Management**:
  - Direct access to system configuration
  - Unified logging and monitoring
  - Seamless updates and rollbacks
  - Integrated secret management
- **Health Monitoring**: Container health checks and auto-recovery
- **Service Discovery**: Internal DNS and service registration

### 4. Container Networking (CNI)
- **CNI Plugins**: Support for standard CNI plugins:
  - Bridge: Basic container networking
  - Host-local: IP address management
  - Loopback: Local loopback interface
  - Portmap: Container port mapping
  - Bandwidth: Traffic shaping
- **Network Policies**: Implementation of network policies for:
  - Pod-to-pod communication
  - Network segmentation
  - Ingress/egress filtering
  - DNS-based service discovery

### 5. OpenStack Integration
- **Nova Compute**: Integration with OpenStack compute
- **Neutron Networking**: Network management
- **Cinder Storage**: Persistent storage volumes
- **Heat**: Orchestration templates for deployment

## Kernel Configuration

### Minimal Kernel Features
- **Namespaces**: PID, network, mount, IPC, UTS, user, cgroup
- **Cgroups**: v2 with all controllers
- **Filesystems**: OverlayFS, BTRFS, tmpfs, proc, sysfs
- **Networking**: Basic TCP/IP, bridge, veth, iptables, nftables
- **Security**: Seccomp, AppArmor, capabilities
- **Containers**: All container-related features enabled

### Kernel Optimizations
- Disabled: Unused drivers, legacy filesystems, debugging symbols
- Optimized for: Fast boot, low memory footprint, container workloads
- Memory: KSM (Kernel Samepage Merging) enabled

## Data Flow

1. **Configuration Management**:
   - System and application configurations stored in vector database
   - Fast semantic search and retrieval of configuration parameters
   - Versioned configuration changes with rollback support

2. **Deployment**:
   - User defines container specs in YAML
   - Configurations are validated against vector database
   - rastOS deploys to OpenStack
   - Containers are scheduled and launched with vector-backed configs

2. **Networking**:
   - Each container gets a unique IP
   - Network policies control traffic between containers
   - External access via OpenStack load balancers

3. **Storage**:
   - Ephemeral storage per container
   - Persistent volumes via OpenStack Cinder
   - Volume snapshots for backup

## Security Model

- **Isolation**: Strong container isolation using:
  - Linux namespaces
  - cgroups v2 for resource constraints
  - Seccomp profiles
  - SELinux/AppArmor policies
- **RBAC**: Role-based access control with:
  - Fine-grained permissions
  - Service accounts for processes
  - Token-based authentication
- **Network Policies**: Fine-grained network controls including:
  - Network segmentation
  - Egress/ingress filtering
  - Network encryption (IPSec, WireGuard)
  - Service mesh integration
- **Audit Logging**: Comprehensive logging of:
  - Container lifecycle events
  - Network operations
  - Security-relevant operations
  - System calls and resource usage

## GUI Architecture

### Overview
rastOS features a lightweight, secure GUI architecture built on:
- **Smithay**: Custom Wayland compositor for minimal overhead
- **Tauri**: Secure web-based frontend framework
- **WebView**: System webview for rendering modern UIs

### Components

#### 1. Display Server (Smithay)
- Custom Wayland compositor
- Minimal resource footprint
- Secure process isolation
- Support for hardware acceleration

#### 2. Application Framework (Tauri)
- Web-based frontend with system webview
- Secure IPC between frontend and backend
- Plugin system for extensibility
- Support for multiple windows and dialogs

#### 3. Security Model
- Process isolation between applications
- Fine-grained permissions system
- Secure IPC channels
- Sandboxed applications

### Terminal Integration

#### Service Management (rinit)
- **TTY Management**
  - Virtual terminal creation and allocation
  - TTY device management
  - Session tracking
  - Resource allocation and cleanup

#### State Management
- **PTY/TTY State Persistence**
  - OS-managed state (line discipline, termios, window size)
  - Maintained while PTY remains open
  - Independent of attached UI layer (TUI/GUI)
  - Managed through service layer
- **Terminal Handling Layer**
  - Unix-focused terminal abstraction
  - Lightweight input/output processing
  - Terminal capabilities detection
  - Event handling
  - Minimal styling support

- **PTY/TTY Layer**
  - Rust-based PTY backend implementation
  - Cross-platform process spawning and management
  - Asynchronous I/O handling
  - Signal forwarding and process control

- **Frontend Terminal**
  - xterm.js integration
  - Terminal emulation
  - Input/output handling
  - Terminal theming

### Integration Points
1. **System Services**
   - Window management
   - Input handling
   - Display configuration
   - Session management
   - Terminal sessions

2. **Application Services**
   - File system access
   - Network access
   - Device integration
   - Notifications

## Testing with OpenStack

### Test Environment Setup
```bash
# Install OpenStack client
pip install python-openstackclient

# Configure OpenStack credentials
export OS_AUTH_URL=https://your-openstack:5000/v3
export OS_PROJECT_ID=your-project-id
export OS_USERNAME=your-username
export OS_PASSWORD=your-password
```

### Integration Tests
- **Unit Tests**: Test individual components
- **Integration Tests**: Test OpenStack integration
- **End-to-End Tests**: Full deployment tests

## Development Roadmap

### Phase 1: Core Container Runtime
- [ ] Basic container runtime
- [ ] Networking implementation
- [ ] Storage integration

### Phase 2: OpenStack Integration
- [ ] Nova compute integration
- [ ] Neutron networking
- [ ] Cinder storage

### Phase 3: Advanced Features
- [ ] Multi-tenant support
- [ ] Auto-scaling
- [ ] Monitoring and logging

## Performance Considerations

- **Resource Usage**: Minimal overhead
- **Startup Time**: Sub-second container startup
- **Density**: High container density per host

## Monitoring and Logging

- **Metrics**: Container and host metrics
- **Logging**: Centralized logging
- **Tracing**: Distributed tracing

## Backup and Recovery

- **Snapshots**: Container and volume snapshots
- **Backup**: Regular backups to object storage
- **Disaster Recovery**: Cross-region replication

## Scaling

- **Horizontal Scaling**: Add more nodes
- **Vertical Scaling**: Scale container resources
- **Auto-scaling**: Based on metrics
