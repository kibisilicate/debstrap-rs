use crate::functions::print_message;
use crate::functions::MessageConfig;
use crate::functions::TargetPackage;

pub fn resolve_dependencies(
    package_list: &Vec<String>,
    input_package_set: &Vec<String>,
    mirror_uri: &String,
    consider_recommends: &bool,
    packages_to_prohibit: &Vec<String>,
    message_config: &MessageConfig,
) -> Result<Vec<TargetPackage>, ()> {
    let mut list_of_package_names: Vec<String> = Vec::new();

    #[derive(Debug)]
    struct PackageProvides {
        name: String,
        provides: Vec<String>,
    }

    let mut provides_list: Vec<PackageProvides> = Vec::new();

    for entry in package_list {
        let lines_from_entry: Vec<String> = entry
            .lines()
            .map(|element| String::from(element.trim()))
            .collect::<Vec<String>>();

        for line in &lines_from_entry {
            if line.starts_with("Package: ") == true {
                list_of_package_names.push(line.replacen("Package: ", "", 1));
            };

            if line.starts_with("Provides: ") == true {
                let mut provides: Vec<String> = Vec::new();

                for entry in line
                    .replacen("Provides: ", "", 1)
                    .split(',')
                    .map(|element| String::from(element.trim()))
                    .collect::<Vec<String>>()
                {
                    let package: Vec<&str> = entry
                        .split_whitespace()
                        .map(|element| element.trim())
                        .collect::<Vec<&str>>();

                    provides.push(String::from(package[0]));
                }

                let current_package_provides: PackageProvides = PackageProvides {
                    name: lines_from_entry[0].clone().replacen("Package: ", "", 1),
                    provides: provides,
                };

                provides_list.push(current_package_provides);
            };
        }
    }

    list_of_package_names.sort_unstable();
    list_of_package_names.dedup();

    let list_of_package_names: Vec<String> = list_of_package_names;

    let provides_list: Vec<PackageProvides> = provides_list;

    //////////////////////////////////////////////

    let initial_packages: Vec<String> = input_package_set.clone();
    let mut current_packages: Vec<String> = input_package_set.clone();
    let mut total_accumulated_packages: Vec<String> = Vec::new();

    let mut iteration_counter: u16 = 0;

    loop {
        if current_packages.len() == 0 {
            break;
        };

        for package in current_packages.iter_mut() {
            if package.contains(':') == true {
                let split_colon: Vec<&str> =
                    package.split(':').map(|element| element.trim()).collect();

                *package = String::from(split_colon[0]);
            };
        }

        if packages_to_prohibit.len() != 0 {
            for prohibited in packages_to_prohibit {
                for package in current_packages.iter_mut() {
                    if package == prohibited {
                        print_message(
                            "debug",
                            &format!("excluding prohibited package: \"{package}\""),
                            &message_config,
                        );

                        *package = String::new();

                        break;
                    };
                }
            }
        };

        for provider in &provides_list {
            if total_accumulated_packages.contains(&provider.name) == true {
                for provided in &provider.provides {
                    if total_accumulated_packages.contains(provided) == true {
                        for package in total_accumulated_packages.iter_mut() {
                            if package == provided {
                                if initial_packages.contains(&*package) == false {
                                    print_message(
                                        "debug",
                                        &format!(
                                            "package \"{package}\" is provided by \"{}\"",
                                            provider.name,
                                        ),
                                        &message_config,
                                    );

                                    *package = provider.name.clone();
                                };

                                break;
                            };
                        }
                    };
                }
            };
        }

        total_accumulated_packages.sort_unstable();
        total_accumulated_packages.dedup();

        while total_accumulated_packages.contains(&String::new()) == true {
            total_accumulated_packages.remove(0);
        }

        for package in total_accumulated_packages.iter_mut() {
            if list_of_package_names.contains(&*package) == false {
                for provider in &provides_list {
                    for provided in &provider.provides {
                        if provided.contains(&*package) == true {
                            print_message(
                                "debug",
                                &format!(
                                    "package \"{package}\" is provided by \"{}\"",
                                    provider.name,
                                ),
                                &message_config,
                            );

                            *package = provider.name.clone();

                            break;
                        };
                    }
                }
            };

            if list_of_package_names.contains(&*package) == false {
                print_message(
                    "error",
                    &format!("failed to find package: \"{package}\""),
                    &message_config,
                );

                return Err(());
            };
        }

        current_packages.sort_unstable();
        current_packages.dedup();

        while current_packages.contains(&String::new()) == true {
            current_packages.remove(0);
        }

        total_accumulated_packages.extend(current_packages.clone());

        total_accumulated_packages.sort_unstable();
        total_accumulated_packages.dedup();

        while total_accumulated_packages.contains(&String::new()) == true {
            total_accumulated_packages.remove(0);
        }

        print_message(
            "debug",
            &format!(
                "iteration: \"{iteration_counter}\", amount: \"{}\"",
                current_packages.len()
            ),
            &message_config,
        );

        iteration_counter += 1;

        let mut packages_dependencies: Vec<String> = Vec::new();

        for entry in package_list {
            let mut package_name: String = String::new();

            for line in entry.lines() {
                if line.starts_with("Package: ") == true {
                    package_name = line.replacen("Package: ", "", 1);

                    break;
                };
            }

            if current_packages.contains(&package_name) == true {
                for line in entry.lines() {
                    if line.starts_with("Depends: ") {
                        packages_dependencies.extend(
                            line.replacen("Depends: ", "", 1)
                                .split(',')
                                .map(|element| String::from(element.trim()))
                                .collect::<Vec<String>>(),
                        );
                    };

                    if line.starts_with("Pre-Depends: ") {
                        packages_dependencies.extend(
                            line.replacen("Pre-Depends: ", "", 1)
                                .split(',')
                                .map(|element| String::from(element.trim()))
                                .collect::<Vec<String>>(),
                        );
                    };

                    if *consider_recommends == true {
                        if line.starts_with("Recommends: ") {
                            packages_dependencies.extend(
                                line.replacen("Recommends: ", "", 1)
                                    .split(',')
                                    .map(|element| String::from(element.trim()))
                                    .collect::<Vec<String>>(),
                            );
                        };
                    };
                }
            };
        }

        for value in packages_dependencies.iter_mut() {
            let name: Vec<&str> = value
                .split_whitespace()
                .map(|element| element.trim())
                .collect::<Vec<&str>>();

            *value = String::from(name[0]);
        }

        packages_dependencies.sort_unstable();
        packages_dependencies.dedup();

        for dependency in packages_dependencies.iter_mut() {
            if total_accumulated_packages.contains(&*dependency) == true {
                *dependency = String::new();
            };
        }

        packages_dependencies.sort_unstable();
        packages_dependencies.dedup();

        while packages_dependencies.contains(&String::new()) == true {
            packages_dependencies.remove(0);
        }

        current_packages = packages_dependencies;
    }

    if total_accumulated_packages.len() == 0 {
        print_message("error", "failed to resolve dependencies.", &message_config);

        return Err(());
    };

    let mut target_package_set: Vec<TargetPackage> = Vec::new();

    for package in &total_accumulated_packages {
        let mut version: String = String::new();
        let mut architecture: String = String::new();
        let mut description: String = String::new();
        let mut file_size: f64 = 0.0;
        let mut file_name: String = String::new();

        for entry in package_list {
            if entry.starts_with(&format!("Package: {package}\n")) == true {
                for line in entry
                    .lines()
                    .map(|element| String::from(element.trim()))
                    .collect::<Vec<String>>()
                {
                    if line.starts_with(&format!("Version: ")) == true {
                        version = line.replacen("Version: ", "", 1);
                    } else if line.starts_with(&format!("Architecture: ")) == true {
                        architecture = line.replacen("Architecture: ", "", 1);
                    } else if line.starts_with(&format!("Description: ")) == true {
                        description = line.replacen("Description: ", "", 1);
                    } else if line.starts_with(&format!("Size: ")) == true {
                        file_size = line.replacen("Size: ", "", 1).parse().unwrap();
                    } else if line.starts_with(&format!("Filename: ")) == true {
                        file_name = line.replacen("Filename: ", "", 1);
                    };
                }

                break;
            };
        }

        let target_package: TargetPackage = TargetPackage {
            name: String::from(package),
            version: version,
            architecture: architecture,
            description: description,
            file_size: file_size,
            file_name: file_name,
            uri: String::from(mirror_uri),
        };

        target_package_set.push(target_package);
    }

    return Ok(target_package_set);
}
