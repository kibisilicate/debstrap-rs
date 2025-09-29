#! /usr/bin/env bash

make || exit 1

declare format='directory'
#declare format='tarball'

declare -a mirrors=(
  # Debian mirrors:
  'https://deb.debian.org/debian'
  #'https://deb.debian.org/debian-ports'
  #'http://archive.debian.org/debian'
  # Ubuntu mirrors:
  #'http://archive.ubuntu.com/ubuntu'
  #'http://ports.ubuntu.com/ubuntu-ports'
  #'https://old-releases.ubuntu.com/ubuntu'
)

declare -a suites=(
  # Debian releases:
  'unstable'        # alias for 'sid'
  #'experimental'    # alias for 'rc-buggy'
  #'testing'         # alias for 'forky'
  #'stable'          # alias for 'trixie'
  #'oldstable'       # alias for 'bookworm'
  #'oldoldstable'    # alias for 'bullseye'
  #'sid'             # latest version
  #'rc-buggy'        # latest experimental packages
  #'forky'           # Debian 14
  #'trixie'          # Debian 13
  #'bookworm'        # Debian 12
  #'bullseye'        # Debian 11
  #'buster'          # Debian 10
  #'stretch'         # Debian 9
  #'jessie'          # Debian 8
  # Ubuntu releases:
  #'devel'           # alias for latest version
  #'questing'        # Ubuntu 25.10
  #'plucky'          # Ubuntu 25.04
  #'oracular'        # Ubuntu 24.10
  #'noble'           # Ubuntu 24.04 LTS
  #'mantic'          # Ubuntu 23.10
  #'lunar'           # Ubuntu 23.04
  #'kinetic'         # Ubuntu 22.10
  #'jammy'           # Ubuntu 22.04 LTS
  #'impish'          # Ubuntu 21.10
  #'hirsute'         # Ubuntu 21.04
  #'groovy'          # Ubuntu 20.10
  #'focal'           # Ubuntu 20.04 LTS
  #'eoan'            # Ubuntu 19.10
  #'disco'           # Ubuntu 19.04
  #'cosmic'          # Ubuntu 18.10
  #'bionic'          # Ubuntu 18.04 LTS
  #'artful'          # Ubuntu 17.10
  #'zesty'           # Ubuntu 17.04
  #'yakkety'         # Ubuntu 16.10
  #'xenial'          # Ubuntu 16.04 LTS
  #'wily'            # Ubuntu 15.10
  #'vivid'           # Ubuntu 15.04
  #'utopic'          # Ubuntu 14.10
  #'trusty'          # Ubuntu 14.04 LTS
  #'saucy'           # Ubuntu 13.10
  #'raring'          # Ubuntu 13.04
  #'quantal'         # Ubuntu 12.10
  #'precise'         # Ubuntu 12.04 LTS
)

declare -a components=(
  'main'
  # Extra Debian components:
  #'contrib'
  #'non-free'
  #'non-free-firmware'
  # Extra Ubuntu components:
  #'universe'
  #'multiverse'
  #'restricted'
)

declare -a architectures=(
  #'host'
  #'alpha'
  'amd64'
  #'arm64'
  #'armel'
  #'armhf'
  #'hppa'
  #'i386'
  #'ia64'
  #'loong64'
  #'m68k'
  #'mips64el'
  #'mipsel'
  #'powerpc'
  #'ppc64'
  #'ppc64el'
  #'riscv64'
  #'s390x'
  #'sh4'
  #'sparc64'
  #'x32'
)

#declare variant='essential'
declare variant='required'
#declare variant='buildd'
#declare variant='important'
#declare variant='standard'
#declare variant='custom'

declare resolver='internal'
#declare resolver='none'

sudo ./target/release/debstrap \
  --format="$format" \
  --mirrors="$(echo "${mirrors[@]}")" \
  --releases="$(echo "${suites[@]}")" \
  --components="$(echo "${components[@]}")" \
  --architectures="$(echo "${architectures[@]}")" \
  --variant="$variant" \
  --resolver="$resolver" \
  --discard-output \
  $(echo "$@")

if [ "$?" = '0' ]; then
  exit 0
else
  exit 1
fi

