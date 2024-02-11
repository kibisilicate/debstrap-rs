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
    pub section: String,
    pub priority: String,
    pub uri: String,
    pub suite: String,
    pub component: String,
    pub architecture: String,
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
    pub file_size: u32,
    pub file_name: String,
    pub maintainer: String,
    pub description: String,
    pub homepage: String,
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
        package_uri: &str,
        package_suite: &str,
        package_component: &str,
        package_architecture: &str,
    ) -> Self {
        let mut name: String = String::new();
        let mut version: String = String::new();
        let mut section: String = String::new();
        let mut priority: String = String::new();
        let uri: String = String::from(package_uri);
        let suite: String = String::from(package_suite);
        let component: String = String::from(package_component);
        let mut architecture: String = String::from(package_architecture);
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
        let mut file_size: u32 = 0;
        let mut file_name: String = String::new();
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
                _ if line.starts_with("Section: ") => {
                    section = line.replacen("Section: ", "", 1);
                }
                _ if line.starts_with("Priority: ") => {
                    priority = line.replacen("Priority: ", "", 1);
                }
                _ if line.starts_with("Architecture: ") => {
                    architecture = line.replacen("Architecture: ", "", 1);
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
                _ if line.starts_with("Size: ") => {
                    file_size = line.replacen("Size: ", "", 1).parse().unwrap();
                }
                _ if line.starts_with("Filename: ") => {
                    file_name = line.replacen("Filename: ", "", 1);
                }
                _ if line.starts_with("Maintainer: ") => {
                    maintainer = line.replacen("Maintainer: ", "", 1);
                }
                _ if line.starts_with("Description: ") => {
                    description = line.replacen("Description: ", "", 1);
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
            section: section,
            priority: priority,
            uri: uri,
            suite: suite,
            component: component,
            architecture: architecture,
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
            file_size: file_size,
            file_name: file_name,
            maintainer: maintainer,
            description: description,
            homepage: homepage,
        }
    }
}
