# astOS Backup System

## Overview
The astOS backup system provides a reliable and efficient way to create, manage, and restore system snapshots. Built on top of BTRFS, it offers atomic snapshots, incremental backups, and cloud storage integration.

## Features

### Core Features
- **Atomic Snapshots**: Create consistent, point-in-time snapshots of your system
- **Incremental Backups**: Save space by only storing changes between snapshots
- **Cloud Storage**: Built-in support for S3-compatible storage providers
- **Encryption**: Optional client-side encryption for sensitive data
- **CLI Interface**: Simple command-line tools for all operations
- **Verification**: Ensure backup integrity with built-in verification

## Installation

```bash
# Install the backup package
sudo pacman -S rast-backup
```

## Configuration

### Storage Configuration
Backup storage is configured in `/etc/rast/backup/config.toml`:

```toml
[storage]
# Local storage
local.path = "/var/lib/rast/backups"

# S3-compatible storage (optional)
[s3]
endpoint = "https://s3.example.com"
access_key_id = "your-access-key"
secret_access_key = "your-secret-key"
bucket = "your-bucket-name"
region = "us-east-1"
```

### Encryption
To enable encryption, add the following to your configuration:

```toml
[encryption]
enabled = true
# Path to encryption key file (generate with 'rast-backup generate-key')
key_path = "/etc/rast/backup/encryption.key"
```

## Usage

### Creating Backups

#### Full Backup
```bash
# Create a full backup
rast-backup create /path/to/backup --name "Full Backup"
```

#### Incremental Backup
```bash
# Create an incremental backup based on a previous backup
rast-backup create /path/to/backup --name "Incremental Backup" --parent <parent-backup-id>
```

### Managing Backups

#### List Backups
```bash
# List all backups
rast-backup list

# Show detailed information about a specific backup
rast-backup info <backup-id>
```

#### Verify Backups
```bash
# Verify backup integrity
rast-backup verify <backup-id>
```

### Restoring Data

#### Full Restore
```bash
# Restore a backup to a specified location
rast-backup restore <backup-id> /path/to/restore
```

#### Partial Restore
```bash
# Restore specific files or directories
rast-backup restore <backup-id> /path/to/restore --include "/path/to/file1" "/path/to/dir/*"
```

## Advanced Usage

### Scheduling Backups
Use systemd timers to schedule regular backups:

```ini
# /etc/systemd/system/rast-backup-daily.timer
[Unit]
Description=Daily backup

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

```ini
# /etc/systemd/system/rast-backup-daily.service
[Unit]
Description=Daily backup

[Service]
Type=oneshot
ExecStart=/usr/bin/rast-backup create / --name "Daily Backup" --tag daily
```

### Backup Retention
Configure retention policies in your configuration:

```toml
[retention]
# Keep daily backups for 7 days
daily = 7
# Keep weekly backups for 4 weeks
weekly = 4
# Keep monthly backups for 12 months
monthly = 12
```

## Troubleshooting

### Common Issues

#### Permission Denied
Ensure the backup user has the necessary permissions:
```bash
sudo usermod -aG rast-backup your-username
```

#### Out of Space
Check available space and clean up old backups:
```bash
rast-backup list --all
rast-backup delete <old-backup-id>
```

## Security Considerations

- Always store encryption keys securely
- Use IAM roles when possible instead of access keys
- Regularly test backup restoration
- Monitor backup success/failure notifications

## Contributing

Contributions are welcome! Please see our [contributing guidelines](CONTRIBUTING.md) for more information.

## License

astOS Backup System is licensed under the [GPL-3.0 License](LICENSE).
