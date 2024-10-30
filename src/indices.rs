use crate::functions::*;
use crate::sources::*;

use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Release {
    pub origin: String,
    pub label: String,
    pub version: String,
    pub suite: String,
    pub code_name: String,
    pub architectures: Vec<String>,
    pub components: Vec<String>,
    pub date: String,
    pub valid_until: String,
    pub description: String,
    pub sha256_hashes: HashMap<String, (String, u64)>,
    pub md5_hashes: HashMap<String, (String, u64)>,
}

pub fn parse_release_file(
    release_file: &str,
    message_config: &MessageConfig,
) -> Result<Release, ()> {
    let release_entry: Release;

    match std::fs::read_to_string(release_file) {
        Ok(result) => {
            let mut origin: String = String::new();
            let mut label: String = String::new();
            let mut version: String = String::new();
            let mut suite: String = String::new();
            let mut code_name: String = String::new();
            let mut architectures: Vec<String> = Vec::new();
            let mut components: Vec<String> = Vec::new();
            let mut date: String = String::new();
            let mut valid_until: String = String::new();
            let mut description: String = String::new();

            for line in result.lines() {
                match &line as &str {
                    _ if line.starts_with("Origin: ") => {
                        origin = line.replacen("Origin: ", "", 1);
                    }
                    _ if line.starts_with("Label: ") => {
                        label = line.replacen("Label: ", "", 1);
                    }
                    _ if line.starts_with("Version: ") => {
                        version = line.replacen("Version: ", "", 1);
                    }
                    _ if line.starts_with("Suite: ") => {
                        suite = line.replacen("Suite: ", "", 1);
                    }
                    _ if line.starts_with("Codename: ") => {
                        code_name = line.replacen("Codename: ", "", 1);
                    }
                    _ if line.starts_with("Architectures: ") => {
                        architectures = parse_list_of_values("Architectures: ", &line);
                    }
                    _ if line.starts_with("Components: ") => {
                        components = parse_list_of_values("Components: ", &line);
                    }
                    _ if line.starts_with("Date: ") => {
                        date = line.replacen("Date: ", "", 1);
                    }
                    _ if line.starts_with("Valid-Until: ") => {
                        valid_until = line.replacen("Valid-Until: ", "", 1);
                    }
                    _ if line.starts_with("Description: ") => {
                        description = line.replacen("Description: ", "", 1);
                    }
                    _ => {}
                };
            }

            //////////////////////////////////////

            let mut sha256_hashes: HashMap<String, (String, u64)> = HashMap::new();

            let mut continue_from_next: bool = false;

            for line in result.lines() {
                if line.starts_with("SHA256:") == true {
                    continue_from_next = true;
                    continue;
                };

                if continue_from_next == true {
                    if line.starts_with(" ") == true {
                        let split_string: Vec<String> = line
                            .split_whitespace()
                            .map(|element| String::from(element.trim()))
                            .collect::<Vec<String>>();

                        sha256_hashes.insert(
                            split_string[2].clone(),
                            (split_string[0].clone(), split_string[1].parse().unwrap()),
                        );
                    } else {
                        break;
                    };
                };
            }

            //////////////////////////////////////

            let mut md5_hashes: HashMap<String, (String, u64)> = HashMap::new();

            let mut continue_from_next: bool = false;

            for line in result.lines() {
                if line.starts_with("MD5Sum:") == true {
                    continue_from_next = true;
                    continue;
                };

                if continue_from_next == true {
                    if line.starts_with(" ") == true {
                        let split_string: Vec<String> = line
                            .split_whitespace()
                            .map(|element| String::from(element.trim()))
                            .collect::<Vec<String>>();

                        md5_hashes.insert(
                            split_string[2].clone(),
                            (split_string[0].clone(), split_string[1].parse().unwrap()),
                        );
                    } else {
                        break;
                    };
                };
            }

            //////////////////////////////////////

            release_entry = Release {
                origin: origin,
                label: label,
                version: version,
                suite: suite,
                code_name: code_name,
                architectures: architectures,
                components: components,
                date: date,
                valid_until: valid_until,
                description: description,
                sha256_hashes: sha256_hashes,
                md5_hashes: md5_hashes,
            };
        }
        Err(..) => {
            print_message(
                "error",
                &format!("failed to read Release file: \"{release_file}\""),
                &message_config,
            );
            return Err(());
        }
    };

    return Ok(release_entry);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn download_indices(
    sources_list: &Vec<SourcesEntry>,
    output_directory: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    let mut total_amount_to_download: u64 = 0;

    for entry in sources_list {
        for _uri in &entry.uris {
            for _suite in &entry.suites {
                total_amount_to_download += 1;

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
                counter += 1;

                println!(
                    "{} {path} {suite} Release",
                    space_and_truncate_string(
                        &format!("({counter}/{total_amount_to_download}):"),
                        counter_spacing,
                    ),
                );

                let release_file_uri: String = format!("{scheme}{path}/dists/{suite}/Release");

                let release_file_name: String =
                    format!("{path}_dists_{suite}_Release").replace("/", "_");

                match does_network_resource_exist(&release_file_uri) {
                    true => {
                        match download_file(&release_file_uri, &output_directory, &message_config) {
                            Ok(..) => {
                                if std::fs::rename(
                                    format!("{output_directory}/Release"),
                                    format!("{output_directory}/{release_file_name}"),
                                )
                                .is_err()
                                    == true
                                {
                                    print_message(
                                        "error",
                                        "failed to rename file: \"Release\"",
                                        &message_config,
                                    );
                                    return Err(());
                                };
                            }
                            Err(..) => {
                                print_message(
                                    "error",
                                    "failed to download Release file.",
                                    &message_config,
                                );
                                return Err(());
                            }
                        };
                    }
                    false => {
                        print_message("error", "failed to find Release file.", &message_config);
                        return Err(());
                    }
                };

                //////////////////////////////////

                for component in &entry.components {
                    for architecture in &entry.architectures {
                        counter += 1;

                        println!(
                            "{} {path} {suite}/{component} {architecture} Packages",
                            space_and_truncate_string(
                                &format!("({counter}/{total_amount_to_download}):"),
                                counter_spacing,
                            ),
                        );

                        let package_list_parent_path: String = format!(
                            "{scheme}{path}/dists/{suite}/{component}/binary-{architecture}"
                        );

                        let potential_file_names: Vec<String> = Vec::from([
                            String::from("Packages.xz"),
                            String::from("Packages.gz"),
                            String::from("Packages.bz2"),
                            String::from("Packages.lzma"),
                            String::from("Packages"),
                        ]);

                        let package_list_file_name: String = format!(
                            "{path}_dists_{suite}_{component}_binary-{architecture}_Packages"
                        )
                        .replace("/", "_");

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

                                        if std::fs::rename(
                                            format!("{output_directory}/Packages"),
                                            format!("{output_directory}/{package_list_file_name}"),
                                        )
                                        .is_err()
                                            == true
                                        {
                                            print_message(
                                                "error",
                                                "failed to rename file: \"Packages\"",
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

                        //////////////////////////

                        match parse_release_file(
                            &format!("{output_directory}/{release_file_name}"),
                            &message_config,
                        ) {
                            Ok(result) => {
                                let package_list_file_location: String =
                                    format!("{output_directory}/{package_list_file_name}");

                                if result.sha256_hashes.len() != 0 {
                                    match result
                                        .sha256_hashes
                                        .get(&format!("{component}/binary-{architecture}/Packages"))
                                    {
                                        Some((hash, size)) => {
                                            if verify_file_checksum(
                                                "sha256",
                                                &package_list_file_location,
                                                &hash,
                                                &size,
                                                &message_config,
                                            )
                                            .is_err()
                                                == true
                                            {
                                                return Err(());
                                            };
                                        }
                                        None => {
                                            print_message(
                                                "error",
                                                &format!("failed to find checksum for file: \"{package_list_file_name}\""),
                                                &message_config,
                                            );
                                            return Err(());
                                        }
                                    };
                                } else if result.md5_hashes.len() != 0 {
                                    print_message(
                                        "warning",
                                        "falling back to using md5 checksum.",
                                        &message_config,
                                    );

                                    match result
                                        .md5_hashes
                                        .get(&format!("{component}/binary-{architecture}/Packages"))
                                    {
                                        Some((hash, size)) => {
                                            if verify_file_checksum(
                                                "md5",
                                                &package_list_file_location,
                                                &hash,
                                                &size,
                                                &message_config,
                                            )
                                            .is_err()
                                                == true
                                            {
                                                return Err(());
                                            };
                                        }
                                        None => {
                                            print_message(
                                                "error",
                                                &format!("failed to find checksum for file: \"{package_list_file_name}\""),
                                                &message_config,
                                            );
                                            return Err(());
                                        }
                                    };
                                } else {
                                    print_message(
                                        "error",
                                        &format!("failed to find any checksums in file: \"{output_directory}/{release_file_name}\""),
                                        &message_config,
                                    );
                                    return Err(());
                                };
                            }
                            Err(..) => return Err(()),
                        };
                    }
                }
            }
        }
    }

    return Ok(());
}
