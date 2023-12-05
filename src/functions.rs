use cmd_lib::{run_cmd, run_fun};
use std::error::Error;
use std::io::Cursor;
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

pub fn create_file(
    file_path: &str,
    file_contents: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    if std::fs::write(file_path, file_contents).is_err() == true {
        print_message(
            "error",
            &format!("failed to create file \"{file_path}\""),
            &message_config,
        );
        return Err(());
    };

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
            "xz" => {
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

pub async fn download_file(
    uri: &str,
    output_directory: &str,
    message_config: &MessageConfig,
) -> Result<(), String> {
    let server_response = reqwest::get(uri.clone()).await;

    let filename: String = String::from(Path::new(&uri).file_name().unwrap().to_string_lossy());

    print_message("debug", &format!("downloading \"{uri}\""), &message_config);

    match server_response {
        Ok(result) => {
            if result.status().is_success() == true {
                let mut output_file =
                    std::fs::File::create(format!("{output_directory}/{filename}")).unwrap();

                let mut binary_contents = Cursor::new(result.bytes().await.unwrap());

                if std::io::copy(&mut binary_contents, &mut output_file).is_err() == true {
                    return Err(format!("failed to write file \"{filename}\""));
                };
            } else {
                let status_code: u16 = result.status().as_u16();

                let reason: String = String::from(result.status().canonical_reason().unwrap());

                return Err(format!(
                    "failed to download file \"{filename}\" ({status_code} {reason})"
                ));
            };
        }
        Err(result) => {
            return Err(format!(
                "failed to download file \"{filename}\" ({})",
                result.source().unwrap()
            ));
        }
    };

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Clone)]
pub struct TargetPackage {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub description: String,
    pub file_size: f64,
    pub file_name: String,
    pub uri: String,
}

pub fn package_set_contains(package_set: &Vec<TargetPackage>, package_name: &str) -> bool {
    let mut is_package_present: bool = false;

    for package in package_set {
        if package.name == package_name {
            is_package_present = true;
        };
    }

    return is_package_present;
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
                        &format!("failed to read file \"{package}\""),
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
                        &format!("failed to read file \"{package}\""),
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
    architectures: &Vec<String>,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    let mut usr_directories_to_symlink: Vec<&str> = Vec::from(["bin", "lib", "sbin"]);

    for architecture in architectures {
        match architecture as &str {
            "amd64" => {
                usr_directories_to_symlink.extend(["lib32", "lib64", "libx32"]);
            }
            "i386" => {
                usr_directories_to_symlink.extend(["lib64", "libx32"]);
            }
            "mipsel" => {
                usr_directories_to_symlink.extend(["lib32", "lib64"]);
            }
            "mips64el" => {
                usr_directories_to_symlink.extend(["lib32", "lib64", "libo32"]);
            }
            "powerpc" => {
                usr_directories_to_symlink.push("lib64");
            }
            "ppc64" => {
                usr_directories_to_symlink.extend(["lib32", "lib64"]);
            }
            "ppc64el" => {
                usr_directories_to_symlink.push("lib64");
            }
            "s390x" => {
                usr_directories_to_symlink.push("lib32");
            }
            "sparc64" => {
                usr_directories_to_symlink.extend(["lib32", "lib64"]);
            }
            "x32" => {
                usr_directories_to_symlink.extend(["lib32", "lib64", "libx32"]);
            }
            _ => {}
        };
    }

    usr_directories_to_symlink.sort_unstable();
    usr_directories_to_symlink.dedup();

    print_message(
        "debug",
        &format!("creating directory \"{bootstrap_directory}/usr\""),
        &message_config,
    );

    if std::fs::create_dir(&format!("{bootstrap_directory}/usr")).is_err() == true {
        print_message(
            "error",
            &format!("failed to create directory \"{bootstrap_directory}/usr\""),
            &message_config,
        );

        return Err(());
    };

    for usr_directory in usr_directories_to_symlink {
        print_message(
            "debug",
            &format!("creating directory \"{bootstrap_directory}/usr/{usr_directory}\""),
            &message_config,
        );

        if std::fs::create_dir(&format!("{bootstrap_directory}/usr/{usr_directory}")).is_err()
            == true
        {
            print_message(
                "error",
                &format!(
                    "failed to create directory \"{bootstrap_directory}/usr/{usr_directory}\""
                ),
                &message_config,
            );

            return Err(());
        };

        print_message(
            "debug",
            &format!("linking \"{bootstrap_directory}/usr/{usr_directory}\" to \"{bootstrap_directory}/{usr_directory}\""),
            &message_config,
        );

        if run_cmd!(ln --symbolic --relative "$bootstrap_directory/usr/$usr_directory" "$bootstrap_directory/$usr_directory" 2> /dev/stdout).is_err() == true {
            print_message(
                "error",
                &format!("failed to create link \"{bootstrap_directory}/{usr_directory}\""),
                &message_config,
            );

            return Err(());
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
        &format!("bind-mounting \"/dev\" to \"{directory}/dev\""),
        &message_config,
    );
    if run_cmd!(mount --bind /dev "$directory/dev" 2> /dev/stdout).is_err() == true {
        unmount_virtual_kernel_file_systems(&directory, &message_config).unwrap_or(());
        return Err(());
    };

    print_message(
        "debug",
        &format!("bind-mounting \"/dev/pts\" to \"{directory}/dev/pts\""),
        &message_config,
    );
    if run_cmd!(mount --bind /dev/pts "$directory/dev/pts" 2> /dev/stdout).is_err() == true {
        unmount_virtual_kernel_file_systems(&directory, &message_config).unwrap_or(());
        return Err(());
    };

    print_message(
        "debug",
        &format!("mounting \"proc\" to \"{directory}/proc\""),
        &message_config,
    );
    if run_cmd!(mount --types proc proc "$directory/proc" 2> /dev/stdout).is_err() == true {
        unmount_virtual_kernel_file_systems(&directory, &message_config).unwrap_or(());
        return Err(());
    };

    print_message(
        "debug",
        &format!("mounting \"sysfs\" to \"{directory}/sys\""),
        &message_config,
    );
    if run_cmd!(mount --types sysfs sysfs "$directory/sys" 2> /dev/stdout).is_err() == true {
        unmount_virtual_kernel_file_systems(&directory, &message_config).unwrap_or(());
        return Err(());
    };

    print_message(
        "debug",
        &format!("mounting \"tmpfs\" to \"{directory}/run\""),
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
                &format!("unmounting \"{directory}/run\""),
                &message_config,
            );
            if run_cmd!(umount --force "$directory/run" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to unmount \"{directory}/run\""),
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
                &format!("unmounting \"{directory}/sys\""),
                &message_config,
            );
            if run_cmd!(umount --force "$directory/sys" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to unmount \"{directory}/sys\""),
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
                &format!("unmounting \"{directory}/proc\""),
                &message_config,
            );
            if run_cmd!(umount --force "$directory/proc" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to unmount \"{directory}/proc\""),
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
                &format!("unmounting \"{directory}/dev/pts\""),
                &message_config,
            );
            if run_cmd!(umount --force "$directory/dev/pts" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to unmount \"{directory}/dev/pts\""),
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
                &format!("unmounting \"{directory}/dev\""),
                &message_config,
            );
            if run_cmd!(umount --force "$directory/dev" 2> /dev/stdout).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to unmount \"{directory}/dev\""),
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
            println!("not removing directory \"{workspace_directory}\"");
        } else {
            print_message(
                "debug",
                &format!("removing directory \"{workspace_directory}\""),
                &message_config,
            );

            if std::fs::remove_dir_all(workspace_directory).is_err() == true {
                print_message(
                    "error",
                    &format!("failed to remove directory \"{workspace_directory}\""),
                    &message_config,
                );
                return Err(());
            };
        };
    };

    print_message("debug", "debstrap exiting now.", &message_config);

    return Ok(());
}
