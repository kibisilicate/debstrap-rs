use crate::functions::*;
use crate::releases::*;

use cmd_lib::run_fun;

#[derive(Debug, Default, Clone)]
pub struct SourcesEntry {
    pub uris: Vec<(String, String)>,
    pub suites: Vec<String>,
    pub components: Vec<String>,
    pub architectures: Vec<String>,
}

pub fn parse_sources_file(
    sources_file: &str,
    message_config: &MessageConfig,
) -> Result<Vec<SourcesEntry>, ()> {
    let mut sources_list: Vec<SourcesEntry> = Vec::new();

    match std::fs::read_to_string(sources_file) {
        Ok(result) => {
            let sources_entries: Vec<String> = result
                .trim()
                .split("\n\n")
                .map(|element| String::from(element))
                .collect::<Vec<String>>();

            for entry in sources_entries {
                let mut entries_uris: Vec<String> = Vec::new();
                let mut entries_suites: Vec<String> = Vec::new();
                let mut entries_components: Vec<String> = Vec::new();
                let mut entries_architectures: Vec<String> = Vec::new();

                for line in entry.lines() {
                    match &line as &str {
                        _ if line.starts_with("URIs: ") => {
                            entries_uris = parse_list_of_values("URIs: ", &line);
                        }
                        _ if line.starts_with("Suites: ") => {
                            entries_suites = parse_list_of_values("Suites: ", &line);
                        }
                        _ if line.starts_with("Components: ") => {
                            entries_components = parse_list_of_values("Components: ", &line);
                        }
                        _ if line.starts_with("Architectures: ") => {
                            entries_architectures = parse_list_of_values("Architectures: ", &line);
                        }
                        _ => {}
                    };
                }

                //////////////////////////////////

                if entries_uris.len() == 0 {
                    print_message(
                        "error",
                        &format!("no URI(s) were provided."),
                        &message_config,
                    );
                    return Err(());
                };

                let mut parsed_uris: Vec<(String, String)> = Vec::new();

                for uri in entries_uris {
                    match parse_uri(&uri) {
                        Ok(result) => {
                            parsed_uris.push(result);
                        }
                        Err(..) => {
                            print_message(
                                "error",
                                &format!("invalid URI: \"{uri}\""),
                                &message_config,
                            );
                            return Err(());
                        }
                    };
                }

                //////////////////////////////////

                if entries_suites.len() == 0 {
                    print_message(
                        "error",
                        &format!("no suite(s) were provided."),
                        &message_config,
                    );
                    return Err(());
                };

                let mut parsed_suites: Vec<String> = Vec::new();

                for (index, suite) in entries_suites.iter().enumerate() {
                    if index == 0 {
                        if check_primary_suite(suite) == false {
                            print_message(
                                "error",
                                &format!("unrecognized suite: \"{suite}\""),
                                &message_config,
                            );
                            return Err(());
                        };
                    };

                    if parsed_suites.contains(suite) == false {
                        parsed_suites.push(String::from(suite));
                    };
                }

                //////////////////////////////////

                if entries_components.len() == 0 {
                    print_message(
                        "error",
                        &format!("no component(s) were provided."),
                        &message_config,
                    );
                    return Err(());
                };

                let mut parsed_components: Vec<String> = Vec::new();

                for (index, component) in entries_components.iter().enumerate() {
                    if index == 0 {
                        if component != "main" {
                            print_message(
                                "error",
                                &format!("invalid first component: \"{component}\""),
                                &message_config,
                            );
                            return Err(());
                        };
                    };

                    if parsed_components.contains(component) == false {
                        parsed_components.push(String::from(component));
                    };
                }

                //////////////////////////////////

                // If no architecture(s) is provided then assume the same as the host.

                if entries_architectures.len() == 0 {
                    entries_architectures =
                        vec![String::from(run_fun!(uname "--machine").unwrap())];
                };

                let mut parsed_architectures: Vec<String> = Vec::new();

                for architecture in entries_architectures {
                    match get_debian_architecture_name(&architecture) {
                        Ok(result) => {
                            if parsed_architectures.contains(&result) == false {
                                parsed_architectures.push(result);
                            };
                        }
                        Err(..) => {
                            print_message(
                                "error",
                                &format!("unrecognized architecture: \"{architecture}\""),
                                &message_config,
                            );
                            return Err(());
                        }
                    };
                }

                //////////////////////////////////

                let sources_entry: SourcesEntry = SourcesEntry {
                    uris: parsed_uris,
                    suites: parsed_suites,
                    components: parsed_components,
                    architectures: parsed_architectures,
                };

                sources_list.push(sources_entry);
            }
        }
        Err(..) => {
            print_message(
                "error",
                &format!("failed to read sources file: \"{sources_file}\""),
                &message_config,
            );
            return Err(());
        }
    };

    return Ok(sources_list);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn create_sources_list(
    input_suites: &Vec<String>,
    input_components: &Vec<String>,
    input_architectures: &Vec<String>,
    input_uris: &Vec<String>,
    message_config: &MessageConfig,
) -> Result<Vec<SourcesEntry>, ()> {
    if input_suites.len() == 0 {
        print_message(
            "error",
            &format!("no suite(s) were provided."),
            &message_config,
        );
        return Err(());
    };

    let mut parsed_suites: Vec<String> = Vec::new();

    for (index, suite) in input_suites.iter().enumerate() {
        if index == 0 {
            if check_primary_suite(suite) == false {
                print_message(
                    "error",
                    &format!("unrecognized suite: \"{suite}\""),
                    &message_config,
                );
                return Err(());
            };
        };

        if parsed_suites.contains(suite) == false {
            parsed_suites.push(String::from(suite));
        };
    }

    //////////////////////////////////////////////

    // If no component(s) is provided then assume 'main'

    let mut input_components: Vec<String> = input_components.to_vec();

    if input_components.len() == 0 {
        input_components = vec![String::from("main")];
    };

    let mut parsed_components: Vec<String> = Vec::new();

    for (index, component) in input_components.iter().enumerate() {
        if index == 0 {
            if component != "main" {
                print_message(
                    "error",
                    &format!("invalid first component: \"{component}\""),
                    &message_config,
                );
                return Err(());
            };
        };

        if parsed_components.contains(component) == false {
            parsed_components.push(String::from(component));
        };
    }

    //////////////////////////////////////////////

    // If no architecture(s) is provided then assume the same as the host.

    let mut input_architectures: Vec<String> = input_architectures.to_vec();

    if input_architectures.len() == 0 {
        input_architectures = vec![String::from(run_fun!(uname "--machine").unwrap())];
    };

    let mut parsed_architectures: Vec<String> = Vec::new();

    for architecture in input_architectures {
        match get_debian_architecture_name(&architecture) {
            Ok(result) => {
                if parsed_architectures.contains(&result) == false {
                    parsed_architectures.push(result);
                };
            }
            Err(..) => {
                print_message(
                    "error",
                    &format!("unrecognized architecture: \"{architecture}\""),
                    &message_config,
                );
                return Err(());
            }
        };
    }

    //////////////////////////////////////////////

    let mut input_uris: Vec<String> = input_uris.to_vec();

    if input_uris.len() == 0 {
        for uri in default_uris(&parsed_suites[0], &parsed_architectures[0]) {
            input_uris.push(uri);
        }
    };

    let mut parsed_uris: Vec<(String, String)> = Vec::new();

    for uri in input_uris {
        match parse_uri(&uri) {
            Ok(result) => {
                parsed_uris.push(result);
            }
            Err(..) => {
                print_message("error", &format!("invalid URI: \"{uri}\""), &message_config);
                return Err(());
            }
        };
    }

    //////////////////////////////////////////////

    let sources_list: Vec<SourcesEntry> = vec![SourcesEntry {
        uris: parsed_uris,
        suites: parsed_suites,
        components: parsed_components,
        architectures: parsed_architectures,
    }];

    return Ok(sources_list);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn parse_uri(uri_to_parse: &str) -> Result<(String, String), ()> {
    let uri_scheme: String;
    let mut uri_path: String;

    match uri_to_parse {
        uri if uri.starts_with("http://") => {
            uri_scheme = String::from("http://");
            uri_path = uri_to_parse.replacen("http://", "", 1);
        }
        uri if uri.starts_with("https://") => {
            uri_scheme = String::from("https://");
            uri_path = uri_to_parse.replacen("https://", "", 1);
        }
        _ => return Err(()),
    };

    while uri_path.contains("//") == true {
        uri_path = uri_path.replace("//", "/");
    }

    if uri_path.starts_with("/") == true {
        uri_path = String::from(uri_path.strip_prefix("/").unwrap());
    };

    if uri_path.ends_with("/") == true {
        uri_path = String::from(uri_path.strip_suffix("/").unwrap());
    };

    return Ok((uri_scheme, uri_path));
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn create_sources_list_file(
    sources_list: &Vec<SourcesEntry>,
    sources_signed_by: &str,
    format: &str,
    output_directory: &str,
    message_config: &MessageConfig,
) -> Result<(), ()> {
    match &format as &str {
        "deb822-style" => {
            print_message(
                "debug",
                &format!("creating default deb822-style sources list: \"{output_directory}/sources.sources\""),
                &message_config,
            );

            if create_file(
                &format!("{output_directory}/sources.sources"),
                "",
                &message_config,
            )
            .is_err()
                == true
            {
                return Err(());
            };

            for (index, entry) in sources_list.iter().enumerate() {
                let mut full_uris: Vec<String> = Vec::new();

                for (scheme, path) in &entry.uris {
                    full_uris.push(format!("{scheme}{path}"));
                }

                if index != 0 {
                    if append_file(
                        &format!("{output_directory}/sources.sources"),
                        "\n",
                        &message_config,
                    )
                    .is_err()
                        == true
                    {
                        return Err(());
                    };
                };

                if append_file(
                    &format!("{output_directory}/sources.sources"),
                    &format!(
                        "\
Types: deb deb-src
URIs: {}
Suites: {}
Components: {}
Signed-By: /usr/share/keyrings/{}
",
                        format!("{:?}", &full_uris).replace(['[', ']', '"', ','], ""),
                        format!("{:?}", &entry.suites).replace(['[', ']', '"', ','], ""),
                        format!("{:?}", &entry.components).replace(['[', ']', '"', ','], ""),
                        sources_signed_by,
                    ),
                    &message_config,
                )
                .is_err()
                    == true
                {
                    return Err(());
                };
            }
        }
        "one-line-style" => {
            print_message(
                "debug",
                &format!("creating default one-line-style sources list: \"{output_directory}/sources.list\""),
                &message_config,
            );

            if create_file(
                &format!("{output_directory}/sources.list"),
                "",
                &message_config,
            )
            .is_err()
                == true
            {
                print_message(
                    "error",
                    &format!("failed to create file: \"{output_directory}/sources.list\""),
                    &message_config,
                );
                return Err(());
            };

            for (entry_index, entry) in sources_list.iter().enumerate() {
                let mut full_uris: Vec<String> = Vec::new();

                for (scheme, path) in &entry.uris {
                    full_uris.push(format!("{scheme}{path}"));
                }

                if entry_index != 0 {
                    if append_file(
                        &format!("{output_directory}/sources.list"),
                        "\n",
                        &message_config,
                    )
                    .is_err()
                        == true
                    {
                        return Err(());
                    };
                };

                for (uri_index, uri) in full_uris.iter().enumerate() {
                    if uri_index != 0 {
                        if append_file(
                            &format!("{output_directory}/sources.list"),
                            "\n",
                            &message_config,
                        )
                        .is_err()
                            == true
                        {
                            return Err(());
                        };
                    };

                    for (suite_index, suite) in entry.suites.iter().enumerate() {
                        if suite_index != 0 {
                            if append_file(
                                &format!("{output_directory}/sources.list"),
                                "\n",
                                &message_config,
                            )
                            .is_err()
                                == true
                            {
                                return Err(());
                            };
                        };

                        if append_file(
                            &format!("{output_directory}/sources.list"),
                            &format!(
                                "deb-src {uri} {suite} {}\ndeb {uri} {suite} {}\n",
                                format!("{:?}", &entry.components)
                                    .replace(['[', ']', '"', ','], ""),
                                format!("{:?}", &entry.components)
                                    .replace(['[', ']', '"', ','], "")
                            ),
                            &message_config,
                        )
                        .is_err()
                            == true
                        {
                            return Err(());
                        };
                    }
                }
            }
        }
        _ => {}
    };

    return Ok(());
}
