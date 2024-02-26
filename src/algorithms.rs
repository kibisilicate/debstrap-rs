use crate::functions::print_message;
use crate::functions::MessageConfig;
use crate::package::Package;

use std::collections::HashMap;

pub fn resolve_dependencies(
    package_database: &HashMap<String, Vec<Package>>,
    input_package_set: &Vec<Package>,
    consider_recommends: &bool,
    packages_to_prohibit: &Vec<String>,
    message_config: &MessageConfig,
) -> Result<Vec<Package>, ()> {
    #[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
    struct Provider {
        package: Package,
        provides: Vec<String>,
    }

    let mut provides_list: Vec<Provider> = Vec::new();

    for package in package_database.values() {
        if package[0].provides.len() != 0 {
            let mut provides: Vec<String> = Vec::new();

            for provided in &package[0].provides {
                provides.push(provided[0].name.clone());
            }

            let provider: Provider = Provider {
                package: package[0].clone(),
                provides: provides,
            };

            provides_list.push(provider);
        };
    }

    provides_list.sort_unstable();
    provides_list.dedup();

    let provides_list: Vec<Provider> = provides_list;

    //////////////////////////////////////////////

    let mut current_packages: Vec<Package> = input_package_set.clone();

    let mut accumulated_packages: Vec<Package> = Vec::new();

    let mut iteration_counter: u16 = 0;

    loop {
        if current_packages.len() == 0 {
            break;
        };

        if packages_to_prohibit.len() != 0 {
            for prohibited in packages_to_prohibit {
                for (index, current) in current_packages.iter_mut().enumerate() {
                    if current.name == *prohibited {
                        print_message(
                            "debug",
                            &format!("excluding prohibited package: \"{}\"", current.name),
                            &message_config,
                        );

                        current_packages.remove(index);

                        break;
                    };
                }
            }
        };

        current_packages.sort_unstable();
        current_packages.dedup();

        accumulated_packages.extend(current_packages.clone());
        accumulated_packages.sort_unstable();
        accumulated_packages.dedup();

        print_message(
            "debug",
            &format!(
                "iteration: \"{iteration_counter}\", amount: \"{}\"",
                current_packages.len(),
            ),
            &message_config,
        );

        iteration_counter += 1;

        let mut dependency_packages: Vec<Package> = Vec::new();

        for current in current_packages {
            let mut packages_not_found: Vec<String> = Vec::new();

            for dependency in current.depends {
                match package_database.get(&dependency[0].name) {
                    Some(result) => {
                        dependency_packages.push(result[0].clone());
                    }
                    None => {
                        packages_not_found.push(dependency[0].name.clone());
                    }
                };
            }

            for dependency in current.pre_depends {
                match package_database.get(&dependency[0].name) {
                    Some(result) => {
                        dependency_packages.push(result[0].clone());
                    }
                    None => {
                        packages_not_found.push(dependency[0].name.clone());
                    }
                };
            }

            if *consider_recommends == true {
                for dependency in current.recommends {
                    match package_database.get(&dependency[0].name) {
                        Some(result) => {
                            dependency_packages.push(result[0].clone());
                        }
                        None => {
                            packages_not_found.push(dependency[0].name.clone());
                        }
                    };
                }
            };

            for package in packages_not_found {
                let mut was_package_found: bool = false;

                for provider in &provides_list {
                    if provider.provides.contains(&package) == true {
                        if accumulated_packages.contains(&provider.package) {
                            print_message(
                                "debug",
                                &format!(
                                    "package \"{}\" is provided by \"{}\"",
                                    package, provider.package.name,
                                ),
                                &message_config,
                            );

                            was_package_found = true;

                            break;
                        };
                    };
                }

                if was_package_found == false {
                    for provider in &provides_list {
                        if provider.provides.contains(&package) == true {
                            if accumulated_packages.contains(&provider.package) == false {
                                dependency_packages.push(provider.package.clone());

                                print_message(
                                    "debug",
                                    &format!(
                                        "package \"{}\" is provided by \"{}\"",
                                        package, provider.package.name,
                                    ),
                                    &message_config,
                                );

                                was_package_found = true;

                                break;
                            };
                        };
                    }
                };

                if was_package_found == false {
                    print_message(
                        "error",
                        &format!("failed to find package: \"{package}\""),
                        &message_config,
                    );
                    return Err(());
                };
            }
        }

        dependency_packages.sort_unstable();
        dependency_packages.dedup();

        current_packages = Vec::new();

        for dependency in dependency_packages {
            if accumulated_packages.contains(&dependency) == false {
                current_packages.push(dependency);
            };
        }
    }

    if accumulated_packages.len() == 0 {
        print_message("error", "failed to resolve dependencies.", &message_config);
        return Err(());
    };

    return Ok(accumulated_packages);
}
