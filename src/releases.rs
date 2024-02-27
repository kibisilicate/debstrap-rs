pub const DEBIAN_CURRENT_SUITES: [&str; 10] = [
    "unstable",     // alias for "sid"
    "testing",      // alias for "trixie"
    "stable",       // alias for "bookworm"
    "oldstable",    // alias for "bullseye"
    "oldoldstable", // alias for "buster"
    "sid",          // latest version
    "trixie",       // Debian 13
    "bookworm",     // Debian 12
    "bullseye",     // Debian 11
    "buster",       // Debian 10
];

pub const DEBIAN_OBSOLUTE_SUITES: [&str; 2] = [
    "stretch", // Debian 9
    "jessie",  // Debian 8
];

pub const DEBIAN_CURRENT_MIRROR: &str = "https://deb.debian.org/debian";

pub const DEBIAN_PORTS_MIRROR: &str = "https://deb.debian.org/debian-ports";

pub const DEBIAN_OBSOLETE_MIRROR: &str = "http://archive.debian.org/debian";

////////////////////////////////////////////////////////////////////////////////////////////////////

pub const UBUNTU_CURRENT_SUITES: [&str; 9] = [
    "devel",  // alias for latest version
    "noble",  // Ubuntu 24.04 LTS
    "mantic", // Ubuntu 23.10
    "lunar",  // Ubuntu 23.04
    "jammy",  // Ubuntu 22.04 LTS
    "focal",  // Ubuntu 20.04 LTS
    "bionic", // Ubuntu 18.04 LTS
    "xenial", // Ubuntu 16.04 LTS
    "trusty", // Ubuntu 14.04 LTS
];

pub const UBUNTU_OBSOLETE_SUITES: [&str; 17] = [
    "kinetic", // Ubuntu 22.10
    "impish",  // Ubuntu 21.10
    "hirsute", // Ubuntu 21.04
    "groovy",  // Ubuntu 20.10
    "eoan",    // Ubuntu 19.10
    "disco",   // Ubuntu 19.04
    "cosmic",  // Ubuntu 18.10
    "artful",  // Ubuntu 17.10
    "zesty",   // Ubuntu 17.04
    "yakkety", // Ubuntu 16.10
    "wily",    // Ubuntu 15.10
    "vivid",   // Ubuntu 15.04
    "utopic",  // Ubuntu 14.10
    "saucy",   // Ubuntu 13.10
    "raring",  // Ubuntu 13.04
    "quantal", // Ubuntu 12.10
    "precise", // Ubuntu 12.04 LTS
];

pub const UBUNTU_CURRENT_MIRROR: &str = "http://archive.ubuntu.com/ubuntu";

pub const UBUNTU_PORTS_MIRROR: &str = "http://ports.ubuntu.com/ubuntu-ports";

pub const UBUNTU_OBSOLETE_MIRROR: &str = "https://old-releases.ubuntu.com/ubuntu";

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn check_primary_suite(suite: &str) -> bool {
    if DEBIAN_CURRENT_SUITES.contains(&suite) == true
        || DEBIAN_OBSOLUTE_SUITES.contains(&suite) == true
        || UBUNTU_CURRENT_SUITES.contains(&suite) == true
        || UBUNTU_OBSOLETE_SUITES.contains(&suite) == true
    {
        return true;
    } else {
        return false;
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn case_specific_packages(suite: &str, variant: &str) -> Option<Vec<&'static str>> {
    let mut packages_to_include: Vec<&str> = Vec::new();

    match variant {
        "essential" | "custom" => {}
        _ => {
            packages_to_include.push("ca-certificates");
            match suite {
                "stretch" | "jessie" | "zesty" | "yakkety" | "xenial" | "wily" | "vivid"
                | "utopic" | "trusty" | "saucy" | "raring" | "quantal" | "precise" => {
                    packages_to_include.push("apt-transport-https");
                }
                _ => {}
            };
        }
    };

    return Some(packages_to_include);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn default_output_file_name(suite: &str, architecture: &str, variant: &str) -> String {
    let mut file_name: String = String::new();

    if DEBIAN_CURRENT_SUITES.contains(&suite) == true
        || DEBIAN_OBSOLUTE_SUITES.contains(&suite) == true
    {
        file_name = format!("Debian_{}_{}_{}", suite, architecture, variant,);
    } else if UBUNTU_CURRENT_SUITES.contains(&suite) == true
        || UBUNTU_OBSOLETE_SUITES.contains(&suite) == true
    {
        file_name = format!("Ubuntu_{}_{}_{}", suite, architecture, variant,);
    };

    return file_name;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn default_uris(suite: &str, architecture: &str) -> Vec<String> {
    let mut uris: Vec<String> = Vec::new();

    if DEBIAN_CURRENT_SUITES.contains(&suite) == true {
        match architecture {
            "amd64" | "arm64" | "armel" | "armhf" | "i386" | "mips64el" | "mipsel" | "ppc64el"
            | "s390x" => {
                uris.push(String::from(DEBIAN_CURRENT_MIRROR));
            }
            _ => {
                uris.push(String::from(DEBIAN_PORTS_MIRROR));
            }
        };
    } else if DEBIAN_OBSOLUTE_SUITES.contains(&suite) == true {
        uris.push(String::from(DEBIAN_OBSOLETE_MIRROR));
    } else if UBUNTU_CURRENT_SUITES.contains(&suite) == true {
        match architecture {
            "amd64" | "i386" => {
                uris.push(String::from(UBUNTU_CURRENT_MIRROR));
            }
            _ => {
                uris.push(String::from(UBUNTU_PORTS_MIRROR));
            }
        };
    } else if UBUNTU_OBSOLETE_SUITES.contains(&suite) == true {
        uris.push(String::from(UBUNTU_OBSOLETE_MIRROR));
    };

    return uris;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn default_merge_usr_directories(suite: &str, variant: &str) -> bool {
    let mut merge_usr_directories: bool = false;

    if DEBIAN_CURRENT_SUITES.contains(&suite) == true
        || DEBIAN_OBSOLUTE_SUITES.contains(&suite) == true
    {
        match suite as &str {
            // merged-usr blacklist for past Debian releases.
            "stretch" | "jessie" => {
                merge_usr_directories = false;
            }
            "bookworm" | "bullseye" | "buster" => {
                match variant as &str {
                    // Debian releases <= bookworm keep the variant 'buildd' split-usr by default
                    // and merged-usr for all other variants.
                    "buildd" => {
                        merge_usr_directories = false;
                    }
                    _ => {
                        merge_usr_directories = true;
                    }
                };
            }
            // all Debian releases >= trixie no longer support split-usr.
            _ => {
                merge_usr_directories = true;
            }
        };
    } else if UBUNTU_CURRENT_SUITES.contains(&suite) == true
        || UBUNTU_OBSOLETE_SUITES.contains(&suite) == true
    {
        match suite as &str {
            // merged-usr blacklist for past Ubuntu releases.
            "groovy" | "focal" | "eoan" | "disco" | "cosmic" | "bionic" | "artful" | "zesty"
            | "yakkety" | "xenial" | "wily" | "vivid" | "utopic" | "trusty" | "saucy"
            | "raring" | "quantal" | "precise" => {
                merge_usr_directories = false;
            }
            // for hirsute keep the variant 'buildd' split-usr by default and merged-usr for all
            // other variants.
            "hirsute" => {
                match variant as &str {
                    "buildd" => {
                        merge_usr_directories = false;
                    }
                    _ => {
                        merge_usr_directories = true;
                    }
                };
            }
            _ => {
                merge_usr_directories = true;
            }
        };
    };

    return merge_usr_directories;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn method_of_usr_merge(suite: &str) -> String {
    let merge_method: String;

    match suite as &str {
        "bookworm" | "bullseye" | "buster" | "stretch" | "jessie" | "hirsute" | "groovy"
        | "focal" | "eoan" | "disco" | "cosmic" | "bionic" | "artful" | "zesty" | "yakkety"
        | "xenial" | "wily" | "vivid" | "utopic" | "trusty" | "saucy" | "raring" | "quantal"
        | "precise" => {
            merge_method = String::from("manual_usr_merge");
        }
        _ => {
            merge_method = String::from("usrmerge_package");
        }
    };

    return merge_method;
}

//////////////////////////////////////////////////

pub fn is_split_usr_supported(suite: &str) -> bool {
    let is_supported: bool;

    match suite as &str {
        "bullseye" | "buster" | "stretch" | "jessie" | "groovy" | "focal" | "eoan" | "disco"
        | "cosmic" | "bionic" | "artful" | "zesty" | "yakkety" | "xenial" | "wily" | "vivid"
        | "utopic" | "trusty" | "saucy" | "raring" | "quantal" | "precise" => {
            is_supported = true;
        }
        _ => {
            // Debian releases >= bookworm and Ubuntu releases >= hirsute are merged-usr.
            is_supported = false;
        }
    };

    return is_supported;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn default_sources_list_format(suite: &str) -> String {
    let sources_list_format: String;

    match suite {
        "jessie" | "wily" | "vivid" | "utopic" | "trusty" | "saucy" | "raring" | "quantal"
        | "precise" => {
            sources_list_format = String::from("one-line-style");
        }
        _ => {
            sources_list_format = String::from("deb822-style");
        }
    };

    return sources_list_format;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn default_hostname(suite: &str) -> String {
    let mut hostname: String = String::new();

    if DEBIAN_CURRENT_SUITES.contains(&suite) == true
        || DEBIAN_OBSOLUTE_SUITES.contains(&suite) == true
    {
        hostname = String::from("debian");
    } else if UBUNTU_CURRENT_SUITES.contains(&suite) == true
        || UBUNTU_OBSOLETE_SUITES.contains(&suite) == true
    {
        hostname = String::from("ubuntu");
    };

    return hostname;
}
