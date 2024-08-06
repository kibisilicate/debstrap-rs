use crate::package::*;
use crate::sources::*;

use byte_unit::{Byte, Unit, UnitType};
use cmd_lib::{run_cmd, run_fun};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Cursor, Write};
use std::os::unix::fs;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub struct MessageConfig {
    pub color: bool,
    pub debug: bool,
}

/*

print_message("KIND", &format!("MESSAGE"), &message_config);

*/

pub fn print_message(kind: &str, message: &str, message_config: &MessageConfig) {
    let prefix: &str;

    match kind {
        "debug" => {
            prefix = if message_config.color == true {
                "\x1b[01;36m[DEBUG]\x1b[00m"
            } else {
                "[DEBUG]"
            }
        }
        "warning" => {
            prefix = if message_config.color == true {
                "\x1b[01;33mwarning\x1b[00m:"
            } else {
                "warning:"
            }
        }
        "error" => {
            prefix = if message_config.color == true {
                "\x1b[01;31merror\x1b[00m:"
            } else {
                "error:"
            }
        }
        _ => panic!("invalid message type"),
    }

    match kind {
        "debug" => {
            if message_config.debug == true {
                println!("{prefix} {message}")
            }
        }
        "warning" | "error" => eprintln!("{prefix} {message}"),
        _ => {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn parse_list_of_values(prefix: &str, input: &str) -> Vec<String> {
    let value: Vec<String> = input
        .replacen(prefix, "", 1)
        .replace(",", " ")
        .split_whitespace()
        .map(|element| String::from(element.trim()))
        .collect::<Vec<String>>();

    return value;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn space_and_truncate_string(input_string: &str, output_length: u16) -> String {
    let mut output_string: String = String::from(input_string);

    let mut input_length: u16 = output_string.len().try_into().unwrap();

    if input_length > output_length {
        match output_string.char_indices().nth((output_length - 3).into()) {
            Some((value, _)) => {
                output_string = output_string[..value].to_string() + "...";
            }
            None => {}
        };
        input_length = output_length;
    };

    let spacing_length: u16 = output_length + 1 - input_length;

    for _value in 1..spacing_length {
        output_string.push(' ');
    }

    return output_string;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn get_debian_architecture_name(architecture: &str) -> Result<String, ()> {
    let debian_architecture_name: String;

    match &architecture.to_lowercase().replace("-", "_") as &str {
        "alpha" => {
            debian_architecture_name = String::from("alpha");
        }
        "amd64" | "x86_64" | "x64" => {
            debian_architecture_name = String::from("amd64");
        }
        "arm64" | "aarch64" => {
            debian_architecture_name = String::from("arm64");
        }
        "armel" => {
            debian_architecture_name = String::from("armel");
        }
        "armhf" | "aarch32" | "armv7l" => {
            debian_architecture_name = String::from("armhf");
        }
        "hppa" | "parisc" => {
            debian_architecture_name = String::from("hppa");
        }
        "i386" | "i686" | "ia32" | "x86" | "x86_32" => {
            debian_architecture_name = String::from("i386");
        }
        "ia64" => {
            debian_architecture_name = String::from("ia64");
        }
        "loong64" | "loongarch64" => {
            debian_architecture_name = String::from("loong64");
        }
        "m68k" => {
            debian_architecture_name = String::from("m68k");
        }
        "mips64el" | "mips64" => {
            debian_architecture_name = String::from("mips64el");
        }
        "mipsel" | "mips" => {
            debian_architecture_name = String::from("mipsel");
        }
        "powerpc" | "ppc" => {
            debian_architecture_name = String::from("powerpc");
        }
        "ppc64" => {
            debian_architecture_name = String::from("ppc64");
        }
        "ppc64el" | "ppc64le" | "powerpc64le" => {
            debian_architecture_name = String::from("ppc64el");
        }
        "riscv64" => {
            debian_architecture_name = String::from("riscv64");
        }
        "s390x" => {
            debian_architecture_name = String::from("s390x");
        }
        "sh4" => {
            debian_architecture_name = String::from("sh4");
        }
        "sparc64" => {
            debian_architecture_name = String::from("sparc64");
        }
        "x32" => {
            debian_architecture_name = String::from("x32");
        }
        _ => {
            return Err(());
        }
    };

    return Ok(debian_architecture_name);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn create_file(
    file_path: &str,
    file_contents: &str,
    chosen_file_permissions: Option<u32>,
    chosen_file_ownership: Option<(u32, u32)>,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    let file_permissions: u32;

    match chosen_file_permissions {
        Some(result) => file_permissions = result,
        None => file_permissions = 0o644,
    };

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(file_permissions)
        .open(file_path)
    {
        Ok(mut result) => {
            if result.write_all(file_contents.as_bytes()).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to write to file: \"{file_path}\""),
                    &message_config,
                );
                return Err(());
            };
        }
        Err(..) => {
            print_message(
                "error",
                &format!("failed to create file: \"{file_path}\""),
                &message_config,
            );
            return Err(());
        }
    };

    let file_ownership: (u32, u32);

    match chosen_file_ownership {
        Some((uid, gid)) => file_ownership = (uid, gid),
        None => file_ownership = (0, 0),
    };

    if fs::chown(file_path, Some(file_ownership.0), Some(file_ownership.1)).is_err() == true {
        print_message(
            "error",
            &format!("failed to set ownership of file: \"{file_path}\""),
            &message_config,
        );
        return Err(());
    };

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn append_file(
    file_path: &str,
    file_contents: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    match std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(file_path)
    {
        Ok(mut result) => {
            if result.write_all(file_contents.as_bytes()).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to write to file: \"{file_path}\""),
                    &message_config,
                );
                return Err(());
            };
        }
        Err(..) => {
            print_message(
                "error",
                &format!("failed to open file: \"{file_path}\""),
                &message_config,
            );
            return Err(());
        }
    }

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn move_file(from: String, to: String) -> Result<(), ()> {
    if std::fs::copy(&from, to).is_err() == true {
        return Err(());
    };

    if std::fs::remove_file(from).is_err() == true {
        return Err(());
    };

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn decompress_file(
    input_file_location: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    if Path::new(input_file_location).extension().is_some() == true {
        let file_extension: String = String::from(
            Path::new(input_file_location)
                .extension()
                .unwrap()
                .to_string_lossy(),
        );

        match &file_extension as &str {
            "xz" | "lzma" => {
                if run_cmd!(xz --decompress "$input_file_location" 2> /dev/stdout).is_err() == true
                {
                    print_message(
                        "error",
                        &format!("failed to decompress file: \"{input_file_location}\""),
                        &message_config,
                    );

                    return Err(());
                };
            }
            "gz" => {
                if run_cmd!(gzip --decompress "$input_file_location" 2> /dev/stdout).is_err()
                    == true
                {
                    print_message(
                        "error",
                        &format!("failed to decompress file: \"{input_file_location}\""),
                        &message_config,
                    );

                    return Err(());
                };
            }
            "bz2" => {
                if run_cmd!(bzip2 --decompress "$input_file_location" 2> /dev/stdout).is_err()
                    == true
                {
                    print_message(
                        "error",
                        &format!("failed to decompress file: \"{input_file_location}\""),
                        &message_config,
                    );

                    return Err(());
                };
            }
            _ => {
                print_message(
                    "error",
                    &format!("unrecognized file format: \"{input_file_location}\""),
                    &message_config,
                );

                return Err(());
            }
        };
    };

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn create_directory(directory_path: &str, message_config: &MessageConfig) -> Result<(), ()> {
    match Path::new(directory_path).exists() {
        true => return Ok(()),
        false => {
            print_message(
                "debug",
                &format!("creating directory: \"{directory_path}\""),
                &message_config,
            );

            match std::fs::create_dir(directory_path) {
                Ok(..) => return Ok(()),
                Err(..) => {
                    print_message(
                        "error",
                        &format!("failed to create directory: \"{directory_path}\""),
                        &message_config,
                    );
                    return Err(());
                }
            };
        }
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn remove_directory(directory_path: &str, message_config: &MessageConfig) -> Result<(), ()> {
    match Path::new(directory_path).exists() {
        true => {
            print_message(
                "debug",
                &format!("removing directory: \"{directory_path}\""),
                &message_config,
            );

            match std::fs::remove_dir(directory_path) {
                Ok(..) => return Ok(()),
                Err(..) => {
                    print_message(
                        "error",
                        &format!("failed to remove directory: \"{directory_path}\""),
                        &message_config,
                    );
                    return Err(());
                }
            };
        }
        false => {
            print_message(
                "error",
                &format!("directory: \"{directory_path}\" does not exist."),
                &message_config,
            );
            return Err(());
        }
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn does_network_resource_exist(uri: &str) -> bool {
    async fn does_network_resource_exist(uri: &str) -> Result<(), ()> {
        match reqwest::get(uri).await {
            Ok(result) => {
                if result.status().is_success() == true {
                    return Ok(());
                } else {
                    return Err(());
                };
            }
            Err(..) => return Err(()),
        };
    }

    match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(does_network_resource_exist(uri))
    {
        Ok(..) => return true,
        Err(..) => return false,
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn download_file(
    uri: &str,
    output_directory: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    async fn download_file(
        uri: &str,
        output_directory: &str,
        message_config: &MessageConfig,
    ) -> Result<(), ()> {
        let file_name: String =
            String::from(Path::new(&uri).file_name().unwrap().to_string_lossy());

        match reqwest::get(uri).await {
            Ok(result) => {
                if result.status().is_success() == true {
                    let mut output_file =
                        std::fs::File::create(format!("{output_directory}/{file_name}")).unwrap();

                    let mut binary_contents = Cursor::new(result.bytes().await.unwrap());

                    if std::io::copy(&mut binary_contents, &mut output_file).is_err() == true {
                        print_message(
                            "error",
                            &format!("failed to write file: \"{file_name}\""),
                            &message_config,
                        );
                        return Err(());
                    };
                } else {
                    print_message(
                        "error",
                        &format!(
                            "failed to download file: \"{file_name}\" ({} {})",
                            result.status().as_u16(),
                            result.status().canonical_reason().unwrap(),
                        ),
                        &message_config,
                    );

                    return Err(());
                };
            }
            Err(result) => {
                print_message(
                    "error",
                    &format!(
                        "failed to download file: \"{file_name}\" ({})",
                        result.source().unwrap(),
                    ),
                    &message_config,
                );
                return Err(());
            }
        };

        return Ok(());
    }

    print_message("debug", &format!("downloading: \"{uri}\""), &message_config);

    match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(download_file(uri, output_directory, message_config))
    {
        Ok(..) => return Ok(()),
        Err(..) => return Err(()),
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn download_package_lists(
    sources_list: &Vec<SourcesEntry>,
    output_directory: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    let mut total_amount_to_download: u64 = 0;

    for entry in sources_list {
        for _uri in &entry.uris {
            for _suite in &entry.suites {
                for _component in &entry.components {
                    for _architecture in &entry.architectures {
                        total_amount_to_download += 1;
                    }
                }
            }
        }
    }

    let counter_spacing: u16;

    match total_amount_to_download {
        length if length < 10 => {
            counter_spacing = 6;
        }
        length if length < 100 => {
            counter_spacing = 8;
        }
        length if length < 1000 => {
            counter_spacing = 10;
        }
        length if length < 10000 => {
            counter_spacing = 12;
        }
        length if length < 100000 => {
            counter_spacing = 14;
        }
        _ => {
            print_message("error", "invalid size.", &message_config);

            return Err(());
        }
    };

    let mut counter: u64 = 0;

    for entry in sources_list {
        for (scheme, path) in &entry.uris {
            for suite in &entry.suites {
                for component in &entry.components {
                    for architecture in &entry.architectures {
                        counter += 1;

                        println!(
                            "{} {}",
                            space_and_truncate_string(
                                &format!("({counter}/{total_amount_to_download}):"),
                                counter_spacing,
                            ),
                            format!("{scheme}{path} {suite}/{component} {architecture} Packages"),
                        );

                        let package_list_parent_path: String = format!(
                            "{scheme}{path}/dists/{suite}/{component}/binary-{architecture}"
                        );

                        let potential_file_names: Vec<String> = Vec::from([
                            String::from("Packages.xz"),
                            String::from("Packages.gz"),
                            String::from("Packages.bz2"),
                            String::from("Packages"),
                        ]);

                        let mut did_package_list_download: bool = false;

                        for file_name in potential_file_names {
                            let package_list_uri: String =
                                format!("{package_list_parent_path}/{file_name}");

                            if does_network_resource_exist(&package_list_uri) == true {
                                match download_file(
                                    &package_list_uri,
                                    &output_directory,
                                    &message_config,
                                ) {
                                    Ok(..) => {
                                        did_package_list_download = true;

                                        if decompress_file(
                                            &format!("{output_directory}/{file_name}"),
                                            &message_config,
                                        )
                                        .is_err()
                                            == true
                                        {
                                            return Err(());
                                        };

                                        let package_list_file_name: String = format!("{path}/dists/{suite}/{component}/binary-{architecture}_Packages").replace("/", "_");

                                        if std::fs::rename(
                                            format!("{output_directory}/Packages"),
                                            format!("{output_directory}/{package_list_file_name}"),
                                        )
                                        .is_err()
                                            == true
                                        {
                                            print_message(
                                                "error",
                                                &format!("failed to rename file: \"Packages\""),
                                                &message_config,
                                            );
                                            return Err(());
                                        };

                                        break;
                                    }
                                    Err(..) => return Err(()),
                                };
                            };
                        }

                        if did_package_list_download == false {
                            print_message("error", "failed to find package list.", &message_config);
                            return Err(());
                        };
                    }
                }
            }
        }
    }

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn download_packages(
    target_package_set: &Vec<Package>,
    output_directory: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    let counter_spacing: u16;

    match target_package_set.len() {
        length if length < 10 => {
            counter_spacing = 6;
        }
        length if length < 100 => {
            counter_spacing = 8;
        }
        length if length < 1000 => {
            counter_spacing = 10;
        }
        length if length < 10000 => {
            counter_spacing = 12;
        }
        length if length < 100000 => {
            counter_spacing = 14;
        }
        _ => {
            print_message("error", "invalid size.", &message_config);

            return Err(());
        }
    };

    let mut full_uri_length: u16 = 0;
    let mut suite_and_component_length: u16 = 0;
    let mut name_length: u16 = 0;
    let mut version_length: u16 = 0;
    let mut architecture_length: u16 = 0;

    for package in target_package_set {
        if (package.origin_uri_scheme.len() as u16 + package.origin_uri_path.len() as u16)
            > full_uri_length
        {
            full_uri_length =
                package.origin_uri_scheme.len() as u16 + package.origin_uri_path.len() as u16;
        };

        if (package.origin_suite.len() as u16 + 1 + package.origin_component.len() as u16)
            > suite_and_component_length
        {
            suite_and_component_length =
                package.origin_suite.len() as u16 + 1 + package.origin_component.len() as u16;
        };

        if package.name.len() as u16 > name_length {
            name_length = package.name.len() as u16;
        };

        if package.version.len() as u16 > version_length {
            version_length = package.version.len() as u16;
        };

        if package.architecture.len() as u16 > architecture_length {
            architecture_length = package.architecture.len() as u16;
        };
    }

    let mut counter: u64 = 0;

    for package in target_package_set {
        counter += 1;

        println!(
            "{} {} {} {} {} {} {}",
            space_and_truncate_string(
                &format!("({counter}/{}):", target_package_set.len()),
                counter_spacing
            ),
            space_and_truncate_string(
                &format!("{}{}", package.origin_uri_scheme, package.origin_uri_path),
                full_uri_length
            ),
            space_and_truncate_string(
                &format!("{}/{}", package.origin_suite, package.origin_component),
                suite_and_component_length
            ),
            space_and_truncate_string(&package.name, name_length),
            space_and_truncate_string(&package.version, version_length),
            space_and_truncate_string(&package.architecture, architecture_length),
            format!(
                "{:.2}",
                &Byte::from_f64_with_unit(package.file_size as f64, Unit::B)
                    .unwrap()
                    .get_appropriate_unit(UnitType::Binary),
            ),
        );

        if download_file(
            &format!(
                "{}{}/{}",
                package.origin_uri_scheme, package.origin_uri_path, package.file_name
            ),
            &output_directory,
            &message_config,
        )
        .is_err()
            == true
        {
            return Err(());
        };
    }

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn extract_deb_control_field(
    extractor: &str,
    package: &str,
    message_config: &MessageConfig,
) -> Result<String, ()> {
    let mut control_field: String = String::new();

    match extractor as &str {
        "ar" => {
            let tarball: String;

            match run_fun!(ar -t "$package" | grep --regexp="^control.tar") {
                Ok(result) => {
                    tarball = result;
                }
                Err(..) => {
                    print_message(
                        "error",
                        &format!("failed to read file: \"{package}\""),
                        &message_config,
                    );
                    return Err(());
                }
            };

            match &tarball as &str {
                "control.tar" => {
                    control_field = String::from(
                        run_fun!(ar -p "$package" "$tarball" | tar --extract --to-stdout ./control)
                            .unwrap(),
                    );
                }
                "control.tar.bz2" => {
                    control_field = String::from(
                        run_fun!(ar -p "$package" "$tarball" | bzip2 --decompress --stdout --force | tar --extract --to-stdout ./control).unwrap(),
                    );
                }
                "control.tar.gz" => {
                    control_field = String::from(
                        run_fun!(ar -p "$package" "$tarball" | gzip --decompress --to-stdout --force | tar --extract --to-stdout ./control).unwrap(),
                    );
                }
                "control.tar.xz" => {
                    control_field = String::from(
                        run_fun!(ar -p "$package" "$tarball" | xz --decompress --to-stdout --force | tar --extract --to-stdout ./control).unwrap(),
                    );
                }
                "control.tar.zst" => {
                    control_field =
                        String::from(run_fun!(ar -p "$package" "$tarball" | zstd --decompress --stdout --force | tar --extract --to-stdout ./control).unwrap());
                }
                _ => {}
            };
        }
        "dpkg-deb" => {
            control_field = String::from(run_fun!(dpkg-deb --field "$package").unwrap());
        }
        _ => {}
    };

    return Ok(control_field);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn extract_deb_data(
    extractor: &str,
    package: &str,
    output_directory: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    match extractor as &str {
        "ar" => {
            let tarball: String;

            match run_fun!(ar -t "$package" | grep --regexp="^data.tar") {
                Ok(result) => {
                    tarball = result;
                }
                Err(..) => {
                    print_message(
                        "error",
                        &format!("failed to read file: \"{package}\""),
                        &message_config,
                    );
                    return Err(());
                }
            };

            match &tarball as &str {
                "data.tar" => {
                    if run_cmd!(
                        ar -p "$package" "$tarball" |
                            tar --extract --keep-directory-symlink --directory="$output_directory"
                    )
                    .is_err()
                        == true
                    {
                        print_message(
                            "error",
                            &format!("failed to extract package: \"{package}\""),
                            &message_config,
                        );
                        return Err(());
                    };
                }
                "data.tar.bz2" => {
                    if run_cmd!(
                        ar -p "$package" "$tarball" |
                            bzip2 --decompress --stdout --force |
                            tar --extract --keep-directory-symlink --directory="$output_directory"
                    )
                    .is_err()
                        == true
                    {
                        print_message(
                            "error",
                            &format!("failed to extract package: \"{package}\""),
                            &message_config,
                        );
                        return Err(());
                    };
                }
                "data.tar.gz" => {
                    if run_cmd!(
                        ar -p "$package" "$tarball" |
                            gzip --decompress --to-stdout --force |
                            tar --extract --keep-directory-symlink --directory="$output_directory"
                    )
                    .is_err()
                        == true
                    {
                        print_message(
                            "error",
                            &format!("failed to extract package: \"{package}\""),
                            &message_config,
                        );
                        return Err(());
                    };
                }
                "data.tar.xz" => {
                    if run_cmd!(
                        ar -p "$package" "$tarball" |
                            xz --decompress --to-stdout --force |
                            tar --extract --keep-directory-symlink --directory="$output_directory"
                    )
                    .is_err()
                        == true
                    {
                        print_message(
                            "error",
                            &format!("failed to extract package: \"{package}\""),
                            &message_config,
                        );
                        return Err(());
                    };
                }
                "data.tar.zst" => {
                    if run_cmd!(
                        ar -p "$package" "$tarball" |
                            zstd --decompress --stdout --force |
                            tar --extract --keep-directory-symlink --directory="$output_directory"
                    )
                    .is_err()
                        == true
                    {
                        print_message(
                            "error",
                            &format!("failed to extract package: \"{package}\""),
                            &message_config,
                        );
                        return Err(());
                    };
                }
                _ => {}
            };
        }
        "dpkg-deb" => {
            if run_cmd!(dpkg-deb --fsys-tarfile "$package").is_err() == true {
                print_message(
                    "error",
                    &format!("failed to extract package: \"{package}\""),
                    &message_config,
                );
                return Err(());
            };
        }
        _ => {}
    };

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn manually_merge_usr_directories(
    bootstrap_directory: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    println!("Merging /usr directories ...");

    let usr_directories_to_symlink: Vec<String> = Vec::from([
        String::from("bin"),
        String::from("lib"),
        String::from("lib32"),
        String::from("lib64"),
        String::from("libo32"),
        String::from("libx32"),
        String::from("sbin"),
    ]);

    for directory in &usr_directories_to_symlink {
        if Path::new(&format!("{bootstrap_directory}/{directory}")).exists() == false {
            print_message(
                "debug",
                &format!(
                    "directory: \"{bootstrap_directory}/{directory}\" does not exist, skipping."
                ),
                &message_config,
            );
            continue;
        } else if Path::new(&format!("{bootstrap_directory}/{directory}")).is_symlink() == true {
            print_message(
                "debug",
                &format!("directory: \"{bootstrap_directory}/{directory}\" is already a symlink, skipping."),
                &message_config,
            );
            continue;
        } else {
            print_message(
                "debug",
                &format!("merging directory: \"{bootstrap_directory}/{directory}\" to \"{bootstrap_directory}/usr/{directory}\""),
                &message_config,
            );

            if run_cmd!(cp --recursive --preserve=all "$bootstrap_directory/$directory" "$bootstrap_directory/usr" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to merge directory: \"{bootstrap_directory}/{directory}\""),
                    &message_config,
                );
                return Err(());
            };

            if run_cmd!(rm --recursive --force "$bootstrap_directory/$directory" 2> /dev/stdout)
                .is_err()
                == true
            {
                print_message(
                    "error",
                    &format!("failed to remove directory: \"{bootstrap_directory}/{directory}\""),
                    &message_config,
                );
                return Err(());
            };

            print_message(
                "debug",
                &format!("linking: \"{bootstrap_directory}/usr/{directory}\" to \"{bootstrap_directory}/{directory}\""),
                &message_config,
            );

            if run_cmd!(ln --symbolic --relative "$bootstrap_directory/usr/$directory" "$bootstrap_directory/$directory" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to create link: \"{bootstrap_directory}/{directory}\""),
                    &message_config,
                );
                return Err(());
            };
        };
    }

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn mount_virtual_kernel_file_systems(
    directory: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    print_message(
        "debug",
        &format!("bind-mounting: \"/dev\" to \"{directory}/dev\""),
        &message_config,
    );
    if run_cmd!(mount --bind /dev "$directory/dev" 2> /dev/stdout).is_err() == true {
        unmount_virtual_kernel_file_systems(&directory, &message_config).unwrap_or(());
        return Err(());
    };

    print_message(
        "debug",
        &format!("bind-mounting: \"/dev/pts\" to \"{directory}/dev/pts\""),
        &message_config,
    );
    if run_cmd!(mount --bind /dev/pts "$directory/dev/pts" 2> /dev/stdout).is_err() == true {
        unmount_virtual_kernel_file_systems(&directory, &message_config).unwrap_or(());
        return Err(());
    };

    print_message(
        "debug",
        &format!("mounting: \"proc\" to \"{directory}/proc\""),
        &message_config,
    );
    if run_cmd!(mount --types proc proc "$directory/proc" 2> /dev/stdout).is_err() == true {
        unmount_virtual_kernel_file_systems(&directory, &message_config).unwrap_or(());
        return Err(());
    };

    print_message(
        "debug",
        &format!("mounting: \"sysfs\" to \"{directory}/sys\""),
        &message_config,
    );
    if run_cmd!(mount --types sysfs sysfs "$directory/sys" 2> /dev/stdout).is_err() == true {
        unmount_virtual_kernel_file_systems(&directory, &message_config).unwrap_or(());
        return Err(());
    };

    print_message(
        "debug",
        &format!("mounting: \"tmpfs\" to \"{directory}/run\""),
        &message_config,
    );
    if run_cmd!(mount --types tmpfs tmpfs "$directory/run" 2> /dev/stdout).is_err() == true {
        unmount_virtual_kernel_file_systems(&directory, &message_config).unwrap_or(());
        return Err(());
    };

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn unmount_virtual_kernel_file_systems(
    directory: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    let currently_mounted_file_systems: Vec<String> = run_fun!(mount)
        .unwrap()
        .lines()
        .map(|element| String::from(element))
        .collect();

    for entry in &currently_mounted_file_systems {
        if entry.contains(&format!(" {directory}/run ")) == true {
            print_message(
                "debug",
                &format!("unmounting: \"{directory}/run\""),
                &message_config,
            );
            if run_cmd!(umount --force "$directory/run" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to unmount: \"{directory}/run\""),
                    &message_config,
                );
                return Err(());
            };
            break;
        };
    }

    for entry in &currently_mounted_file_systems {
        if entry.contains(&format!(" {directory}/sys ")) == true {
            print_message(
                "debug",
                &format!("unmounting: \"{directory}/sys\""),
                &message_config,
            );
            if run_cmd!(umount --force "$directory/sys" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to unmount: \"{directory}/sys\""),
                    &message_config,
                );
                return Err(());
            };
            break;
        };
    }

    for entry in &currently_mounted_file_systems {
        if entry.contains(&format!(" {directory}/proc ")) == true {
            print_message(
                "debug",
                &format!("unmounting: \"{directory}/proc\""),
                &message_config,
            );
            if run_cmd!(umount --force "$directory/proc" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to unmount: \"{directory}/proc\""),
                    &message_config,
                );
                return Err(());
            };
            break;
        };
    }

    for entry in &currently_mounted_file_systems {
        if entry.contains(&format!(" {directory}/dev/pts ")) == true {
            print_message(
                "debug",
                &format!("unmounting: \"{directory}/dev/pts\""),
                &message_config,
            );
            if run_cmd!(umount --force "$directory/dev/pts" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to unmount: \"{directory}/dev/pts\""),
                    &message_config,
                );
                return Err(());
            };
            break;
        };
    }

    for entry in &currently_mounted_file_systems {
        if entry.contains(&format!(" {directory}/dev ")) == true {
            print_message(
                "debug",
                &format!("unmounting: \"{directory}/dev\""),
                &message_config,
            );
            if run_cmd!(umount --force "$directory/dev" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to unmount: \"{directory}/dev\""),
                    &message_config,
                );
                return Err(());
            };
            break;
        };
    }

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn run_hooks(
    kind: &str,
    hooks: &Vec<String>,
    workspace_directory: &str,
    chosen_packages_directory: Option<&str>,
    chosen_target_directory: Option<&str>,
    message_config: &MessageConfig,
) {
    let packages_directory: String;

    match chosen_packages_directory {
        Some(result) => packages_directory = String::from(result),
        None => packages_directory = String::new(),
    };

    let target_directory: String;

    match chosen_target_directory {
        Some(result) => target_directory = String::from(result),
        None => target_directory = String::new(),
    };

    let mut counter: u16 = 0;

    for shell_code in hooks {
        let mut did_hook_return_error: bool = false;

        counter += 1;

        print_message(
            "debug",
            &format!("running {kind} hook no. {counter} {{\n{shell_code}\n}}"),
            &message_config,
        );

        match kind as &str {
            "download" => {
                if run_cmd!(
                    /usr/bin/env bash -c "
export WORKSPACE='$workspace_directory'
export PACKAGES='$packages_directory'

cd \"$$WORKSPACE\"

$shell_code
" 2> /dev/stdout
                )
                .is_err()
                    == true
                {
                    did_hook_return_error = true;
                };
            }
            "extract" | "essential" | "target" => {
                if run_cmd!(
                    /usr/bin/env bash -c "
export WORKSPACE='$workspace_directory'
export PACKAGES='$packages_directory'
export TARGET='$target_directory'

cd \"$$WORKSPACE\"

$shell_code
" 2> /dev/stdout
                )
                .is_err()
                    == true
                {
                    did_hook_return_error = true;
                };
            }
            "done" => {
                if run_cmd!(
                    /usr/bin/env bash -c "
export WORKSPACE='$workspace_directory'
export TARGET='$target_directory'

cd \"$$WORKSPACE\"

$shell_code
" 2> /dev/stdout
                )
                .is_err()
                    == true
                {
                    did_hook_return_error = true;
                };
            }
            _ => panic!("invalid hook type"),
        };

        if did_hook_return_error == true {
            print_message("warning", "hook returned an error.", &message_config);
        };
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn clean_up_on_exit(
    workspace_directory: &str,
    target_directory: Option<&str>,
    actions_to_skip: &Vec<String>,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    if target_directory.is_some() == true && Path::new(&target_directory.unwrap()).exists() == true
    {
        let mut are_any_file_systems_mounted: bool = false;

        for entry in run_fun!(mount)
            .unwrap()
            .lines()
            .map(|element| String::from(element))
            .collect::<Vec<String>>()
        {
            if entry.contains(&format!(" {}/run ", target_directory.unwrap())) == true
                || entry.contains(&format!(" {}/sys ", target_directory.unwrap())) == true
                || entry.contains(&format!(" {}/proc ", target_directory.unwrap())) == true
                || entry.contains(&format!(" {}/dev/pts ", target_directory.unwrap())) == true
                || entry.contains(&format!(" {}/dev ", target_directory.unwrap())) == true
            {
                are_any_file_systems_mounted = true;
            };
        }

        if are_any_file_systems_mounted == true {
            print_message(
                "debug",
                "some virtual kernel file systems are still mounted.",
                &message_config,
            );
            if unmount_virtual_kernel_file_systems(&target_directory.unwrap(), &message_config)
                .is_err()
                == true
            {
                return Err(());
            };
        };
    };

    if Path::new(workspace_directory).exists() == true {
        if actions_to_skip.contains(&String::from("workspace_removal")) == true {
            println!("not removing directory: \"{workspace_directory}\"");
        } else {
            print_message(
                "debug",
                &format!("removing directory: \"{workspace_directory}\""),
                &message_config,
            );

            if std::fs::remove_dir_all(workspace_directory).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to remove directory: \"{workspace_directory}\""),
                    &message_config,
                );
                return Err(());
            };
        };
    };

    print_message("debug", "debstrap exiting now", &message_config);

    return Ok(());
}
