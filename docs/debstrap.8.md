% DEBSTRAP(8) debstrap 0.0.1-experimental
%
% 26 November 2023

# NAME
debstrap - Bootstrap overly complicated Debian systems

# SYNOPSIS
**debstrap**\ \[*OPTION*\...\]\ **-r**=*SUITE*\[,*SUITE*,\...\]\ **-o**=*DIRECTORY*|*FILENAME*

**debstrap**\ \[*OPTION*\...\]\ **\--release**=*SUITE*\[,*SUITE*,\...\]\ **\--output**=*DIRECTORY*|*FILENAME*

# DESCRIPTION
**debstrap** bootstraps a Debian system using one or more suites, components, architectures, and mirrors.

**debstrap** can be used to create a base Debian installation or a Debian chroot environment (see the section **EXAMPLES**).

# OPTIONS
Any option that accepts a list of values can be specified multiple times as the values are concatenated,
and values can be separated by commas and/or whitespace.

**-h**, **\--help**
:   Print help message and exit.

**-V**, **\--version**
:   Print **debstrap** version information and exit.

**-d**, **\--debug**
:   Print extra information useful for debugging.

**\--no-debug**
:   Don\'t print debug information.

**-o**, **\--output**=*DIRECTORY*|*FILENAME*
:   Set the output location.
    It can be a relative or absolute path to a directory or filename.
    If no filename is provided and the format requires one then one will be auto-generated.

**-f**, **\--format**=*FORMAT*
:   Set the output format.
    The default format is \'**directory**\' unless the provided output location implies a different format.

    See the section **FORMATS** for more information.

**-r**, **\--release**, **\--releases**=*SUITE*\[,*SUITE*,\...\]
:   List of suites from the archive to use.
    At least one suite must be specified.

    *SUITE* can be release code names (e.g. \'**sid**\', \'**trixie**\', \'**bookworm**\', \'**bullseye**\')
    and/or symbolic names (e.g. \'**unstable**\', \'**testing**\', \'**stable**\', \'**oldstable**\').

**-c**, **\--comp**, **\--component**, **\--components**=*COMPONENT*\[,*COMPONENT*,\...\]
:   List of components from the archive to use.
    The default component is \'**main**\'.

**-a**, **\--arch**, **\--architecture**, **\--architectures**=*ARCHITECTURE*\[,*ARCHITECTURE*,\...\]
:   List of architectures to use.
    The first is the targets native architecture and any remaining are added to the targets **dpkg** foreign architectures list.
    The default architecture is the same as the host.

    See the section **ARCHITECTURES** for more information.

**-v**, **\--variant**=*VARIANT*
:   Set which package set (variant) to use.
    The default variant is \'**important**\'.

    See the section **VARIANTS** for more information.

**-p**, **\--package-set**=*PACKAGE*\[,*PACKAGE*,\...\]
:   List of packages to be used as the initial package set of the variant \'**custom**\'.

**-i**, **\--include**=*PACKAGE*\[,*PACKAGE*,\...\]
:   List of packages to be added to the initial package set before dependency resolution.

**-e**, **\--exclude**=*PACKAGE*\[,*PACKAGE*,\...\]
:   List of packages to be removed from the initial package set before dependency resolution.

**-P**, **\--prohibit**=*PACKAGE*\[,*PACKAGE*,\...\]
:   List of packages to be prohibited from being present in the target package set during dependency resolution.

**-m**, **\--mirror**, **\--mirrors**=*URI*\[,*URI*,\...\]
:   List of mirrors to use.

    Supported *URI* schemes are \'**http://**\', and \'**https://**\'.

**-R**, **\--resolver**=*RESOLVER*
:   Set which dependency resolver to use.
    The default resolver is \'**internal**\'.

    See the section **RESOLVERS** for more information.

**\--include-recommends**
:   Consider recommended packages as dependencies during dependency resolution.

**\--exclude-recommends**
:   Do not consider recommended packages as dependencies during dependency resolution (this is the default behavior).

**-E**, **\--extractor**=*EXTRACTOR*
:   Set which package extractor to use.
    The default extractor is \'**ar**\'.

    Supported extractors are \'**ar**\', and \'**dpkg-deb**\'.

**\--extract-essential**
:   Extract only the essential packages and their immediate dependencies.
    This is recommended for all variants and is the default behavior.

**\--extract-all**
:   Extract all of the packages, not just the essentials.
    Not recommended for larger variants.

**\--consider-essential**=*PACKAGE*\[,*PACKAGE*,\...\]
:   List of packages to consider essential when separating and extracting the essential packages.

**\--consider-non-essential**=*PACKAGE*\[,*PACKAGE*,\...\]
:   List of packages to consider not essential when separating and extracting the essential packages.

**-M**, **\--merge-usr**=*WHEN*
:   Whether or not to have a merged /usr directory scheme.

    *WHEN* can be \'**yes**\'|\'**true**\', \'**no**\'|\'**false**\', or \'**auto**\'.

    The default is \'**auto**\' which merges the /usr directories if supported by the chosen suite and/or variant.

**\--deb822-style**
:   Use the deb822-style format when creating the targets sources list.
    This is the default when the targets version of **apt** is 1.1 or newer.

**\--one-line-style**
:   Use the traditional one-line-style format when creating the targets sources list.
    This is the default when the targets version of **apt** is older than 1.1.

**\--interactive**
:   Wait for user input on prompts during the bootstrap.
    Useful for configuration during the bootstrap but may cause hanging in automated scripts.

    Inside the target sets the environment variables **DEBIAN_FRONTEND** to \'**dialog**\', and **DEBCONF_NONINTERACTIVE_SEEN** to \'**false**\'.


**\--non-interactive**
:   Do not wait for user input on prompts during the bootstrap (this is the default behavior).
    Useful for preventing hanging in scripts.

    Inside the target sets the environment variables **DEBIAN_FRONTEND** to \'**noninteractive**\', and **DEBCONF_NONINTERACTIVE_SEEN** to \'**true**\'.

**\--hook-***NAME*=*COMMAND*
:   Run shell code at specified points during the operation of **debstrap**.

    See the section **HOOKS** for more information.

**-D**, **\--discard**, **\--discard-output**
:   Uses a sub-directory inside the temporary workspace as the output directory of the target which is deleted on exit unless the option **\--skip=workspace-removal** is used.

**-y**, **\--yes**, **\--assume-yes**
:   Automatically assume yes to all prompts.
    If used with **\--interactive** may cause hanging in automated scripts.

**-n**, **\--no**, **\--assume-no**
:   Automatically assume no to all prompts.

**\--color**\[=*WHEN*\]
:   Whether or not to use a colorful output.

    *WHEN* can be \'**always**\'|\'**true**\', \'**never**\'|\'**false**\', or \'**auto**\'.

    The default is \'**auto**\' which only uses color when standard output is a terminal.

**-O**, **\--only**=*ACTION*
:   Only perform a certain action then exit.

    See the section **ONLY ACTION** for more information.

**-S**, **\--skip**=*ACTION*\[,*ACTION*,\...\]
:   List of actions to skip during the operation of **debstrap**.

    See the section **SKIP ACTION** for more information.

# FORMATS
The following formats are supported by **debstrap**:

**directory**, **dir**
:   Uses the output directory as the target directory for the bootstrap.
    The output directory must either be empty or only contain the directories \'**/boot**\', \'**/efi**\', or \'**/lost+found**\'.
    This can be skipped if the option **\--skip=output-directory-check** is used.

**tarball**, **tar**
:   Uses a sub-directory inside the temporary workspace as the target directory for the bootstrap.
    Once finished it creates a tarball containing all of the contents of the target directory, then moves the tarball to the output directory.

# ARCHITECTURES
The following architectures are supported by **debstrap**:

**host**
An alias that selects all the architectures the host kernel natively supports.

**alpha**

**amd64**, **x86_64**, **x64**

**arm64**, **aarch64**

**armel**

**armhf**, **aarch32**, **armv7l**

**hppa**, **parisc**

**i386**, **i686**, **ia32**, **x86**, **x86_32**

**ia64**

**loong64**, **loongarch64**

**m68k**

**mips64el**, **mips64**

**mipsel**, **mips**

**powerpc**, **ppc**

**ppc64**

**ppc64el**, **ppc64le**, **powerpc64le**

**riscv64**

**s390x**

**sh4**

**sparc64**

**x32**

# VARIANTS
The following variants are supported by **debstrap**:

**essential**\
\ \ \ \ \ \ \ Consists of all \'**Essential: yes**\' packages plus **mawk** (the default provider of **awk** which is essential).

**required**\
\ \ \ \ \ \ \ Consists of the variant \'**essential**\' plus all \'**Priority: required**\' packages and **apt**.

**buildd**\
\ \ \ \ \ \ \ Consists of the variant \'**required**\' plus all \'**Build-Essential: yes**\' packages and **build-essential**.

**important**\
\ \ \ \ \ \ \ Consists of the variant \'**required**\' plus all \'**Priority: important**\' packages.

**standard**\
\ \ \ \ \ \ \ Consists of the variant \'**important**\' plus all \'**Priority: standard**\' packages.

**custom**\
\ \ \ \ \ \ \ Consists of nothing by default, only packages provided by the option **\--package-set** will be used.

# RESOLVERS
The following resolvers are supported by **debstrap**:

**internal**\
\ \ \ \ \ \ \ \ Dependencies are resolved internally by **debstrap**.
    Currently, it can only use the first suite, component, architecture, and mirror.

**none**\
\ \ \ \ \ \ \ \ Does not perform any dependency resolution, uses the initial package set as the target package set.

# HOOKS
The following hooks are provided by **debstrap**:

**download**\
\ \ \ \ \ \ \ Run shell code after the packages are downloaded but before they are extracted.

**extract**\
\ \ \ \ \ \ \ Run shell code after the packages have been extracted but before installing them.

**essential**\
\ \ \ \ \ \ \ Run shell code after all the essential packages have been installed but before installing the remaining packages.
    If the option **\--extract-all** is used then the essential hooks function the same as the target hooks.

**target**\
\ \ \ \ \ \ \ Run shell code after all the packages have been installed but before cleanup.

**done**\
\ \ \ \ \ \ \ Run shell code after the cleanup before exiting.

# ONLY ACTION
The following specific actions are provided by **debstrap**:

**print**\
\ \ \ \ \ \ \ Print both the initial and target package sets and exit.

**print-initial**\
\ \ \ \ \ \ \ Only print the initial package set (before dependency resolution) and exit.

**print-target**\
\ \ \ \ \ \ \ Only print the target package set (after dependency resolution) and exit.

**download**\
\ \ \ \ \ \ \ Only download the packages and exit.

<!-- If this option is used and no format was chosen and/or implied then the default format will be \'**tarball**\'. -->

**extract**\
\ \ \ \ \ \ \ Only extract the packages and exit.

# SKIP ACTION
The following **debstrap** actions can be skipped:

**architecture-check**
:   Skip checking whether the host kernel supports executing the target architecture.

**output-directory-check**
:   Skip checking whether the output directory is empty.

**packages-removal**
:   Skip removing the packages directory \'**\\\$TARGET/packages/**\' inside the target during cleanup.

**workspace-removal**
:   Skip removing the temporary workspace on exit.

# ENVIRONMENT

## External Environment

**DEBSTRAP_COLOR**\
\ \ \ \ \ \ \ Accepts the same values as the option **\--color** (the option **\--color** can override this).

**NO_COLOR**\
\ \ \ \ \ \ \ Do not use a colorful output (the option **\--color**, and the variable **DEBSTRAP_COLOR** can override this).
    See \'https://no-color.org/\' for more information.

**DEBSTRAP_DEBUG**\
\ \ \ \ \ \ \ Whether or not to print debug information.
    Valid values are \'**true**\', and \'**false**\'.

**DEBSTRAP_DIRECTORY**\
\ \ \ \ \ \ \ Use the specified directory as the temporary workspace.

\ \ \ \ \ \ \ The directory must exist and be empty, and it will be deleted on exit unless the option **\--skip=workspace-removal** is used.

## Internal Environment

Make sure to escape the dollar sign \'**\$**\' when using these variables so they don\'t get interpreted by the shell before getting passed to **debstrap**.

**WORKSPACE**\
\ \ \ \ \ \ \ The temporary directory that **debstrap** uses for storing downloaded package list(s) and downloaded packages.

\ \ \ \ \ \ \ All hooks export this variable.

**PACKAGES**\
\ \ \ \ \ \ \ The directory that the downloaded packages are stored in prior to separating essentials, extraction, and installation.

\ \ \ \ \ \ \ The hook \'**download**\' exports this variable.

**INITIAL_PACKAGES**\
\ \ \ \ \ \ \ The directory that all the essential packages are stored in prior to extraction, and installation.

\ \ \ \ \ \ \ The hooks \'**extract**\', \'**essential**\', and \'**target**\' export this variable.

**REMAINING_PACKAGES**\
\ \ \ \ \ \ \ The directory that all the remaining packages are stored in prior to installation.

\ \ \ \ \ \ \ The hooks \'**extract**\', \'**essential**\', and \'**target**\' export this variable.

**TARGET**\
\ \ \ \ \ \ \ The directory that the contents of the target is stored in.

\ \ \ \ \ \ \ The hooks \'**extract**\', \'**essential**\', \'**target**\', and \'**done**\' export this variable.

# EXAMPLES

* Basic usage:

\ \ \ \ **debstrap \--output=/mnt \--release=sid**

* Creating a buildd chroot:

\ \ \ \ **mkdir \--verbose ./buildd-chroot**\
\ \ \ \ **debstrap \--output=./buildd-chroot \--release=sid \--variant=buildd**

* Copying a local package into the targets packages directory:

\ \ \ \ **debstrap \--output=/mnt \--release=sid \--hook-download=\"cp -v ~/my-package.deb \\\"\\\$PACKAGES\\\"\"**

* Removing an unwanted package from the targets packages directory:

\ \ \ \ **debstrap \--output=/mnt \--release=sid \--hook-download=\"rm -v \\\"\\\$PACKAGES/unwanted-package.deb\\\"\"**

* Listing the contents of the target:

\ \ \ \ **debstrap \--output=/mnt \--release=sid \--hook-target=\"ls -lah \\\"\\\$TARGET\\\"\"**

* Adding the hosts network configuration for internet access inside the target:

\ \ \ \ **debstrap \--output=/mnt \--release=sid \--hook-target=\"**\
\ \ \ \ \ \ \ \ **cp \--verbose /etc/hostname \\\"\\\$TARGET/etc/hostname\\\"**

\ \ \ \ \ \ \ \ **rm \--verbose \\\"\\\$TARGET/etc/hosts\\\"**\
\ \ \ \ \ \ \ \ **cp \--verbose /etc/hosts \\\"\\\$TARGET/etc/hosts\\\"**

\ \ \ \ \ \ \ \ **rm \--verbose \\\"\\\$TARGET/etc/resolv.conf\\\"**\
\ \ \ \ \ \ \ \ **cp \--verbose /etc/resolv.conf \\\"\\\$TARGET/etc/resolv.conf\\\"**\
\ \ \ \ \ \ **\"**

* Entering the chroot environment of the target:

\ \ \ \ **debstrap \--output=/mnt \--release=sid \--hook-target=\"**\
\ \ \ \ \ \ \ \ **chroot \\\"\\\$TARGET\\\" /usr/bin/env \--ignore-environment bash -c \\\"**\
\ \ \ \ \ \ \ \ \ \ **export HOME=\'/root\'**\
\ \ \ \ \ \ \ \ \ \ **export TERM=\'xterm-256color\'**\
\ \ \ \ \ \ \ \ \ \ **bash \--login**\
\ \ \ \ \ \ \ \ **\\\"**\
\ \ \ \ \ \ **\"**

* Editing the targets fstab:

\ \ \ \ **debstrap \--output=/mnt \--release=sid \--hook-target=\"nano \\\"\\\$TARGET/etc/fstab\\\"\"**

# SEE ALSO

**debootstrap**(8), **mmdebstrap**(1)

