use crate::functions::*;
use crate::package::*;

use std::collections::HashMap;
use std::path::Path;

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
                let mut was_dependency_found: bool = false;

                for alternative in &dependency {
                    match package_database.get(&alternative.name) {
                        Some(result) => {
                            dependency_packages.push(result[0].clone());
                            was_dependency_found = true;
                            break;
                        }
                        None => {}
                    };
                }

                if was_dependency_found == false {
                    packages_not_found.push(dependency[0].name.clone());
                };
            }

            for dependency in current.pre_depends {
                let mut was_dependency_found: bool = false;

                for alternative in &dependency {
                    match package_database.get(&alternative.name) {
                        Some(result) => {
                            dependency_packages.push(result[0].clone());
                            was_dependency_found = true;
                            break;
                        }
                        None => {}
                    };
                }

                if was_dependency_found == false {
                    packages_not_found.push(dependency[0].name.clone());
                };
            }

            if *consider_recommends == true {
                for dependency in current.recommends {
                    let mut was_dependency_found: bool = false;

                    for alternative in &dependency {
                        match package_database.get(&alternative.name) {
                            Some(result) => {
                                dependency_packages.push(result[0].clone());
                                was_dependency_found = true;
                                break;
                            }
                            None => {}
                        };
                    }

                    if was_dependency_found == false {
                        packages_not_found.push(dependency[0].name.clone());
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
                                    "package: \"{}\" is provided by: \"{}\"",
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
                                        "package: \"{}\" is provided by: \"{}\"",
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

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn separate_packages_by_priority(
    package_database: &HashMap<String, Vec<Package>>,
    input_directory: &str,
    target_package_set: &Vec<Package>,
    packages_to_prohibit: &Vec<String>,
    packages_to_consider_essential: &Vec<String>,
    packages_to_consider_non_essential: &Vec<String>,
    target_extractor: &str,
    output_directory: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    let mut initial_essential_subset: Vec<Package> = Vec::new();
    let mut initial_required_subset: Vec<Package> = Vec::new();
    let mut initial_important_subset: Vec<Package> = Vec::new();
    let mut initial_standard_subset: Vec<Package> = Vec::new();

    for package in target_package_set {
        if package.is_essential == true
            || package.name == "usr-is-merged"
            || package.name == "usrmerge"
        {
            initial_essential_subset.push(package.clone());
        };

        if package.priority == "required" || package.name == "apt" {
            initial_required_subset.push(package.clone());
        };

        if package.priority == "important" {
            initial_important_subset.push(package.clone());
        };

        if package.priority == "standard" {
            initial_standard_subset.push(package.clone());
        };
    }

    let mut is_awk_present: bool = false;

    if is_awk_present == false {
        for package in target_package_set {
            if package.name == "mawk" {
                initial_essential_subset.push(package.clone());
                is_awk_present = true;
                break;
            };
        }
    };

    if is_awk_present == false {
        for package in target_package_set {
            if package.name == "original-awk" {
                initial_essential_subset.push(package.clone());
                is_awk_present = true;
                break;
            };
        }
    };

    if is_awk_present == false {
        for package in target_package_set {
            if package.name == "gawk" {
                initial_essential_subset.push(package.clone());
                is_awk_present = true;
                break;
            };
        }
    };

    if is_awk_present == false {
        print_message(
            "error",
            "no packages that provide: \"awk\" are present.",
            &message_config,
        );
        return Err(());
    };

    initial_essential_subset.sort_unstable();
    initial_essential_subset.dedup();

    initial_required_subset.sort_unstable();
    initial_required_subset.dedup();

    initial_important_subset.sort_unstable();
    initial_important_subset.dedup();

    initial_standard_subset.sort_unstable();
    initial_standard_subset.dedup();

    //////////////////////////////////////////////

    if packages_to_consider_essential.len() != 0 {
        for included in packages_to_consider_essential {
            if initial_essential_subset
                .iter()
                .any(|initial| &initial.name == included)
                == true
            {
                print_message(
                    "debug",
                    &format!("essential package present: \"{included}\""),
                    &message_config,
                );
            } else {
                let mut is_package_present: bool = false;

                for package in package_database.values() {
                    if package[0].name == *included {
                        initial_essential_subset.push(package[0].clone());

                        print_message(
                            "debug",
                            &format!("essential package added:   \"{included}\""),
                            &message_config,
                        );

                        is_package_present = true;

                        break;
                    };
                }

                if is_package_present == false {
                    print_message(
                        "error",
                        &format!("failed to find package: \"{included}\""),
                        &message_config,
                    );

                    return Err(());
                };
            };
        }

        initial_essential_subset.sort_unstable();
    };

    if packages_to_consider_non_essential.len() != 0 {
        for excluded in packages_to_consider_non_essential {
            if initial_essential_subset
                .iter()
                .any(|initial| &initial.name == excluded)
                == true
            {
                for (index, initial) in initial_essential_subset.iter_mut().enumerate() {
                    if initial.name == *excluded {
                        initial_essential_subset.remove(index);

                        print_message(
                            "debug",
                            &format!("essential package removed: \"{excluded}\""),
                            &message_config,
                        );

                        break;
                    };
                }
            } else {
                print_message(
                    "debug",
                    &format!("essential package absent:  \"{excluded}\""),
                    &message_config,
                );
            };
        }

        initial_essential_subset.sort_unstable();
    };

    //////////////////////////////////////////////

    let target_essential_subset: Vec<Package>;
    let mut target_required_subset: Vec<Package> = Vec::new();
    let mut target_important_subset: Vec<Package> = Vec::new();
    let mut target_standard_subset: Vec<Package> = Vec::new();

    let quiet_message_config: MessageConfig = MessageConfig {
        color: message_config.color,
        debug: false,
    };

    print_message("debug", "calculating essential subset.", &message_config);

    match resolve_dependencies(
        &package_database,
        &initial_essential_subset,
        &false,
        &packages_to_prohibit,
        &quiet_message_config,
    ) {
        Ok(result) => target_essential_subset = result,
        Err(..) => return Err(()),
    };

    if initial_required_subset.len() != 0 {
        print_message("debug", "calculating required subset.", &message_config);

        match resolve_dependencies(
            &package_database,
            &initial_required_subset,
            &false,
            &packages_to_prohibit,
            &quiet_message_config,
        ) {
            Ok(result) => target_required_subset = result,
            Err(..) => return Err(()),
        };
    };

    if initial_important_subset.len() != 0 {
        print_message("debug", "calculating important subset.", &message_config);

        match resolve_dependencies(
            &package_database,
            &initial_important_subset,
            &false,
            &packages_to_prohibit,
            &quiet_message_config,
        ) {
            Ok(result) => target_important_subset = result,
            Err(..) => return Err(()),
        };
    };

    if initial_standard_subset.len() != 0 {
        print_message("debug", "calculating standard subset.", &message_config);

        match resolve_dependencies(
            &package_database,
            &initial_standard_subset,
            &false,
            &packages_to_prohibit,
            &quiet_message_config,
        ) {
            Ok(result) => target_standard_subset = result,
            Err(..) => return Err(()),
        };
    };

    let target_required_subset: Vec<Package> = target_required_subset;
    let target_important_subset: Vec<Package> = target_important_subset;
    let target_standard_subset: Vec<Package> = target_standard_subset;

    //////////////////////////////////////////////

    let output_sub_directories: Vec<String> = Vec::from([
        String::from("essential"),
        String::from("required"),
        String::from("important"),
        String::from("standard"),
        String::from("remaining"),
    ]);

    println!("Separating packages by priority ...");

    for sub_directory in &output_sub_directories {
        let mut downloaded_package_file_names: Vec<String> = std::fs::read_dir(&input_directory)
            .unwrap()
            .map(|element| {
                String::from(
                    element
                        .unwrap()
                        .path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy(),
                )
            })
            .collect::<Vec<String>>();

        downloaded_package_file_names.sort_unstable();

        for package in &downloaded_package_file_names {
            let mut package_name: String = String::new();

            match extract_deb_control_field(
                &target_extractor,
                &format!("{input_directory}/{package}"),
                &message_config,
            ) {
                Ok(result) => {
                    for line in result.lines() {
                        if line.starts_with("Package: ") == true {
                            package_name = line.replacen("Package: ", "", 1);
                            break;
                        };
                    }
                }
                Err(..) => {
                    return Err(());
                }
            };

            let mut is_package_in_subset: bool = false;

            match sub_directory as &str {
                "essential" => {
                    if target_essential_subset
                        .iter()
                        .any(|essential| essential.name == package_name)
                        == true
                    {
                        is_package_in_subset = true;
                    };
                }
                "required" => {
                    if target_required_subset
                        .iter()
                        .any(|required| required.name == package_name)
                        == true
                    {
                        is_package_in_subset = true;
                    };
                }
                "important" => {
                    if target_important_subset
                        .iter()
                        .any(|important| important.name == package_name)
                        == true
                    {
                        is_package_in_subset = true;
                    };
                }
                "standard" => {
                    if target_standard_subset
                        .iter()
                        .any(|standard| standard.name == package_name)
                        == true
                    {
                        is_package_in_subset = true;
                    };
                }
                _ => {}
            };

            if is_package_in_subset == true {
                if Path::new(&format!("{output_directory}/{sub_directory}")).exists() == false {
                    if create_directory(
                        &format!("{output_directory}/{sub_directory}"),
                        &message_config,
                    )
                    .is_err()
                        == true
                    {
                        return Err(());
                    };
                };

                print_message(
                    "debug",
                    &format!("moving {sub_directory} package: \"{package}\" to \"{output_directory}/{sub_directory}\""),
                    &message_config,
                );

                if std::fs::rename(
                    format!("{input_directory}/{package}"),
                    format!("{output_directory}/{sub_directory}/{package}"),
                )
                .is_err()
                    == true
                {
                    print_message(
                        "error",
                        &format!("failed to move file: \"{input_directory}/{package}\""),
                        &message_config,
                    );

                    return Err(());
                };
            };
        }
    }

    let mut remaining_package_file_names: Vec<String> = std::fs::read_dir(&input_directory)
        .unwrap()
        .map(|element| {
            String::from(
                element
                    .unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy(),
            )
        })
        .collect::<Vec<String>>();

    remaining_package_file_names.sort_unstable();

    if remaining_package_file_names.len() != 0 {
        if create_directory(
            &format!("{output_directory}/{}", &output_sub_directories[4]),
            &message_config,
        )
        .is_err()
            == true
        {
            return Err(());
        };

        for package in &remaining_package_file_names {
            print_message(
                "debug",
                &format!(
                    "moving remaining package: \"{package}\" to \"{}\"",
                    format!("{output_directory}/{}", &output_sub_directories[4])
                ),
                &message_config,
            );

            if std::fs::rename(
                format!("{input_directory}/{package}"),
                format!(
                    "{}/{package}",
                    format!("{output_directory}/{}", &output_sub_directories[4])
                ),
            )
            .is_err()
                == true
            {
                print_message(
                    "error",
                    &format!("failed to move file: \"{input_directory}/{package}\""),
                    &message_config,
                );

                return Err(());
            };
        }
    };

    return Ok(());
}
