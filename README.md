# debstrap-rs

**debstrap** bootstraps Debian and derivative systems (Currently it can only use the first of any provided suites, components, architectures, and mirrors).

**debstrap** can be used to create a base Debian installation or a Debian chroot environment.

## Dependencies

The following packages are needed for building **debstrap**:

* rustc
* cargo
* libssl-dev
* pandoc

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
sudo debuild --unsigned-source --unsigned-changes
sudo dpkg --install ../debstrap_0.0.1_*.deb
```

