use crate::functions::*;

use byte_unit::{Byte, Unit, UnitType};

#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Relationship {
    pub name: String,
    pub version: String,
    pub architecture: String,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub section: String,
    pub priority: String,
    pub depends: Vec<Vec<Relationship>>,
    pub pre_depends: Vec<Vec<Relationship>>,
    pub recommends: Vec<Vec<Relationship>>,
    pub suggests: Vec<Vec<Relationship>>,
    pub enhances: Vec<Vec<Relationship>>,
    pub breaks: Vec<Vec<Relationship>>,
    pub conflicts: Vec<Vec<Relationship>>,
    pub provides: Vec<Vec<Relationship>>,
    pub replaces: Vec<Vec<Relationship>>,
    pub is_essential: bool,
    pub is_build_essential: bool,
    pub file_name: String,
    pub file_size: u64,
    pub installed_size: u64,
    pub maintainer: String,
    pub description: String,
    pub homepage: String,
    pub origin_suite: String,
    pub origin_component: String,
    pub origin_architecture: String,
    pub origin_uri_scheme: String,
    pub origin_uri_path: String,
}

fn parse_relationships(prefix: &str, input: &str) -> Vec<Vec<Relationship>> {
    let mut output: Vec<Vec<Relationship>> = Vec::new();

    for entry in input
        .replacen(prefix, "", 1)
        .split(',')
        .map(|element| String::from(element.trim()))
        .collect::<Vec<String>>()
    {
        let mut relationship_enties: Vec<Relationship> = Vec::new();

        for alternative in entry
            .split("|")
            .map(|element| String::from(element.trim()))
            .collect::<Vec<String>>()
        {
            let mut relationship: Relationship = Default::default();

            let split_line: Vec<String> = alternative
                .replacen(")", "", 1)
                .split(" (")
                .map(|element| String::from(element.trim()))
                .collect::<Vec<String>>();

            if split_line[0].contains(":") == true {
                let split_string: Vec<String> = split_line[0]
                    .clone()
                    .split(':')
                    .map(|element| String::from(element))
                    .collect::<Vec<String>>();

                relationship.name = split_string[0].clone();
                relationship.architecture = split_string[1].clone();
            } else {
                relationship.name = split_line[0].clone();
            };

            if split_line.len() == 2 {
                relationship.version = split_line[1].clone();
            };

            relationship_enties.push(relationship);
        }

        output.push(relationship_enties);
    }

    return output;
}

impl Package {
    pub fn new(
        package_entries: &str,
        origin_suite: &str,
        origin_component: &str,
        origin_architecture: &str,
        origin_uri_scheme: &str,
        origin_uri_path: &str,
    ) -> Self {
        let mut name: String = String::new();
        let mut version: String = String::new();
        let mut architecture: String = String::new();
        let mut section: String = String::new();
        let mut priority: String = String::new();
        let mut depends: Vec<Vec<Relationship>> = Vec::new();
        let mut pre_depends: Vec<Vec<Relationship>> = Vec::new();
        let mut recommends: Vec<Vec<Relationship>> = Vec::new();
        let mut suggests: Vec<Vec<Relationship>> = Vec::new();
        let mut enhances: Vec<Vec<Relationship>> = Vec::new();
        let mut breaks: Vec<Vec<Relationship>> = Vec::new();
        let mut conflicts: Vec<Vec<Relationship>> = Vec::new();
        let mut provides: Vec<Vec<Relationship>> = Vec::new();
        let mut replaces: Vec<Vec<Relationship>> = Vec::new();
        let mut is_essential: bool = false;
        let mut is_build_essential: bool = false;
        let mut file_name: String = String::new();
        let mut file_size: u64 = 0;
        let mut installed_size: u64 = 0;
        let mut maintainer: String = String::new();
        let mut description: String = String::new();
        let mut homepage: String = String::new();

        for line in package_entries.lines() {
            match &line as &str {
                _ if line.starts_with("Package: ") => {
                    name = line.replacen("Package: ", "", 1);
                }
                _ if line.starts_with("Version: ") => {
                    version = line.replacen("Version: ", "", 1);
                }
                _ if line.starts_with("Architecture: ") => {
                    architecture = line.replacen("Architecture: ", "", 1);
                }
                _ if line.starts_with("Section: ") => {
                    section = line.replacen("Section: ", "", 1);
                }
                _ if line.starts_with("Priority: ") => {
                    priority = line.replacen("Priority: ", "", 1);
                }
                _ if line.starts_with("Depends: ") => {
                    depends = parse_relationships("Depends: ", line);
                }
                _ if line.starts_with("Pre-Depends: ") => {
                    pre_depends = parse_relationships("Pre-Depends: ", line);
                }
                _ if line.starts_with("Recommends: ") => {
                    recommends = parse_relationships("Recommends: ", line);
                }
                _ if line.starts_with("Suggests: ") => {
                    suggests = parse_relationships("Suggests: ", line);
                }
                _ if line.starts_with("Enhances: ") => {
                    enhances = parse_relationships("Enhances: ", line);
                }
                _ if line.starts_with("Breaks: ") => {
                    breaks = parse_relationships("Breaks: ", line);
                }
                _ if line.starts_with("Conflicts: ") => {
                    conflicts = parse_relationships("Conflicts: ", line);
                }
                _ if line.starts_with("Provides: ") => {
                    provides = parse_relationships("Provides: ", line);
                }
                _ if line.starts_with("Replaces: ") => {
                    replaces = parse_relationships("Replaces: ", line);
                }
                "Essential: yes" => {
                    is_essential = true;
                }
                "Build-Essential: yes" => {
                    is_build_essential = true;
                }
                _ if line.starts_with("Filename: ") => {
                    file_name = line.replacen("Filename: ", "", 1);
                }
                _ if line.starts_with("Size: ") => {
                    file_size = line.replacen("Size: ", "", 1).parse().unwrap();
                }
                _ if line.starts_with("Installed-Size: ") => {
                    installed_size = line.replacen("Installed-Size: ", "", 1).parse().unwrap();
                }
                _ if line.starts_with("Maintainer: ") => {
                    maintainer = line.replacen("Maintainer: ", "", 1);
                }
                _ if line.starts_with("Description: ") => {
                    description = line.replacen("Description: ", "", 1).replace("—", "-");
                }
                _ if line.starts_with("Homepage: ") => {
                    homepage = line.replacen("Homepage: ", "", 1);
                }
                _ => {}
            };
        }

        Self {
            name: name,
            version: version,
            architecture: architecture,
            section: section,
            priority: priority,
            depends: depends,
            pre_depends: pre_depends,
            recommends: recommends,
            suggests: suggests,
            enhances: enhances,
            breaks: breaks,
            conflicts: conflicts,
            provides: provides,
            replaces: replaces,
            is_essential: is_essential,
            is_build_essential: is_build_essential,
            file_name: file_name,
            file_size: file_size,
            installed_size: installed_size,
            maintainer: maintainer,
            description: description,
            homepage: homepage,
            origin_suite: String::from(origin_suite),
            origin_component: String::from(origin_component),
            origin_architecture: String::from(origin_architecture),
            origin_uri_scheme: String::from(origin_uri_scheme),
            origin_uri_path: String::from(origin_uri_path),
        }
    }
}

fn pretty_print_string(name: &str, input: &str, message_config: &MessageConfig) {
    if input.is_empty() == false {
        if message_config.color == true {
            println!("\x1b[01m{name}\x1b[00m: {input}");
        } else {
            println!("{name}: {input}");
        };
    };
}

fn pretty_print_relationships(
    name: &str,
    input: &Vec<Vec<Relationship>>,
    message_config: &MessageConfig,
) {
    if input.len() != 0 {
        if message_config.color == true {
            println!("\x1b[01m{name}\x1b[00m: [");
        } else {
            println!("{name}: [");
        };

        for value in input {
            let mut line_to_print: String = String::new();

            for (index, alternative) in value.iter().enumerate() {
                if index == 0 {
                    line_to_print = alternative.name.clone();
                } else {
                    line_to_print = format!("{line_to_print} | {}", alternative.name.clone());
                };

                if alternative.architecture.is_empty() == false {
                    line_to_print = format!("{line_to_print}:{}", alternative.architecture);
                };

                if alternative.version.is_empty() == false {
                    line_to_print = format!("{line_to_print} {}", alternative.version);
                };
            }

            println!("    {line_to_print}");
        }

        println!("]");
    };
}

pub fn pretty_print_package(package: &Package, message_config: &MessageConfig) {
    pretty_print_string("Package", &package.name, &message_config);
    pretty_print_string("Version", &package.version, &message_config);
    pretty_print_string("Architecture", &package.architecture, &message_config);
    pretty_print_string("Section", &package.section, &message_config);
    pretty_print_string("Priority", &package.priority, &message_config);
    pretty_print_relationships("Depends", &package.depends, &message_config);
    pretty_print_relationships("Pre-Depends", &package.pre_depends, &message_config);
    pretty_print_relationships("Recommends", &package.recommends, &message_config);
    pretty_print_relationships("Suggests", &package.suggests, &message_config);
    pretty_print_relationships("Enhances", &package.enhances, &message_config);
    pretty_print_relationships("Breaks", &package.breaks, &message_config);
    pretty_print_relationships("Conflicts", &package.conflicts, &message_config);
    pretty_print_relationships("Provides", &package.provides, &message_config);
    pretty_print_relationships("Replaces", &package.replaces, &message_config);
    if package.is_essential == true {
        pretty_print_string("Essential", "yes", &message_config);
    };
    if package.is_build_essential == true {
        pretty_print_string("Build-Essential", "yes", &message_config);
    };
    pretty_print_string(
        "File Size",
        &format!(
            "{:.2}",
            &Byte::from_u64_with_unit(package.file_size, Unit::B)
                .unwrap()
                .get_appropriate_unit(UnitType::Binary),
        ),
        &message_config,
    );
    pretty_print_string(
        "Installed Size",
        &format!(
            "{:.2}",
            &Byte::from_u64_with_unit(package.installed_size, Unit::KiB)
                .unwrap()
                .get_appropriate_unit(UnitType::Binary),
        ),
        &message_config,
    );
    pretty_print_string("Maintainer", &package.maintainer, &message_config);
    pretty_print_string("Description", &package.description, &message_config);
    pretty_print_string("Homepage", &package.homepage, &message_config);
    pretty_print_string("Origin Suite", &package.origin_suite, &message_config);
    pretty_print_string(
        "Origin Component",
        &package.origin_component,
        &message_config,
    );
    pretty_print_string(
        "Origin Architecture",
        &package.origin_architecture,
        &message_config,
    );
    pretty_print_string(
        "Origin URI",
        &format!(
            "{}{}/{}",
            package.origin_uri_scheme, package.origin_uri_path, package.file_name
        ),
        &message_config,
    );
}
