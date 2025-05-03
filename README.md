# debstrap-rs

**debstrap** bootstraps Debian and derivative systems using one or more suites, components, architectures, and mirrors.

**debstrap** can be used to create a base Debian installation or a Debian chroot environment.

## Dependencies

### Build Dependencies

The following packages are needed for building **debstrap**:

* rustc
* cargo
* pandoc

### Runtime Dependencies

The following packages are needed for running **debstrap**:

* binutils
* coreutils
* mount
* tar
* gzip
* xz-utils
* arch-test

### Optional Dependencies

The following packages provide extra functionality to **debstrap**:

* qemu-user
* qemu-user-binfmt
* binfmt-support
* bzip2
* zstd

## Installation

Clone the repository:

```
git clone https://github.com/kibisilicate/debstrap-rs.git
cd ./debstrap-rs
```

### Using make

Manually compile and install:

```
make
sudo make install
```

### Building Package

Build and install .deb package:

```
debuild --unsigned-source --unsigned-changes
sudo dpkg --install ../debstrap_*.deb
```

