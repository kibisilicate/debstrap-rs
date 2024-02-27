use crate::package::Package;

use byte_unit::{Byte, Unit, UnitType};
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
            &format!("failed to create file: \"{file_path}\""),
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

pub fn print_packages_dynamically(
    initial_package_set: &Vec<Package>,
    target_package_set: &Vec<Package>,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    let counter_spacing: u16;

    match target_package_set.len() {
        length if length < 10 => {
            counter_spacing = 2;
        }
        length if length < 100 => {
            counter_spacing = 3;
        }
        length if length < 1000 => {
            counter_spacing = 4;
        }
        length if length < 10000 => {
            counter_spacing = 5;
        }
        length if length < 100000 => {
            counter_spacing = 6;
        }
        _ => {
            print_message("error", "invalid size.", &message_config);

            return Err(());
        }
    };

    let mut name_length: u16 = 0;
    let mut version_length: u16 = 0;
    let mut suite_and_component_length: u16 = 0;
    let mut architecture_length: u16 = 0;
    let mut description_length: u16 = 0;
    let mut total_packages_file_size: u64 = 0;
    let mut total_packages_installed_size: u64 = 0;

    for package in target_package_set {
        if package.name.len() as u16 > name_length {
            name_length = package.name.len() as u16;
        };

        if package.version.len() as u16 > version_length {
            version_length = package.version.len() as u16;
        };

        if (package.suite.len() as u16 + package.component.len() as u16)
            > suite_and_component_length
        {
            suite_and_component_length =
                package.suite.len() as u16 + package.component.len() as u16;
        };

        if package.architecture.len() as u16 > architecture_length {
            architecture_length = package.architecture.len() as u16;
        };

        if package.description.len() as u16 > description_length {
            description_length = package.description.len() as u16;
        };

        total_packages_file_size += package.file_size;

        total_packages_installed_size += package.installed_size;
    }

    let min_name_length: u16 = 4;
    let min_version_length: u16 = 7;
    let min_suite_and_component_length: u16 = 15;
    let min_architecture_length: u16 = 12;
    let min_description_length: u16 = 11;

    if name_length < min_name_length {
        name_length = min_name_length;
    };

    if version_length < min_version_length {
        version_length = min_version_length;
    };

    if suite_and_component_length < min_suite_and_component_length {
        suite_and_component_length = min_suite_and_component_length;
    };

    if architecture_length < min_architecture_length {
        architecture_length = min_architecture_length;
    };

    if description_length < min_description_length {
        description_length = min_description_length;
    };

    //////////////////////////////////////////////

    match termion::terminal_size() {
        Ok(result) => {
            let max_width = result.0 - 2;

            let mut current_width: u16 = (&counter_spacing
                + &name_length
                + &version_length
                + &suite_and_component_length
                + &architecture_length
                + &description_length
                + 10
                + 7)
            .try_into()
            .unwrap();

            let mut increase_or_decrease: String = String::new();

            if current_width < max_width {
                increase_or_decrease = String::from("increase");
            };

            if current_width > max_width {
                increase_or_decrease = String::from("decrease");
            };

            if increase_or_decrease.is_empty() == false {
                let mut length_to_change: String = String::from("description");

                loop {
                    current_width = (&counter_spacing
                        + &name_length
                        + &version_length
                        + &suite_and_component_length
                        + &architecture_length
                        + &description_length
                        + 10
                        + 7)
                    .try_into()
                    .unwrap();

                    if current_width == max_width {
                        break;
                    } else {
                        match &length_to_change as &str {
                            "description" => {
                                match &increase_or_decrease as &str {
                                    "increase" => description_length += 1,
                                    "decrease" => {
                                        if description_length > min_description_length {
                                            description_length -= 1;
                                        };
                                    }
                                    _ => {}
                                };
                                length_to_change = String::from("architecture");
                            }
                            "architecture" => {
                                match &increase_or_decrease as &str {
                                    "increase" => architecture_length += 1,
                                    "decrease" => {
                                        if architecture_length > min_architecture_length {
                                            architecture_length -= 1;
                                        };
                                    }
                                    _ => {}
                                };
                                length_to_change = String::from("suite_and_component");
                            }
                            "suite_and_component" => {
                                match &increase_or_decrease as &str {
                                    "increase" => suite_and_component_length += 1,
                                    "decrease" => {
                                        if suite_and_component_length
                                            > min_suite_and_component_length
                                        {
                                            suite_and_component_length -= 1;
                                        };
                                    }
                                    _ => {}
                                };
                                length_to_change = String::from("version");
                            }
                            "version" => {
                                match &increase_or_decrease as &str {
                                    "increase" => version_length += 1,
                                    "decrease" => {
                                        if version_length > min_version_length {
                                            version_length -= 1;
                                        };
                                    }
                                    _ => {}
                                };
                                length_to_change = String::from("name");
                            }
                            "name" => {
                                match &increase_or_decrease as &str {
                                    "increase" => name_length += 1,
                                    "decrease" => {
                                        if name_length > min_name_length {
                                            name_length -= 1;
                                        };
                                    }
                                    _ => {}
                                };
                                length_to_change = String::from("description");
                            }
                            _ => {}
                        };

                        continue;
                    };
                }
            };
        }
        Err(..) => {}
    };

    let mut bold_start: String = String::new();
    let mut bold_end: String = String::new();

    if message_config.color == true {
        bold_start = String::from("\x1b[01m");
        bold_end = String::from("\x1b[00m");
    };

    let mut blank_counter_spacing: String = String::from(" ");

    for _value in 1..counter_spacing {
        blank_counter_spacing.push(' ');
    }

    println!(
        "\n{bold_start}{blank_counter_spacing} {} {} {} {} {} Size{bold_end}",
        space_and_truncate_string("Name", name_length),
        space_and_truncate_string("Version", version_length),
        space_and_truncate_string("Suite/Component", suite_and_component_length),
        space_and_truncate_string("Architecture", architecture_length),
        space_and_truncate_string("Description", description_length),
    );

    let mut counter: u16 = 0;

    for package in target_package_set {
        counter += 1;

        println!(
            "{} {} {} {} {} {} {}",
            space_and_truncate_string(&format!("{counter}."), counter_spacing),
            space_and_truncate_string(&package.name, name_length),
            space_and_truncate_string(&package.version, version_length),
            space_and_truncate_string(
                &format!("{}/{}", &package.suite, &package.component),
                suite_and_component_length,
            ),
            space_and_truncate_string(&package.architecture, architecture_length),
            space_and_truncate_string(&package.description, description_length),
            format!(
                "{:.2}",
                &Byte::from_f64_with_unit(*&package.file_size as f64, Unit::B)
                    .unwrap()
                    .get_appropriate_unit(UnitType::Binary),
            ),
        );
    }

    let amount_of_dependencies: u16 =
        target_package_set.len() as u16 - initial_package_set.len() as u16;

    println!(
        "\n{} initially, {} dependencies, {} packages total.",
        initial_package_set.len(),
        amount_of_dependencies,
        target_package_set.len(),
    );

    println!(
        "Total download size: {}\nTotal installed size: {}",
        format!(
            "{:.2}",
            &Byte::from_u64_with_unit(total_packages_file_size, Unit::B)
                .unwrap()
                .get_appropriate_unit(UnitType::Binary),
        ),
        format!(
            "{:.2}",
            &Byte::from_u64_with_unit(total_packages_installed_size, Unit::KiB)
                .unwrap()
                .get_appropriate_unit(UnitType::Binary),
        ),
    );

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

    print_message("debug", &format!("downloading: \"{uri}\""), &message_config);

    match server_response {
        Ok(result) => {
            if result.status().is_success() == true {
                let mut output_file =
                    std::fs::File::create(format!("{output_directory}/{filename}")).unwrap();

                let mut binary_contents = Cursor::new(result.bytes().await.unwrap());

                if std::io::copy(&mut binary_contents, &mut output_file).is_err() == true {
                    return Err(format!("failed to write file: \"{filename}\""));
                };
            } else {
                let status_code: u16 = result.status().as_u16();

                let reason: String = String::from(result.status().canonical_reason().unwrap());

                return Err(format!(
                    "failed to download file: \"{filename}\" ({status_code} {reason})"
                ));
            };
        }
        Err(result) => {
            return Err(format!(
                "failed to download file: \"{filename}\" ({})",
                result.source().unwrap()
            ));
        }
    };

    return Ok(());
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn download_package_lists(
    input_uris: &Vec<(String, String)>,
    input_suites: &Vec<String>,
    input_components: &Vec<String>,
    input_architectures: &Vec<String>,
    output_directory: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    let total_amount_to_download: u64 = (&input_uris.len()
        * &input_suites.len()
        * &input_components.len()
        * &input_architectures.len())
        .try_into()
        .unwrap();

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

    for (scheme, path) in input_uris {
        for suite in input_suites {
            for component in input_components {
                for architecture in input_architectures {
                    counter += 1;

                    println!(
                        "{} {}",
                        space_and_truncate_string(
                            &format!("({counter}/{}):", total_amount_to_download),
                            counter_spacing,
                        ),
                        format!("{scheme}{path} {suite}/{component} {architecture} Packages"),
                    );

                    let mut did_package_list_download: bool = false;

                    let package_list_uri: String =
                        format!("{scheme}{path}/dists/{suite}/{component}/binary-{architecture}");

                    let potential_file_names: Vec<String> = Vec::from([
                        String::from("Packages.xz"),
                        String::from("Packages.gz"),
                        String::from("Packages.bz2"),
                        String::from("Packages"),
                    ]);

                    for file_name in potential_file_names {
                        match tokio::runtime::Runtime::new()
                            .unwrap()
                            .block_on(download_file(
                                &format!("{package_list_uri}/{file_name}"),
                                &output_directory,
                                &message_config,
                            )) {
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
                            Err(message) => {
                                print_message(
                                    "debug",
                                    &format!("{message}, skipping."),
                                    &message_config,
                                );
                            }
                        };
                    }

                    if did_package_list_download == false {
                        print_message("error", "failed to download package list.", &message_config);
                        return Err(());
                    };
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
        if (package.uri_scheme.len() as u16 + package.uri_path.len() as u16) > full_uri_length {
            full_uri_length = package.uri_scheme.len() as u16 + package.uri_path.len() as u16;
        };

        if (package.suite.len() as u16 + 1 + package.component.len() as u16)
            > suite_and_component_length
        {
            suite_and_component_length =
                package.suite.len() as u16 + 1 + package.component.len() as u16;
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
                &format!("{}{}", package.uri_scheme, package.uri_path),
                full_uri_length
            ),
            space_and_truncate_string(
                &format!("{}/{}", package.suite, package.component),
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

        match tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(download_file(
                &format!(
                    "{}{}/{}",
                    package.uri_scheme, package.uri_path, package.file_name
                ),
                &output_directory,
                &message_config,
            )) {
            Ok(..) => {}
            Err(message) => {
                print_message("error", &message, &message_config);

                return Err(());
            }
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
            "loong64" => {
                usr_directories_to_symlink.extend(["lib32", "lib64"]);
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
        &format!("creating directory: \"{bootstrap_directory}/usr\""),
        &message_config,
    );

    if std::fs::create_dir(&format!("{bootstrap_directory}/usr")).is_err() == true {
        print_message(
            "error",
            &format!("failed to create directory: \"{bootstrap_directory}/usr\""),
            &message_config,
        );

        return Err(());
    };

    for usr_directory in usr_directories_to_symlink {
        print_message(
            "debug",
            &format!("creating directory: \"{bootstrap_directory}/usr/{usr_directory}\""),
            &message_config,
        );

        if std::fs::create_dir(&format!("{bootstrap_directory}/usr/{usr_directory}")).is_err()
            == true
        {
            print_message(
                "error",
                &format!(
                    "failed to create directory: \"{bootstrap_directory}/usr/{usr_directory}\""
                ),
                &message_config,
            );

            return Err(());
        };

        print_message(
            "debug",
            &format!("linking: \"{bootstrap_directory}/usr/{usr_directory}\" to \"{bootstrap_directory}/{usr_directory}\""),
            &message_config,
        );

        if run_cmd!(ln --symbolic --relative "$bootstrap_directory/usr/$usr_directory" "$bootstrap_directory/$usr_directory" 2> /dev/stdout).is_err() == true {
            print_message(
                "error",
                &format!("failed to create link: \"{bootstrap_directory}/{usr_directory}\""),
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
            &format!("running hook no. {counter}"),
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
