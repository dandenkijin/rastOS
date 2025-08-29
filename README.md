# rastOS (Rust Arch Snapshot Tree OS)
### The next evolution of astOS, reimagined in Rust with enhanced security and performance

![rastos-logo](logo.jpg)

---

## Table of Contents
* [What is rastOS?](#what-is-rastos)
* [astOS vs rastOS](#astos-vs-rastos)
* [Key Features](#key-features)
* [Advanced Features](#advanced-features)
  * [Bare Metal Implementation](#bare-metal-implementation)
  * [Cloud Storage Backups](#cloud-storage-backups)
  * [LLM-Based Scheduling](#llm-based-scheduling)
* [astOS Documentation](#astos-documentation)
  * [Installation](#installation)
  * [Post Installation](#post-installation-setup)
  * [Snapshot Management](#snapshot-management)
  * [Package Management](#package-management)
* [Additional Documentation](#additional-documentation)
  * [Updating pacman keys](#fixing-pacman-corrupt-packages--key-issues)
  * [Persistent Configuration](#saving-configuration-changes-made-in-etc)
  * [Dual Boot Setup](#dual-boot)
  * [Updating ast/rast](#updating-ast-itself)
  * [Debugging](#debugging-ast)
  * [AUR Setup](#aur-setup)
* [Architecture](#architecture)
* [Development Status](#development-status)
* [Contributing](#contributing)
* [Community](#community)
* [Known Issues](#known-bugs)

---

## What is rastOS?

rastOS is the next evolution of astOS, completely reimagined and rewritten in Rust. It maintains all the core principles of astOS while introducing modern container-first architecture, enhanced security features, and improved performance through Rust's memory safety guarantees.

## astOS vs rastOS

### What stays the same:
- Immutable root filesystem
- BTRFS snapshot-based system management
- Arch Linux compatibility
- Package management through pacman
- Snapshot deployment workflow
- Support for AUR packages

### What's new in rastOS:
- **Rust-Powered Core**: Entirely written in Rust for memory safety and performance
- **Container-First Architecture**: Built from the ground up for containerized workloads
- **Enhanced Security**: Modern security practices and minimal attack surface
- **Vector Database**: Advanced configuration management with semantic search
- **Dual Runtime Support**: Native and virtualized (RustVMM) execution modes
- **Modern Init System**: Custom rinit implementation for better process management

## Key Features

- **Immutable Infrastructure**: System state managed through declarative configurations
- **Minimal Host OS**: Bare minimum components in the host system for improved security
- **Optimized Kernel**: Custom Linux kernel with container-specific optimizations
- **Vector Database Integration**: Advanced configuration management with semantic search capabilities
- **Secure by Default**: Built with security best practices and minimal attack surface
- **Backward Compatibility**: Maintains compatibility with existing astOS workflows

## astOS Documentation

### Installation

astOS is installed from the official Arch Linux live iso available on [https://archlinux.org/](https://archlinux.org/). The installation process remains the same as before, with the rastOS improvements being implemented under the hood.

```bash
# On an Arch Linux live environment
pacman -Sy git
git clone "https://github.com/dandenkijin/rastOS"  
cd rastOS

# Partition and format drive (example for EFI)
lsblk  # Find your drive name
cfdisk /dev/***  # Create partitions
mkfs.btrfs /dev/***  # Create btrfs filesystem

# Run installer
python3 main.py /dev/<partition> /dev/<drive> /dev/<efi_part>
```

### Post Installation Setup

Post-installation setup follows the same workflow as astOS, with additional configuration options for the new rastOS features.

### Snapshot Management

rastOS maintains the same snapshot management interface as astOS, with enhanced reliability and performance:

```bash
# Show filesystem tree
rast tree

# Create new snapshot
rast clone <base_snapshot>

# Deploy snapshot
rast deploy <snapshot>
```

### Package Management

#### astOS (Legacy)
```bash
# Install packages
pacman -S package_name

# Update system
pacman -Syu

# Search for packages
pacman -Ss search_term
```

#### rastOS (Modern Package Management)
rastOS offers both traditional pacman compatibility and next-generation natural language package management:

```bash
# Traditional pacman-compatible commands (fully supported)
rast -S package_name           # Install package
rast -Syu                      # Full system update
rast -Ss search_term           # Search for packages

# New natural language interface
rast "install the latest version of firefox"
rast "update all packages with security fixes"
rast "what python packages do I have installed?"

# Advanced package management
rast pkg install --check-conflicts package_name
rast pkg update --test         # Simulate update
rast pkg clean --unused-deps   # Remove unused dependencies

# Key advantages:
# 100% pacman command-line compatibility
# Natural language understanding for intuitive operations
# Vector database-powered semantic search
# Transactional updates with automatic rollback
# Enhanced dependency resolution and conflict detection

## Additional Documentation

### Fixing Package and Key Issues

#### astOS (Legacy)
```bash
pacman-key --init
pacman-key --populate archlinux
pacman -S archlinux-keyring
pacman -Syu
```

#### rastOS (New)
```bash
# Traditional way (fully supported)
pacman -S archlinux-keyring
pacman -Syu

# New way
rast "fix broken packages and update system"
# or
rast pkg fix-keys
rast pkg update
```

### Saving configuration changes made in /etc

```bash
rast chroot <snapshot>
# Make your changes in /etc
exit 0
rast deploy <snapshot>
```

### Dual Boot

rastOS maintains compatibility with dual-boot setups. Follow the same procedure as with astOS, ensuring you have the `os-prober` package installed.

### Updating ast/rast

```bash
# Update the rastOS core
rast self-update

# Update the base system
rast base-update
```

### Debugging ast

```bash
# Enable debug logging
RAST_DEBUG=1 rast <command>

# Show detailed system information
rast status
```

### AUR Setup

```bash
# Install an AUR helper (e.g., yay)
git clone https://aur.archlinux.org/yay.git
cd yay
makepkg -si

# Install AUR packages
yay -S <package>
```

## Advanced Features

### Bare Metal Implementation

rastOS supports bare metal deployment with the following features:

- **Limine Bootloader** for modern hardware support
- Rust-based kernel with safe abstractions
- Memory management and hardware abstraction
- Device driver framework

For implementation details, see [Bare Metal Documentation](../docs/BAREMETAL.md).

### Cloud Storage Backups

rastOS includes robust cloud storage backup capabilities:

- Support for multiple cloud providers (S3, GCS, Azure Blob)
- Encrypted, incremental backups
- Configurable retention policies
- Scheduled and on-demand backups

For setup and usage, see [Cloud Backup Documentation](../docs/CLOUD_BACKUP.md).

### LLM-Based Scheduling

rastOS introduces intelligent, LLM-powered scheduling for containerized workloads. This advanced feature provides:

- Dynamic, context-aware workload distribution
- Intelligent resource allocation based on real-time metrics
- Semantic understanding of workload requirements
- Automatic optimization for AI/ML workloads

For detailed information, see [LLM Scheduling Documentation](../docs/LLM_SCHEDULING.md).

## Architecture

rastOS builds upon the astOS architecture with these key improvements:

1. **Core System**
   - Custom init system (rinit) for better process management
   - Container runtime integration (crun/youki)
   - Enhanced BTRFS snapshot management

2. **Vector Database**
   - Centralized configuration management
   - Semantic search capabilities
   - Versioned configurations with rollback support

3. **Networking**
   - Native network namespace management
   - CNI plugin support
   - Advanced network policies

For detailed architecture documentation, see [ARCHITECTURE.md](https://github.com/dandenkijin/rastos-rs/blob/main/ARCHITECTURE.md).

## Development Status

rastOS is currently in active development. The project is being built with the following principles:

- Maintain backward compatibility with existing astOS installations
- Gradually introduce new features while ensuring stability
- Focus on security and performance improvements
- Community-driven development

See [AGENTS.md](https://github.com/dandenkijin/rastos-rs/blob/main/AGENTS.md) for the detailed development roadmap.

## Contributing

We welcome contributions from the community! Please see our [Contributing Guide](CONTRIBUTING.md) for details on how to get started.

## Community

- [GitHub Discussions](https://github.com/dandenkijin/rastos-rs/discussions) - For questions and discussions
- [Issue Tracker](https://github.com/dandenkijin/rastos-rs/issues) - To report bugs and request features
- [Matrix](https://matrix.to/#/#rastos:matrix.org) - Real-time chat (coming soon)

## Known Bugs

For known issues and their status, please check the [GitHub Issues](https://github.com/dandenkijin/rastos-rs/issues) page.

* Security
  * Even if running an application with eleveted permissions, it cannot replace system libraries with malicious versions
* Stability and reliability
  * Due to the system being mounted as read only, it's not possible to accidentally overwrite system files
  * If the system runs into issues, you can easily rollback the last working snapshot within minutes
  * Atomic updates - Updating your system all at once is more reliable
  * Thanks to the snapshot feature, astOS can ship cutting edge software without becoming unstable
  * astOS needs little maintenance, as it has a built in fully automatic update tool that creates snapshots before updates and automatically checks if the system upgraded properly before deploying the new snapshot
* Configurability
  * With the snapshots organised into a tree, you can easily have multiple different configurations of your software available, with varying packages, without any interference
  * For example: you can have a single Gnome desktop installed and then have 2 snapshots on top - one with your video games, with the newest kernel and drivers, and the other for work, with the LTS kernel and more stable software, you can then easily switch between these depending on what you're trying to do
  * You can also easily try out software without having to worry about breaking your system or polluting it with unnecessary files, for example you can try out a new desktop environment in a snapshot and then delete the snapshot after, without modifying your main system at all
  * This can also be used for multi-user systems, where each user has a completely separate system with different software, and yet they can share certain packages such as kernels and drivers
  * astOS allows you to install software by chrooting into snapshots, therefore you can use software such as the AUR to install additional packages
  * astOS is, just like Arch, very customizable, you can choose exactly which software you want to use

* Thanks to it's reliabilty and automatic upgrades, astOS is well suitable for single use or embedded devices
* It also makes for a good workstation or general use distribution utilizing development containers and flatpak for desktop applications 

---
## astOS compared to other similar distributions
* **NixOS** - compared to nixOS, astOS is a more traditional system with how it's setup and maintained. While nixOS is entirely configured using the Nix programming language, astOS uses Arch's pacman package manager. astOS consumes less storage, and configuring your system is faster and easier (less reproducible however), it also gives you more customization options. astOS is FHS compliant, ensuring proper software compatibility.
  * astOS allows declarative configuration using Ansible, for somewhat similar functionality to NixOS
* **Fedora Silverblue/Kinoite** - astOS is more customizable, but does require more manual setup. astOS supports dual boot, unlike Silverblue.
* **OpenSUSE MicroOS** - astOS is a more customizable system, but once again requires a bit more manual setup. MicroOS works similarly in the way it utilizes btrfs snapshots. astOS has an official KDE install, but also supports other desktop environments, while MicroOS only properly supports Gnome. astOS supports dual boot, as well as live-patching the system and installing packages without reboot.

---
## Installation
* astOS is installed from the official Arch Linux live iso available on [https://archlinux.org/](https://archlinux.org)
* If you run into issues installing packages during installation, make sure you're using the newest arch iso, and if needed update the pacman keyring
* You need an internet connection to install astOS
* Currently astOS ships 4 installation profiles, one for minimal installs and two for desktop, one with the Gnome desktop environment, one with KDE Plasma, and one with MATE, but support for more DE's will be added
* The installation script is easily configurable and adjusted for your needs (but it works just fine without any modifications)

Install git first - this will allow us to download the install script

```
pacman -Sy git
```
Clone repository

```
git clone "https://github.com/lambdanil/astOS"  
cd astOS  
```
Partition and format drive

* If installing on a BIOS system, use a dos (MBR) partition table
* On EFI you can use GPT
* The EFI partition has to be formatted to FAT32 before running the installer (```mkfs.fat -F32 /dev/<part>```)

```
lsblk  # Find your drive name
cfdisk /dev/*** # Format drive, make sure to add an EFI partition, if using BIOS leave 2M free space before first partition  
mkfs.btrfs /dev/*** # Create a btrfs filesystem, don't skip this step!
```
Run installer

```
python3 main.py /dev/<partition> /dev/<drive> /dev/<efi part> # Skip the EFI partition if installing in BIOS mode
```

## Post installation setup
* Post installation setup is not necessary if you install one of the desktop editions (Gnome or KDE)
* A lot of information for how to handle post-install setup is available on the [ArchWiki page](https://wiki.archlinux.org/title/general_recommendations) 
* Here is a small example setup procedure:
  * Start by creating a new snapshot from `base` using ```ast clone 0```
  * Chroot inside this new snapshot (```ast chroot <snapshot>```) and begin setup
    * Start by adding a new user account: ```useradd username```
    * Set the user password ```passwd username```
    * Now set a new password for root user ```passwd root```
    * Now you can install additional packages (desktop environments, container technologies, flatpak) using pacman
    * Once done, exit the chroot with ```exit 0```
    * Then you can deploy it with ```ast deploy <snapshot>```

## Additional documentation
* It is advised to refer to the [Arch wiki](https://wiki.archlinux.org/) for documentation not part of this project
* Report issues/bugs on the [Github issues page](https://github.com/lambdanil/astOS/issues)
* **HINT: you can use `ast help` to get a quick cheatsheet of all available commands**

#### Base snapshot
* The snapshot ```0``` is reserved for the base system snapshot, it cannot be changed and can only be updated using ```ast base-update```

## Snapshot Management

#### Show filesystem tree

```
ast tree
```

* The output can look for example like this:

```
root - root
├── 0 - base snapshot
└── 1 - multiuser system
    └── 4 - applications
        ├── 6 - MATE full desktop
        └── 2*- Plasma full desktop
```
* The asterisk shows which snapshot is currently selected as default

* You can also get only the number of the currently booted snapshot with

```
ast current
```
#### Add descritption to snapshot
* Snapshots allow you to add a description to them for easier identification

```
ast desc <snapshot> <description>
```
#### Delete a tree
* This removes the tree and all it's branches

```
ast del <tree>
```
#### Custom boot configuration
* If you need to use a custom grub configuration, chroot into a snapshot and edit ```/etc/default/grub```, then deploy the snapshot and reboot

#### chroot into snapshot 
* Once inside the chroot the OS behaves like regular Arch, so you can install and remove packages using pacman or similar
* Do not run ast from inside a chroot, it could cause damage to the system, there is a failsafe in place, which can be bypassed with ```--chroot``` if you really need to (not recommended)  
* The chroot has to be exited properly with ```exit 0```, otherwise the changes made will not be saved
* To discard the changes made, use ```exit 1``` instead
* If you don't exit chroot the "clean" way with ```exit 0```, it's recommended to run ```ast tmp``` to clear temporary files left behind


```
ast chroot <snapshot>
```

* You can enter an unlocked shell inside the current booted snapshot with

```
ast live-chroot
```

* The changes made to live session are not saved on new deployments 

#### Other chroot options

* Runs a specified command inside snapshot

```
ast run <snapshot> <command>
```

* Runs a specified command inside snapshot and all it's branches

```
ast tree-run <tree> <command>
```

#### Clone snapshot
* This clones the snapshot as a new tree

```
ast clone <snapshot>
```

#### Clone a tree recursively  
* This clones an entire tree recursively

```
ast clone-tree <snapshot>
```

#### Create new tree branch

* Adds a new branch to specified snapshot

```
ast branch <snapshot to branch from>
```
#### Clone snapshot under same parent

```
ast cbranch <snapshot>
```
#### Clone snapshot under specified parent

* Make sure to sync the tree after

```
ast ubranch <parent> <snapshot>
```
#### Create new base tree

```
ast new
```
#### Deploy snapshot  

* Reboot to  boot into the new snapshot after deploying

```
ast deploy <snapshot>  
```

#### Update base which new snapshots are built from

```
ast base-update
```
* Note: the base itself is located at ```/.snapshots/rootfs/snapshot-0``` with it's specific ```/var``` files and ```/etc``` being located at ```/.snapshots/var/var-0``` and ```/.snapshots/etc/etc-0``` respectively, therefore if you really need to make a configuration change, you can mount snapshot these as read-write and then snapshot back as read only

## Package management

#### Software installation
* Software can also be installed using pacman in a chroot
* AUR can be used under the chroot
* Flatpak can be used for persistent package installation
* Using containers for additional software installation is also an option. An easy way of doing this is with [distrobox](https://github.com/89luca89/distrobox)

```
ast install <snapshot> <package>
```

* After installing you can sync the newly installed packages to all the branches of the tree with
* Syncing the tree also automatically updates all the snapshots

```
ast sync <tree>
```

* If you wish to sync without updating (could cause package duplication in database) then use

```
ast force-sync <tree>
```

#### AUR setup

* astOS also supports the AUR natively
* Before we can enable AUR support we first have to make sure ``paru`` is not installed:

```
ast remove <snapshot> paru
```

* To use this feature we first need to enable AUR support in the snapshot configuration:

```
EDITOR=nano ast edit-conf <snapshot> # set the EDITOR variable
```

* Now we need to add the following line into the file:

```
aur::True
```

* Save and quit
* AUR support is now enabled - ``ast install`` and other operations can now install AUR packages as usual

#### Removing software

* For a single snapshot

```
ast remove <snapshot> <package or packages>
```

* Recursively

```
ast tree-rmpkg <tree> <pacakge or packages>
```



#### Updating
* It is advised to clone a snapshot before updating it, so you can roll back in case of failure
* This update only updates the system packages, in order to update ast itself see [this section](https://github.com/lambdanil/astOS#updating-ast-itself)
 

* To update a single snapshot

```
ast upgrade <snapshot>
```
* To recursively update an entire tree

```
ast tree-upgrade <tree>
```

* This can be configured in a script (ie. a crontab script) for easy and safe automatic updates

* If the system becomes unbootable after an update, you can boot last working deployment (select in grub menu) and then perform a rollback

```
ast rollback
```

* Then you can reboot back to a working system

## Extras

#### Fixing pacman corrupt packages / key issues
* Arch's pacman package manager sometimes requires a refresh of the PGP keys
* To fix this issue we can simply reinstall they arch keyring

```
ast install <snapshots> archlinux-keyring
```

#### Saving configuration changes made in ``/etc``
* Normally configuration should be done with ``ast chroot``, but sometimes you may want to apply changes you've made to the booted system persistently
* To do this use the following command

```
ast etc-update
```

* This allows you to configure your system by modifying ``/etc`` as usual, and then saving these changes

#### Dual boot
* astOS supports dual boot using the GRUB bootloader
* When installing the system, use the existing EFI partition
* to configure dual boot, we must begin by installing the ```os-prober``` package:

```
ast install <snapshot> os-prober
```

* Now we have to configure grub

```
ast chroot <snapshot>
echo 'GRUB_DISABLE_OS_PROBER=false' >> /etc/default/grub
exit 0
```

* Now just deploy the snapshot to reconfigure the bootloader

```
ast deploy <snapshot>
```

If Windows is detected, ast should return output along the lines of `Found Windows Boot Manager on...`

You may need to install `ntfs-3g` first and re-deploy if you don't see a Windows entry.

#### Updating ast itself
* ast doesn't get updated alongside the system when `ast upgrade` is used
* sometimes it may be necessary to update ast itself
* ast can be updated with a single command

```
ast ast-sync
```

#### Debugging ast

- sometimes it may be necessary to debug ast
- copy `ast` to any location:

```
cp /usr/local/sbin/ast astpk.py
```

- the following command is useful as it shows outputs of commands when running astpk.py:

```
sed -i -e s,\ 2\>\&1\>\ \/dev\/null,,g astpk.py
```

If you have modified the original ast file (possible but not recommended), please make sure to revert it back when done!

## Known bugs

* When running ast without arguments - IndexError: list index out of range
* Running ast without root permissions shows permission denied errors instead of an error message
* Swap partition doesn't work, it's recommended to use a swapfile or zram instead
* Docker has issues with permissions, to fix run
```
sudo chmod 666 /var/run/docker.sock
```

* If you run into any issues, report them on [the issues page](https://github.com/lambdanil/astOS/issues)

# Contributing
* Code and documentation contributions are welcome
* Bug reports are a good way of contributing to the project too
* Before submitting a pull request test your code and make sure to comment it properly

# Community
* Please feel free to join us on [Discord](https://discord.gg/YVHEC6XNZw) for further discussion and support!
* Happy worry-free snapshotting!

---

**Project is licensed under the AGPLv3 license**

