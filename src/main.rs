use algorithms::*;
pub mod algorithms;
use functions::*;
pub mod functions;
use package::*;
pub mod package;
use releases::*;
pub mod releases;

use byte_unit::{Byte, Unit, UnitType};
use cmd_lib::{run_cmd, run_fun};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::path::Path;
use std::process::ExitCode;
use which::which;

const DEBSTRAP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let mut provided_arguments: Vec<String> = std::env::args().collect::<Vec<String>>();
    provided_arguments.remove(0); // ignore what the program was invoked as.

    let mut early_action_then_exit: Vec<String> = Vec::new();
    let mut chosen_print_color: String = String::new();
    let mut chosen_print_debug: Option<bool> = None;

    for argument in provided_arguments.iter_mut() {
        match argument as &str {
            "-h" | "--help" => {
                early_action_then_exit.push(String::from("print_help_message"));
                *argument = String::new();
            }
            "-V" | "--version" => {
                early_action_then_exit.push(String::from("print_version_message"));
                *argument = String::new();
            }
            "--color" => {
                chosen_print_color = String::from("true");
                *argument = String::new();
            }
            _ if argument.starts_with("--color=") => {
                chosen_print_color = String::from(argument.replacen("--color=", "", 1).trim());
                *argument = String::new();
            }
            "-d" | "--debug" => {
                chosen_print_debug = Some(true);
                *argument = String::new();
            }
            "--no-debug" => {
                chosen_print_debug = Some(false);
                *argument = String::new();
            }
            _ => {}
        }
    }

    let remaining_arguments: Vec<String> = provided_arguments;

    let mut provided_arguments: Vec<String> = Vec::new();

    for argument in remaining_arguments {
        if argument.is_empty() == false {
            provided_arguments.push(argument)
        }
    }

    //////////////////////////////////////////////

    if provided_arguments.len() == 0 {
        if early_action_then_exit.len() == 0 {
            early_action_then_exit.push(String::from("print_help_message"));
        };
    };

    //////////////////////////////////////////////

    if chosen_print_color.is_empty() == true {
        match std::env::var("DEBSTRAP_COLOR") {
            Ok(result) => {
                chosen_print_color = result;
            }
            Err(..) => {
                match std::env::var("NO_COLOR") {
                    Ok(..) => {
                        chosen_print_color = String::from("false");
                    }
                    Err(..) => {
                        chosen_print_color = String::from("auto");
                    }
                };
            }
        };
    }

    let print_color: bool;

    match &chosen_print_color as &str {
        "always" | "true" => {
            print_color = true;
        }
        "never" | "false" => {
            print_color = false;
        }
        "auto" => {
            if termion::is_tty(&std::fs::File::create("/dev/stdout").unwrap()) == true {
                print_color = true;
            } else {
                print_color = false;
            }
        }
        _ => {
            print_color = false;
        }
    };

    //////////////////////////////////////////////

    if chosen_print_debug.is_none() == true {
        match std::env::var("DEBSTRAP_DEBUG") {
            Ok(value) => {
                match &value as &str {
                    "true" => {
                        chosen_print_debug = Some(true);
                    }
                    "false" => {
                        chosen_print_debug = Some(false);
                    }
                    _ => {
                        chosen_print_debug = Some(false);
                    }
                };
            }
            Err(..) => {
                chosen_print_debug = Some(false);
            }
        };
    };

    let print_debug: bool;

    match chosen_print_debug.unwrap() {
        true => {
            print_debug = true;
        }
        false => {
            print_debug = false;
        }
    };

    //////////////////////////////////////////////

    let message_config: MessageConfig = MessageConfig {
        color: print_color,
        debug: print_debug,
    };

    //////////////////////////////////////////////

    match std::env::var("USER") {
        Ok(result) => {
            if result != "root" {
                print_message(
                    "error",
                    "debstrap can only be ran as root or with sudo.",
                    &message_config,
                );
                return ExitCode::from(1);
            };
        }
        Err(..) => {
            print_message(
                "error",
                "failed to read environment variable \"USER\"",
                &message_config,
            );
            return ExitCode::from(1);
        }
    };

    //////////////////////////////////////////////

    let colorful_dpkg: String;
    let term_environment_variable: String;

    match print_color {
        true => {
            colorful_dpkg = String::from("always");
            match std::env::var("TERM") {
                Ok(result) => {
                    term_environment_variable = if result == "linux" {
                        String::from("linux")
                    } else {
                        String::from("xterm-256color")
                    };
                }
                Err(..) => {
                    print_message(
                        "error",
                        "failed to read environment variable \"TERM\"",
                        &message_config,
                    );
                    return ExitCode::from(1);
                }
            };
        }
        false => {
            colorful_dpkg = String::from("never");
            term_environment_variable = String::from("dump");
        }
    };

    //////////////////////////////////////////////

    early_action_then_exit.sort_unstable();
    early_action_then_exit.dedup();

    if early_action_then_exit.len() != 0 {
        if early_action_then_exit.len() != 1 {
            print_message(
                "warning",
                "--help and --version are mutually exclusive.",
                &message_config,
            );
            early_action_then_exit = [String::from("print_help_message")].to_vec();
        };

        match &early_action_then_exit[0] as &str {
            "print_help_message" => {
                println!("Placeholder help message.");
            }
            "print_version_message" => {
                println!("debstrap {DEBSTRAP_VERSION}");
            }
            _ => {}
        };

        return ExitCode::from(0);
    };

    //////////////////////////////////////////////

    print_message(
        "debug",
        &format!(
            "{} \"{DEBSTRAP_VERSION}\"",
            space_and_truncate_string("debstrap version:", 41)
        ),
        &message_config,
    );

    ////////////////////////////////////////////////////////////////////////////////////////////////

    fn parse_list_of_values(prefix: &str, input: &str) -> Vec<String> {
        let value: Vec<String> = input
            .replacen(prefix, "", 1)
            .replace(",", " ")
            .split_whitespace()
            .map(|element| String::from(element.trim()))
            .collect::<Vec<String>>();

        return value;
    }

    let mut chosen_only_action_then_exit: String = String::new();
    let mut chosen_actions_to_skip: Vec<String> = Vec::new();
    let mut chosen_output_location: String = String::new();
    let mut chosen_output_format: String = String::new();
    let mut chosen_suites: Vec<String> = Vec::new();
    let mut chosen_components: Vec<String> = Vec::new();
    let mut chosen_architectures: Vec<String> = Vec::new();
    let mut chosen_variant: String = String::new();
    let mut custom_package_set: Vec<String> = Vec::new();
    let mut chosen_packages_to_include: Vec<String> = Vec::new();
    let mut chosen_packages_to_exclude: Vec<String> = Vec::new();
    let mut chosen_packages_to_prohibit: Vec<String> = Vec::new();
    let mut chosen_mirrors: Vec<String> = Vec::new();
    let mut chosen_resolver: String = String::new();
    let mut consider_recommends: Option<bool> = None;
    let mut chosen_extractor: String = String::new();
    let mut extract_only_essentials: Option<bool> = None;
    let mut chosen_packages_to_consider_essential: Vec<String> = Vec::new();
    let mut chosen_packages_to_consider_non_essential: Vec<String> = Vec::new();
    let mut chosen_merge_usr_directories: String = String::new();
    let mut chosen_sources_list_format: Option<String> = None;
    let mut interactive_bootstrap: Option<bool> = None;
    let mut download_hooks: Vec<String> = Vec::new();
    let mut extract_hooks: Vec<String> = Vec::new();
    let mut essential_hooks: Vec<String> = Vec::new();
    let mut target_hooks: Vec<String> = Vec::new();
    let mut done_hooks: Vec<String> = Vec::new();
    let mut discard_output_on_exit: bool = false;
    let mut assume_user_input: Option<String> = None;

    for argument in provided_arguments {
        match &argument as &str {
            _ if argument.starts_with("-O=") => {
                chosen_only_action_then_exit = String::from(argument.replacen("-O=", "", 1).trim());
            }
            _ if argument.starts_with("--only=") => {
                chosen_only_action_then_exit =
                    String::from(argument.replacen("--only=", "", 1).trim());
            }
            _ if argument.starts_with("-S=") => {
                chosen_actions_to_skip.extend(parse_list_of_values("-S=", &argument));
            }
            _ if argument.starts_with("--skip=") => {
                chosen_actions_to_skip.extend(parse_list_of_values("--skip=", &argument));
            }
            _ if argument.starts_with("-o=") => {
                chosen_output_location = String::from(argument.replacen("-o=", "", 1).trim());
            }
            _ if argument.starts_with("--output=") => {
                chosen_output_location = String::from(argument.replacen("--output=", "", 1).trim());
            }
            _ if argument.starts_with("-f=") => {
                chosen_output_format = String::from(argument.replacen("-f=", "", 1).trim());
            }
            _ if argument.starts_with("--format=") => {
                chosen_output_format = String::from(argument.replacen("--format=", "", 1).trim());
            }
            _ if argument.starts_with("-r=") => {
                chosen_suites.extend(parse_list_of_values("-r=", &argument));
            }
            _ if argument.starts_with("--release=") => {
                chosen_suites.extend(parse_list_of_values("--release=", &argument));
            }
            _ if argument.starts_with("--releases=") => {
                chosen_suites.extend(parse_list_of_values("--releases=", &argument));
            }
            _ if argument.starts_with("-c=") => {
                chosen_components.extend(parse_list_of_values("-c=", &argument));
            }
            _ if argument.starts_with("--comp=") => {
                chosen_components.extend(parse_list_of_values("--comp=", &argument));
            }
            _ if argument.starts_with("--component=") => {
                chosen_components.extend(parse_list_of_values("--component=", &argument));
            }
            _ if argument.starts_with("--components=") => {
                chosen_components.extend(parse_list_of_values("--components=", &argument));
            }
            _ if argument.starts_with("-a=") => {
                chosen_architectures.extend(parse_list_of_values("-a=", &argument));
            }
            _ if argument.starts_with("--arch=") => {
                chosen_architectures.extend(parse_list_of_values("--arch=", &argument));
            }
            _ if argument.starts_with("--architecture=") => {
                chosen_architectures.extend(parse_list_of_values("--architecture=", &argument));
            }
            _ if argument.starts_with("--architectures=") => {
                chosen_architectures.extend(parse_list_of_values("--architectures=", &argument));
            }
            _ if argument.starts_with("-v=") => {
                chosen_variant = String::from(argument.replacen("-v=", "", 1).trim());
            }
            _ if argument.starts_with("--variant=") => {
                chosen_variant = String::from(argument.replacen("--variant=", "", 1).trim());
            }
            _ if argument.starts_with("-p=") => {
                custom_package_set.extend(parse_list_of_values("-p=", &argument));
            }
            _ if argument.starts_with("--package-set=") => {
                custom_package_set.extend(parse_list_of_values("--package-set=", &argument));
            }
            _ if argument.starts_with("-i=") => {
                chosen_packages_to_include.extend(parse_list_of_values("-i=", &argument));
            }
            _ if argument.starts_with("--include=") => {
                chosen_packages_to_include.extend(parse_list_of_values("--include=", &argument));
            }
            _ if argument.starts_with("-e=") => {
                chosen_packages_to_exclude.extend(parse_list_of_values("-e=", &argument));
            }
            _ if argument.starts_with("--exclude=") => {
                chosen_packages_to_exclude.extend(parse_list_of_values("--exclude=", &argument));
            }
            _ if argument.starts_with("-P=") => {
                chosen_packages_to_prohibit.extend(parse_list_of_values("-P=", &argument));
            }
            _ if argument.starts_with("--prohibit=") => {
                chosen_packages_to_prohibit.extend(parse_list_of_values("--prohibit=", &argument));
            }
            _ if argument.starts_with("-m=") => {
                chosen_mirrors.extend(parse_list_of_values("-m=", &argument));
            }
            _ if argument.starts_with("--mirror=") => {
                chosen_mirrors.extend(parse_list_of_values("--mirror=", &argument));
            }
            _ if argument.starts_with("--mirrors=") => {
                chosen_mirrors.extend(parse_list_of_values("--mirrors=", &argument));
            }
            _ if argument.starts_with("-R=") => {
                chosen_resolver = String::from(argument.replacen("-R=", "", 1).trim());
            }
            _ if argument.starts_with("--resolver=") => {
                chosen_resolver = String::from(argument.replacen("--resolver=", "", 1).trim());
            }
            "--include-recommends" => {
                consider_recommends = Some(true);
            }
            "--exclude-recommends" => {
                consider_recommends = Some(false);
            }
            _ if argument.starts_with("-E=") => {
                chosen_extractor = String::from(argument.replacen("-E=", "", 1).trim());
            }
            _ if argument.starts_with("--extractor=") => {
                chosen_extractor = String::from(argument.replacen("--extractor=", "", 1).trim());
            }
            "--extract-essential" => {
                extract_only_essentials = Some(true);
            }
            "--extract-all" => {
                extract_only_essentials = Some(false);
            }
            _ if argument.starts_with("--consider-essential=") => {
                chosen_packages_to_consider_essential
                    .extend(parse_list_of_values("--consider-essential=", &argument));
            }
            _ if argument.starts_with("--consider-non-essential=") => {
                chosen_packages_to_consider_non_essential
                    .extend(parse_list_of_values("--consider-non-essential=", &argument));
            }
            _ if argument.starts_with("-M=") => {
                chosen_merge_usr_directories = String::from(argument.replacen("-M=", "", 1).trim());
            }
            _ if argument.starts_with("--merge-usr=") => {
                chosen_merge_usr_directories =
                    String::from(argument.replacen("--merge-usr=", "", 1).trim());
            }
            "--deb822-style" => {
                chosen_sources_list_format = Some(String::from("deb822-style"));
            }
            "--one-line-style" => {
                chosen_sources_list_format = Some(String::from("one-line-style"));
            }
            "--interactive" => {
                interactive_bootstrap = Some(true);
            }
            "--non-interactive" => {
                interactive_bootstrap = Some(false);
            }
            _ if argument.starts_with("--hook-download=") => {
                download_hooks.push(String::from(
                    argument.replacen("--hook-download=", "", 1).trim(),
                ));
            }
            _ if argument.starts_with("--hook-extract=") => {
                extract_hooks.push(String::from(
                    argument.replacen("--hook-extract=", "", 1).trim(),
                ));
            }
            _ if argument.starts_with("--hook-essential=") => {
                essential_hooks.push(String::from(
                    argument.replacen("--hook-essential=", "", 1).trim(),
                ));
            }
            _ if argument.starts_with("--hook-target=") => {
                target_hooks.push(String::from(
                    argument.replacen("--hook-target=", "", 1).trim(),
                ));
            }
            _ if argument.starts_with("--hook-done=") => {
                done_hooks.push(String::from(
                    argument.replacen("--hook-done=", "", 1).trim(),
                ));
            }
            "-D" | "--discard" | "--discard-output" => {
                discard_output_on_exit = true;
            }
            "-y" | "--yes" | "--assume-yes" => {
                assume_user_input = Some(String::from("assume-yes"));
            }
            "-n" | "--no" | "--assume-no" => {
                assume_user_input = Some(String::from("assume-no"));
            }
            _ => {
                print_message(
                    "error",
                    &format!("unrecognized argument: \"{argument}\""),
                    &message_config,
                );
                return ExitCode::from(1);
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////

    let workspace_directory: String;

    match std::env::var("DEBSTRAP_DIRECTORY") {
        Ok(result) => {
            let provided_path = Path::new(&result);

            if provided_path.is_dir() == true {
                let mut directory_contents: Vec<String> = std::fs::read_dir(&provided_path)
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

                directory_contents.sort_unstable();

                if directory_contents.len() == 0 {
                    workspace_directory = String::from(result);
                } else {
                    print_message(
                        "error",
                        &format!("directory \"{result}\" is not empty."),
                        &message_config,
                    );
                    return ExitCode::from(1);
                };
            } else {
                print_message(
                    "error",
                    &format!("directory \"{result}\" does not exist."),
                    &message_config,
                );
                return ExitCode::from(1);
            };
        }
        Err(..) => {
            workspace_directory = String::from("/tmp/debstrap_")
                + &rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(8)
                    .map(char::from)
                    .collect::<String>();
        }
    };

    print_message(
        "debug",
        &format!(
            "{} \"{workspace_directory}\"",
            space_and_truncate_string("workspace directory:", 41)
        ),
        &message_config,
    );

    //////////////////////////////////////////////

    let mut only_action_then_exit: String = String::new();

    if chosen_only_action_then_exit.is_empty() == false {
        match &chosen_only_action_then_exit as &str {
            "print" => {
                only_action_then_exit = String::from("print_both_sets");
            }
            "print-initial" => {
                only_action_then_exit = String::from("print_initial_set");
            }
            "print-target" => {
                only_action_then_exit = String::from("print_target_set");
            }
            "download" => {
                only_action_then_exit = String::from("download_packages");
            }
            "extract" => {
                only_action_then_exit = String::from("extract_packages");
            }
            _ => {
                print_message(
                    "error",
                    &format!("unrecognized action: \"{chosen_only_action_then_exit}\""),
                    &message_config,
                );
                return ExitCode::from(1);
            }
        };
    };

    let only_action_then_exit: String = only_action_then_exit;

    if chosen_only_action_then_exit == "print_both_sets"
        || chosen_only_action_then_exit == "print_initial_set"
        || chosen_only_action_then_exit == "print_target_set"
    {
        discard_output_on_exit = true;
    };

    if only_action_then_exit.is_empty() == false || discard_output_on_exit == true {
        chosen_actions_to_skip.push(String::from("output_directory_check"));
    };

    //////////////////////////////////////////////

    let mut target_actions_to_skip: Vec<String> = Vec::new();

    if chosen_actions_to_skip.len() != 0 {
        for action in chosen_actions_to_skip {
            match &action as &str {
                "architecture-check" => {
                    if target_actions_to_skip.contains(&String::from("architecture_check")) == false
                    {
                        target_actions_to_skip.push(String::from("architecture_check"));
                    };
                }
                "output-directory-check" => {
                    if target_actions_to_skip.contains(&String::from("output_directory_check"))
                        == false
                    {
                        target_actions_to_skip.push(String::from("output_directory_check"));
                    };
                }
                "packages-removal" => {
                    if target_actions_to_skip.contains(&String::from("packages_removal")) == false {
                        target_actions_to_skip.push(String::from("packages_removal"));
                    };
                }
                "workspace-removal" => {
                    if target_actions_to_skip.contains(&String::from("workspace_removal")) == false
                    {
                        target_actions_to_skip.push(String::from("workspace_removal"));
                    };
                }
                _ => {}
            };
        }
    };

    let target_actions_to_skip: Vec<String> = target_actions_to_skip;

    //////////////////////////////////////////////

    let implied_output_format: String;
    let target_output_directory: String;
    let mut chosen_output_file_name: String = String::new();

    if discard_output_on_exit == true {
        print_message(
            "debug",
            "ignoring any provided output location.",
            &message_config,
        );
        implied_output_format = String::from("directory");
        target_output_directory = format!("{workspace_directory}/output");
    } else {
        if chosen_output_location.is_empty() == true {
            print_message("error", "no output location was provided.", &message_config);
            return ExitCode::from(1);
        };

        while chosen_output_location.contains("//") == true {
            chosen_output_location = chosen_output_location.replace("//", "/");
        }

        if chosen_output_location == "/" {
            print_message(
                "error",
                &format!("invalid output location: \"{chosen_output_location}\""),
                &message_config,
            );
            return ExitCode::from(1);
        };

        if chosen_output_location.ends_with("/") == true {
            chosen_output_location =
                String::from(chosen_output_location.strip_suffix("/").unwrap());
        };

        let provided_path = Path::new(&chosen_output_location);

        if provided_path.is_dir() == true {
            implied_output_format = String::from("directory");
            target_output_directory =
                String::from(provided_path.canonicalize().unwrap().to_string_lossy());
        } else if provided_path.is_file() == true {
            print_message(
                "error",
                &format!("file \"{}\" already exists.", provided_path.display()),
                &message_config,
            );
            return ExitCode::from(1);
        } else {
            let parent_dir: String =
                String::from(provided_path.parent().unwrap().to_string_lossy());
            let parent_dir = Path::new(&parent_dir);
            if parent_dir.is_dir() == true {
                target_output_directory =
                    String::from(parent_dir.canonicalize().unwrap().to_string_lossy());
            } else {
                print_message(
                    "error",
                    &format!("invalid output location: \"{chosen_output_location}\""),
                    &message_config,
                );
                return ExitCode::from(1);
            }
            chosen_output_file_name =
                String::from(provided_path.file_name().unwrap().to_string_lossy());
            match chosen_output_file_name {
                _ if chosen_output_file_name.ends_with(".tar") => {
                    implied_output_format = String::from("tarball");
                }
                _ => {
                    print_message(
                        "error",
                        &format!("invalid file name: \"{chosen_output_location}\""),
                        &message_config,
                    );
                    return ExitCode::from(1);
                }
            };
        };
    };

    print_message(
        "debug",
        &format!(
            "{} \"{target_output_directory}\"",
            space_and_truncate_string("target output directory:", 41)
        ),
        &message_config,
    );

    //////////////////////////////////////////////

    if chosen_output_format.is_empty() == true {
        chosen_output_format = implied_output_format.clone();
    };

    let target_output_format: String;

    match &chosen_output_format as &str {
        "directory" | "dir" => {
            target_output_format = String::from("directory");
        }
        "tarball" | "tar" => {
            target_output_format = String::from("tarball");
        }
        _ => {
            print_message(
                "error",
                &format!("unrecognized format: \"{chosen_output_format}\""),
                &message_config,
            );
            return ExitCode::from(1);
        }
    };

    let target_output_format: String = target_output_format;

    print_message(
        "debug",
        &format!(
            "{} \"{target_output_format}\"",
            space_and_truncate_string("target output format:", 41)
        ),
        &message_config,
    );

    if implied_output_format != "directory" {
        if target_output_format != implied_output_format {
            print_message(
                "error",
                &format!("format \"{target_output_format}\" was chosen but output implied \"{implied_output_format}\""),
                &message_config,
            );
            return ExitCode::from(1);
        };
    };

    if target_actions_to_skip.contains(&String::from("output_directory_check")) == true
        || target_output_format != "directory"
        || discard_output_on_exit == true
    {
        print_message("debug", "skipping output directory check.", &message_config);
    } else {
        let mut directory_contents: Vec<String> = std::fs::read_dir(&target_output_directory)
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

        directory_contents.sort_unstable();

        for item in directory_contents {
            match &item as &str {
                "boot" | "efi" | "lost+found" => {}
                _ => {
                    print_message("error", "output directory is not empty.", &message_config);
                    return ExitCode::from(1);
                }
            };
        }
    };

    //////////////////////////////////////////////

    if chosen_suites.len() == 0 {
        print_message("error", "no suite(s) were provided.", &message_config);
        return ExitCode::from(1);
    };

    let mut target_suites: Vec<String> = Vec::new();

    let mut counter: u8 = 0;

    for suite in chosen_suites {
        counter += 1;
        if counter == 1 {
            if check_primary_suite(&suite) == true {
                target_suites = vec![suite];
            } else {
                print_message(
                    "error",
                    &format!("unrecognized suite: \"{suite}\""),
                    &message_config,
                );
                return ExitCode::from(1);
            };
        } else {
            if target_suites.contains(&suite) == false {
                target_suites.push(suite);
            };
        };
    }

    let target_suites: Vec<String> = target_suites;

    print_message(
        "debug",
        &format!(
            "{} {:?}",
            space_and_truncate_string("target suite(s):", 41),
            &target_suites
        ),
        &message_config,
    );

    //////////////////////////////////////////////

    if chosen_components.len() == 0 {
        chosen_components = vec![String::from("main")];
    };

    let mut target_components: Vec<String> = Vec::new();

    let mut counter: u8 = 0;

    for component in chosen_components {
        counter += 1;
        if counter == 1 {
            if component == "main" {
                target_components = vec![component];
            } else {
                print_message(
                    "error",
                    &format!("invalid first component \"{component}\""),
                    &message_config,
                );
                return ExitCode::from(1);
            };
        } else {
            if target_components.contains(&component) == false {
                target_components.push(component);
            };
        };
    }

    let target_components: Vec<String> = target_components;

    print_message(
        "debug",
        &format!(
            "{} {:?}",
            space_and_truncate_string("target component(s):", 41),
            &target_components
        ),
        &message_config,
    );

    //////////////////////////////////////////////

    // If no architecture(s) is provided then assume the same as the host.

    if chosen_architectures.len() == 0 {
        chosen_architectures.push(String::from(run_fun!(uname "--machine").unwrap()));
    };

    let mut is_keyword_host_present: bool = false;

    for architecture in chosen_architectures.iter_mut() {
        if architecture == &"host" {
            is_keyword_host_present = true;
            *architecture = String::new();
        };
    }

    if target_actions_to_skip.contains(&String::from("architecture_check")) == false
        || is_keyword_host_present == true
    {
        if which("arch-test").is_err() == true {
            print_message(
                "error",
                "arch-test is not available on the host.",
                &message_config,
            );
            return ExitCode::from(1);
        };
    };

    if is_keyword_host_present == true {
        chosen_architectures.sort_unstable();
        chosen_architectures.dedup();

        while chosen_architectures.contains(&String::new()) == true {
            chosen_architectures.remove(0);
        }

        let remaining_architectures: Vec<String> = chosen_architectures;

        chosen_architectures = vec![run_fun!(uname "--machine").unwrap()];

        chosen_architectures.extend(
            String::from(run_fun!("arch-test" "-n").unwrap())
                .split_whitespace()
                .map(|element| String::from(element))
                .collect::<Vec<String>>(),
        );

        chosen_architectures.extend(remaining_architectures);
    };

    let mut target_architectures: Vec<String> = Vec::new();

    for architecture in &chosen_architectures {
        match &architecture.to_lowercase().replace("-", "_") as &str {
            "alpha" => {
                if target_architectures.contains(&String::from("alpha")) == false {
                    target_architectures.push(String::from("alpha"));
                };
            }
            "amd64" | "x86_64" | "x64" => {
                if target_architectures.contains(&String::from("amd64")) == false {
                    target_architectures.push(String::from("amd64"));
                };
            }
            "arm64" | "aarch64" => {
                if target_architectures.contains(&String::from("arm64")) == false {
                    target_architectures.push(String::from("arm64"));
                };
            }
            "armel" => {
                if target_architectures.contains(&String::from("armel")) == false {
                    target_architectures.push(String::from("armel"));
                };
            }
            "armhf" | "aarch32" | "armv7l" => {
                if target_architectures.contains(&String::from("armhf")) == false {
                    target_architectures.push(String::from("armhf"));
                };
            }
            "hppa" | "parisc" => {
                if target_architectures.contains(&String::from("hppa")) == false {
                    target_architectures.push(String::from("hppa"));
                };
            }
            "i386" | "i686" | "ia32" | "x86" | "x86_32" => {
                if target_architectures.contains(&String::from("i386")) == false {
                    target_architectures.push(String::from("i386"));
                };
            }
            "ia64" => {
                if target_architectures.contains(&String::from("ia64")) == false {
                    target_architectures.push(String::from("ia64"));
                };
            }
            "loong64" | "loongarch64" => {
                if target_architectures.contains(&String::from("loong64")) == false {
                    target_architectures.push(String::from("loong64"));
                };
            }
            "m68k" => {
                if target_architectures.contains(&String::from("m68k")) == false {
                    target_architectures.push(String::from("m68k"));
                };
            }
            "mips64el" | "mips64" => {
                if target_architectures.contains(&String::from("mips64el")) == false {
                    target_architectures.push(String::from("mips64el"));
                };
            }
            "mipsel" | "mips" => {
                if target_architectures.contains(&String::from("mipsel")) == false {
                    target_architectures.push(String::from("mipsel"));
                };
            }
            "powerpc" | "ppc" => {
                if target_architectures.contains(&String::from("powerpc")) == false {
                    target_architectures.push(String::from("powerpc"));
                };
            }
            "ppc64" => {
                if target_architectures.contains(&String::from("ppc64")) == false {
                    target_architectures.push(String::from("ppc64"));
                };
            }
            "ppc64el" | "ppc64le" | "powerpc64le" => {
                if target_architectures.contains(&String::from("ppc64el")) == false {
                    target_architectures.push(String::from("ppc64el"));
                };
            }
            "riscv64" => {
                if target_architectures.contains(&String::from("riscv64")) == false {
                    target_architectures.push(String::from("riscv64"));
                };
            }
            "s390x" => {
                if target_architectures.contains(&String::from("s390x")) == false {
                    target_architectures.push(String::from("s390x"));
                };
            }
            "sh4" => {
                if target_architectures.contains(&String::from("sh4")) == false {
                    target_architectures.push(String::from("sh4"));
                };
            }
            "sparc64" => {
                if target_architectures.contains(&String::from("sparc64")) == false {
                    target_architectures.push(String::from("sparc64"));
                };
            }
            "x32" => {
                if target_architectures.contains(&String::from("x32")) == false {
                    target_architectures.push(String::from("x32"));
                };
            }
            _ => {
                print_message(
                    "error",
                    &format!("unrecognized architecture: \"{architecture}\""),
                    &message_config,
                );
                return ExitCode::from(1);
            }
        };
    }

    let target_architectures: Vec<String> = target_architectures;

    print_message(
        "debug",
        &format!(
            "{} {:?}",
            space_and_truncate_string("target architecture(s):", 41),
            target_architectures
        ),
        &message_config,
    );

    if target_actions_to_skip.contains(&String::from("architecture_check")) == true {
        print_message("debug", "skipping architecture check.", &message_config);
    } else {
        for architecture in &target_architectures {
            if run_fun!(arch-test "$architecture").is_err() == true {
                if only_action_then_exit.is_empty() == true {
                    print_message(
                        "error",
                        &format!(
                            "architecture \"{architecture}\" is not executable by the host kernel."
                        ),
                        &message_config,
                    );
                    return ExitCode::from(1);
                };
            };
        }
    };

    //////////////////////////////////////////////

    if chosen_variant.is_empty() {
        chosen_variant = String::from("important");
    };

    let target_variant: String;

    match &chosen_variant as &str {
        "essential" => {
            target_variant = String::from("essential");
        }
        "required" => {
            target_variant = String::from("required");
        }
        "buildd" => {
            target_variant = String::from("buildd");
        }
        "important" => {
            target_variant = String::from("important");
        }
        "standard" => {
            target_variant = String::from("standard");
        }
        "custom" => {
            target_variant = String::from("custom");
        }
        _ => {
            print_message(
                "error",
                &format!("unrecognized variant: \"{chosen_variant}\""),
                &message_config,
            );
            return ExitCode::from(1);
        }
    };

    print_message(
        "debug",
        &format!(
            "{} \"{target_variant}\"",
            space_and_truncate_string("target variant:", 41)
        ),
        &message_config,
    );

    print_message(
        "debug",
        &format!(
            "{} \"{}\"",
            space_and_truncate_string("# of packages in custom set:", 41),
            &custom_package_set.len()
        ),
        &message_config,
    );

    if target_variant == "custom" {
        if custom_package_set.len() == 0 {
            print_message("error", "no package set was provided.", &message_config);
            return ExitCode::from(1);
        };
    } else if custom_package_set.len() != 0 {
        print_message(
            "error",
            "only the variant \"custom\" can use a custom package set.",
            &message_config,
        );
        return ExitCode::from(1);
    };

    //////////////////////////////////////////////

    let mut target_output_file_name: String = String::new();

    match &target_output_format as &str {
        "tarball" => {
            if chosen_output_file_name.is_empty() == true {
                target_output_file_name = format!(
                    "{}_{}",
                    default_output_file_name(
                        &target_suites[0],
                        &target_architectures[0],
                        &target_variant,
                    ),
                    run_fun!(date "+%Yy-%mm-%dd").unwrap(),
                );
                match &only_action_then_exit as &str {
                    "download_packages" => {
                        target_output_file_name = format!("Packages_{target_output_file_name}");
                    }
                    "extract_packages" => {
                        target_output_file_name = format!("Extracted_{target_output_file_name}");
                    }
                    _ => {}
                };
            } else {
                target_output_file_name = chosen_output_file_name;
            };

            print_message(
                "debug",
                &format!(
                    "{} \"{target_output_file_name}\"",
                    space_and_truncate_string("target output file name:", 41)
                ),
                &message_config,
            );
        }
        _ => {}
    };

    let target_output_file_name: String = target_output_file_name;

    //////////////////////////////////////////////

    if chosen_mirrors.len() == 0 {
        for uri in default_mirrors(&target_suites[0], &target_architectures[0]) {
            chosen_mirrors.push(uri);
        }
    };

    let mut target_mirrors: Vec<String> = Vec::new();

    for uri in chosen_mirrors {
        let mut mirror: String;

        match uri {
            _ if uri.starts_with("http://") || uri.starts_with("https://") => {
                mirror = uri;
            }
            _ => {
                print_message("error", &format!("invalid URI: \"{uri}\""), &message_config);
                return ExitCode::from(1);
            }
        };

        while mirror.contains("//") == true {
            mirror = mirror.replace("//", "/");
        }

        mirror = mirror.replacen(":/", "://", 1);

        if mirror.ends_with("/") == true {
            mirror = String::from(mirror.strip_suffix("/").unwrap());
        };

        target_mirrors.push(mirror);
    }

    let target_mirrors: Vec<String> = target_mirrors;

    print_message(
        "debug",
        &format!(
            "{} {:?}",
            space_and_truncate_string("target mirror(s):", 41),
            &target_mirrors
        ),
        &message_config,
    );

    //////////////////////////////////////////////

    if chosen_resolver.is_empty() == true {
        chosen_resolver = String::from("internal");
    };

    let target_resolver: String;

    match &chosen_resolver as &str {
        //"apt" => {
        //    target_resolver = String::from("apt");
        //}
        "internal" => {
            target_resolver = String::from("internal");
        }
        "none" => {
            target_resolver = String::from("none");
        }
        _ => {
            print_message(
                "error",
                &format!("unrecognized resolver: \"{chosen_resolver}\""),
                &message_config,
            );
            return ExitCode::from(1);
        }
    };

    let target_resolver: String = target_resolver;

    print_message(
        "debug",
        &format!(
            "{} \"{target_resolver}\"",
            space_and_truncate_string("target resolver:", 41)
        ),
        &message_config,
    );

    if consider_recommends.is_none() == true {
        consider_recommends = Some(false);
    };

    let consider_recommends: bool = consider_recommends.unwrap();

    print_message(
        "debug",
        &format!(
            "{} \"{consider_recommends}\"",
            space_and_truncate_string("consider recommends:", 41)
        ),
        &message_config,
    );

    //////////////////////////////////////////////

    if chosen_extractor.is_empty() == true {
        chosen_extractor = String::from("ar");
    };

    let target_extractor: String;

    match &chosen_extractor as &str {
        "ar" => {
            target_extractor = String::from("ar");
        }
        "dpkg-deb" => {
            target_extractor = String::from("dpkg-deb");
        }
        _ => {
            print_message(
                "error",
                &format!("unrecognized extractor: \"{chosen_extractor}\""),
                &message_config,
            );
            return ExitCode::from(1);
        }
    };

    let target_extractor: String = target_extractor;

    print_message(
        "debug",
        &format!(
            "{} \"{target_extractor}\"",
            space_and_truncate_string("target extractor:", 41)
        ),
        &message_config,
    );

    match &target_extractor as &str {
        "ar" => {
            if which("ar").is_err() == true {
                print_message("error", "ar is not available on the host.", &message_config);
                return ExitCode::from(1);
            };
        }
        "dpkg-deb" => {
            if which("dpkg-deb").is_err() == true {
                print_message(
                    "error",
                    "dpkg-deb is not available on the host.",
                    &message_config,
                );
                return ExitCode::from(1);
            };
        }
        _ => {}
    };

    if extract_only_essentials.is_none() == true {
        if target_variant == "essential" && chosen_packages_to_include.len() == 0 {
            extract_only_essentials = Some(false);
        } else {
            extract_only_essentials = Some(true);
        };
    };

    let extract_only_essentials: bool = extract_only_essentials.unwrap();

    print_message(
        "debug",
        &format!(
            "{} \"{extract_only_essentials}\"",
            space_and_truncate_string("extract only essentials:", 41)
        ),
        &message_config,
    );

    chosen_packages_to_consider_essential.sort_unstable();
    chosen_packages_to_consider_essential.dedup();

    let mut packages_to_consider_essential: Vec<String> = chosen_packages_to_consider_essential;

    print_message(
        "debug",
        &format!(
            "{} \"{}\"",
            space_and_truncate_string("# of packages to consider essential:", 41),
            &packages_to_consider_essential.len()
        ),
        &message_config,
    );

    chosen_packages_to_consider_non_essential.sort_unstable();
    chosen_packages_to_consider_non_essential.dedup();

    let packages_to_consider_non_essential: Vec<String> = chosen_packages_to_consider_non_essential;

    print_message(
        "debug",
        &format!(
            "{} \"{}\"",
            space_and_truncate_string("# of packages to consider non-essential:", 41),
            &packages_to_consider_non_essential.len()
        ),
        &message_config,
    );

    //////////////////////////////////////////////

    if chosen_merge_usr_directories.is_empty() == true {
        chosen_merge_usr_directories = String::from("auto");
    };

    let mut merge_usr_directories: Option<String>;

    match &chosen_merge_usr_directories as &str {
        "yes" | "true" => {
            merge_usr_directories = Some(String::new());
        }
        "no" | "false" => {
            merge_usr_directories = None;
        }
        "auto" => {
            match default_merge_usr_directories(&target_suites[0], &target_variant) {
                true => {
                    merge_usr_directories = Some(String::new());
                }
                false => {
                    merge_usr_directories = None;
                }
            };
        }
        _ => {
            print_message(
                "error",
                &format!("invalid option: \"{chosen_merge_usr_directories}\""),
                &message_config,
            );
            return ExitCode::from(1);
        }
    };

    if merge_usr_directories.is_some() == true {
        match &method_of_usr_merge(&target_suites[0]) as &str {
            "manual_usr_merge" => {
                merge_usr_directories = Some(String::from("manual_usr_merge"));
            }
            "usrmerge_package" => {
                merge_usr_directories = Some(String::from("usrmerge_package"));

                chosen_packages_to_include.push(String::from("usrmerge"));
            }
            _ => {}
        };
    };

    let merge_usr_directories: Option<String> = merge_usr_directories;

    print_message(
        "debug",
        &format!(
            "{} \"{:?}\"",
            space_and_truncate_string("merge /usr directories:", 41),
            merge_usr_directories
        ),
        &message_config,
    );

    if merge_usr_directories.is_none() == true && is_split_usr_supported(&target_suites[0]) == false
    {
        print_message(
            "warning",
            "upgrading non-merged-/usr environments post-bookworm is unsupported.",
            &message_config,
        );
    };

    //////////////////////////////////////////////

    if chosen_sources_list_format.is_none() == true {
        chosen_sources_list_format = Some(default_sources_list_format(&target_suites[0]));
    };

    let sources_list_format: String = chosen_sources_list_format.unwrap();

    print_message(
        "debug",
        &format!(
            "{} \"{sources_list_format}\"",
            space_and_truncate_string("sources list format:", 41)
        ),
        &message_config,
    );

    //////////////////////////////////////////////

    if interactive_bootstrap.is_none() == true {
        interactive_bootstrap = Some(false);
    };

    let interactive_bootstrap: bool = interactive_bootstrap.unwrap();

    print_message(
        "debug",
        &format!(
            "{} \"{interactive_bootstrap}\"",
            space_and_truncate_string("interactive bootstrap:", 41)
        ),
        &message_config,
    );

    let debian_frontend: String;
    let debconf_noninteractive_seen: String;

    match interactive_bootstrap {
        true => {
            debian_frontend = String::from("dialog");
            debconf_noninteractive_seen = String::from("false");
        }
        false => {
            debian_frontend = String::from("noninteractive");
            debconf_noninteractive_seen = String::from("true");
        }
    };

    let debian_frontend: String = debian_frontend;
    let debconf_noninteractive_seen: String = debconf_noninteractive_seen;

    if interactive_bootstrap == true && assume_user_input == Some(String::from("assume-yes")) {
        print_message(
            "warning",
            "using --interactive with --assume-yes may cause hanging in scripts.",
            &message_config,
        );
    };

    //////////////////////////////////////////////

    match case_specific_packages(&target_suites[0], &target_variant) {
        Some(to_include) => {
            for package in to_include {
                chosen_packages_to_include.push(String::from(package));
            }
        }
        None => {}
    };

    //////////////////////////////////////////////

    chosen_packages_to_include.sort_unstable();
    chosen_packages_to_include.dedup();

    let packages_to_include: Vec<String> = chosen_packages_to_include;

    print_message(
        "debug",
        &format!(
            "{} \"{}\"",
            space_and_truncate_string("# of packages to include:", 41),
            &packages_to_include.len()
        ),
        &message_config,
    );

    chosen_packages_to_exclude.extend(chosen_packages_to_prohibit.clone());
    chosen_packages_to_exclude.sort_unstable();
    chosen_packages_to_exclude.dedup();

    let packages_to_exclude: Vec<String> = chosen_packages_to_exclude;

    print_message(
        "debug",
        &format!(
            "{} \"{}\"",
            space_and_truncate_string("# of packages to exclude:", 41),
            &packages_to_exclude.len()
        ),
        &message_config,
    );

    chosen_packages_to_prohibit.sort_unstable();
    chosen_packages_to_prohibit.dedup();

    let packages_to_prohibit: Vec<String> = chosen_packages_to_prohibit;

    print_message(
        "debug",
        &format!(
            "{} \"{}\"",
            space_and_truncate_string("# of packages to prohibit:", 41),
            &packages_to_prohibit.len()
        ),
        &message_config,
    );

    //////////////////////////////////////////////

    print_message(
        "debug",
        &format!(
            "{} \"{}\"",
            space_and_truncate_string("# of download hooks:", 41),
            &download_hooks.len()
        ),
        &message_config,
    );
    print_message(
        "debug",
        &format!(
            "{} \"{}\"",
            space_and_truncate_string("# of extract hooks:", 41),
            &extract_hooks.len()
        ),
        &message_config,
    );
    print_message(
        "debug",
        &format!(
            "{} \"{}\"",
            space_and_truncate_string("# of essential hooks:", 41),
            &essential_hooks.len()
        ),
        &message_config,
    );
    print_message(
        "debug",
        &format!(
            "{} \"{}\"",
            space_and_truncate_string("# of target hooks:", 41),
            &target_hooks.len()
        ),
        &message_config,
    );
    print_message(
        "debug",
        &format!(
            "{} \"{}\"",
            space_and_truncate_string("# of done hooks:", 41),
            &done_hooks.len()
        ),
        &message_config,
    );

    ////////////////////////////////////////////////////////////////////////////////////////////////

    if Path::new(&workspace_directory).exists() == false {
        print_message(
            "debug",
            &format!("creating directory \"{workspace_directory}\""),
            &message_config,
        );

        if std::fs::create_dir(&workspace_directory).is_err() == true {
            print_message(
                "error",
                &format!("failed to create directory \"{workspace_directory}\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                None,
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    };

    if discard_output_on_exit == true {
        print_message(
            "debug",
            &format!("creating directory \"{target_output_directory}\""),
            &message_config,
        );

        if std::fs::create_dir(&target_output_directory).is_err() == true {
            print_message(
                "error",
                &format!("failed to create directory \"{target_output_directory}\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                None,
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    };

    //////////////////////////////////////////////

    println!("Fetching package list ...");

    let mut did_package_list_download: bool = false;

    let package_list_uri: String = format!(
        "{}/dists/{}/{}/binary-{}",
        &target_mirrors[0], &target_suites[0], &target_components[0], &target_architectures[0],
    );

    for file_name in ["Packages.xz", "Packages.gz", "Packages.bz2", "Packages"] {
        match tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(download_file(
                &format!("{package_list_uri}/{file_name}"),
                &workspace_directory,
                &message_config,
            )) {
            Ok(..) => {
                did_package_list_download = true;

                if decompress_file(
                    &format!("{workspace_directory}/{file_name}"),
                    &message_config,
                )
                .is_err()
                    == true
                {
                    clean_up_on_exit(
                        &workspace_directory,
                        None,
                        &target_actions_to_skip,
                        &message_config,
                    )
                    .unwrap_or(());

                    return ExitCode::from(1);
                };

                break;
            }
            Err(message) => {
                print_message("debug", &format!("{message}, skipping."), &message_config);
            }
        };
    }

    if did_package_list_download == false {
        print_message("error", "failed to download package list.", &message_config);

        clean_up_on_exit(
            &workspace_directory,
            None,
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    //////////////////////////////////////////////

    let mut package_database: HashMap<String, Package> = HashMap::new();

    match std::fs::read_to_string(format!("{workspace_directory}/Packages")) {
        Ok(result) => {
            for entry in result
                .trim()
                .split("\n\n")
                .map(|element| String::from(element))
                .collect::<Vec<String>>()
            {
                let package: Package = Package::new(
                    &entry,
                    &target_mirrors[0],
                    &target_suites[0],
                    &target_components[0],
                    &target_architectures[0],
                );

                package_database.insert(package.name.clone(), package);
            }
        }
        Err(..) => {}
    };

    let package_database: HashMap<String, Package> = package_database;

    //////////////////////////////////////////////

    let mut initial_package_set: Vec<String> = Vec::new();

    match &target_variant as &str {
        "essential" => {
            for package in package_database.values() {
                if package.is_essential == true {
                    initial_package_set.push(package.name.clone());
                };
            }

            initial_package_set.push(String::from("mawk"));
        }
        "required" => {
            for package in package_database.values() {
                if package.is_essential == true || package.priority == "required" {
                    initial_package_set.push(package.name.clone());
                };
            }

            initial_package_set.push(String::from("apt"));
        }
        "buildd" => {
            for package in package_database.values() {
                if package.is_essential == true
                    || package.priority == "required"
                    || package.is_build_essential == true
                {
                    initial_package_set.push(package.name.clone());
                };
            }

            initial_package_set.extend([String::from("apt"), String::from("build-essential")]);
        }
        "important" => {
            for package in package_database.values() {
                if package.is_essential == true
                    || package.priority == "required"
                    || package.priority == "important"
                {
                    initial_package_set.push(package.name.clone());
                };
            }
        }
        "standard" => {
            for package in package_database.values() {
                if package.is_essential == true
                    || package.priority == "required"
                    || package.priority == "important"
                    || package.priority == "standard"
                {
                    initial_package_set.push(package.name.clone());
                };
            }
        }
        "custom" => {
            initial_package_set = custom_package_set;
        }
        _ => {}
    };

    initial_package_set.sort_unstable();
    initial_package_set.dedup();

    //////////////////////////////////////////////

    if packages_to_include.len() != 0 {
        for included in &packages_to_include {
            if initial_package_set.contains(included) == true {
                print_message(
                    "debug",
                    &format!("package present: \"{included}\""),
                    &message_config,
                );
            } else {
                print_message(
                    "debug",
                    &format!("package added:   \"{included}\""),
                    &message_config,
                );
                initial_package_set.push(String::from(included));
            };
        }

        initial_package_set.sort_unstable();
    };

    if packages_to_exclude.len() != 0 {
        for excluded in &packages_to_exclude {
            if initial_package_set.contains(excluded) == true {
                for package in initial_package_set.iter_mut() {
                    if &*package == excluded {
                        *package = String::new();

                        print_message(
                            "debug",
                            &format!("package removed: \"{excluded}\""),
                            &message_config,
                        );

                        break;
                    };
                }
            } else {
                print_message(
                    "debug",
                    &format!("package absent:  \"{excluded}\""),
                    &message_config,
                );
            };
        }

        initial_package_set.sort_unstable();

        while initial_package_set.contains(&String::new()) == true {
            initial_package_set.remove(0);
        }
    };

    let initial_package_set: Vec<String> = initial_package_set;

    //////////////////////////////////////////////

    if only_action_then_exit == "print_initial_set" || only_action_then_exit == "print_both_sets" {
        println!("");

        for package in &initial_package_set {
            println!("{package}");
        }

        println!(
            "\n{} packages before dependencies.\n",
            initial_package_set.len()
        );

        if only_action_then_exit == "print_initial_set" {
            if clean_up_on_exit(
                &workspace_directory,
                None,
                &target_actions_to_skip,
                &message_config,
            )
            .is_err()
                == true
            {
                return ExitCode::from(1);
            };

            return ExitCode::from(0);
        };
    };

    ////////////////////////////////////////////////////////////////////////////////////////////////

    let mut target_package_set: Vec<Package> = Vec::new();

    match &target_resolver as &str {
        "internal" => {
            println!("Calculating dependencies ...");

            match resolve_dependencies(
                &package_database,
                &initial_package_set,
                &consider_recommends,
                &packages_to_prohibit,
                &message_config,
            ) {
                Ok(result) => {
                    target_package_set = result;
                }
                Err(..) => {
                    clean_up_on_exit(
                        &workspace_directory,
                        None,
                        &target_actions_to_skip,
                        &message_config,
                    )
                    .unwrap_or(());
                    return ExitCode::from(1);
                }
            };
        }
        "none" => {
            for initial in &initial_package_set {
                match package_database.get(initial) {
                    Some(result) => {
                        target_package_set.push(result.clone());
                    }
                    None => {
                        print_message(
                            "error",
                            &format!("failed to find package: \"{initial}\""),
                            &message_config,
                        );

                        clean_up_on_exit(
                            &workspace_directory,
                            None,
                            &target_actions_to_skip,
                            &message_config,
                        )
                        .unwrap_or(());

                        return ExitCode::from(1);
                    }
                };
            }
        }
        _ => {}
    };

    let target_package_set: Vec<Package> = target_package_set;

    //////////////////////////////////////////////

    if only_action_then_exit == "print_target_set" || only_action_then_exit == "print_both_sets" {
        println!("");

        for package in &target_package_set {
            println!("{}", package.name);
        }

        println!("\n{} packages total.\n", target_package_set.len());

        if clean_up_on_exit(
            &workspace_directory,
            None,
            &target_actions_to_skip,
            &message_config,
        )
        .is_err()
            == true
        {
            return ExitCode::from(1);
        };

        return ExitCode::from(0);
    };

    //////////////////////////////////////////////

    let mut dependency_package_set: Vec<String> = Vec::new();

    if target_package_set.len() != initial_package_set.len() {
        for package in &target_package_set {
            if initial_package_set.contains(&package.name) == false {
                dependency_package_set.push(package.name.clone());
            };
        }
    };

    dependency_package_set.sort_unstable();
    dependency_package_set.dedup();

    let dependency_package_set: Vec<String> = dependency_package_set;

    if initial_package_set.len() + dependency_package_set.len() != target_package_set.len() {
        print_message(
            "error",
            "failed to calculate dependencies.",
            &message_config,
        );

        if clean_up_on_exit(
            &workspace_directory,
            None,
            &target_actions_to_skip,
            &message_config,
        )
        .is_err()
            == true
        {
            return ExitCode::from(1);
        };

        return ExitCode::from(0);
    };

    //////////////////////////////////////////////

    let mut counter_spacing: u16 = 0;

    if target_package_set.len() < 10 {
        counter_spacing = 2;
    } else if target_package_set.len() < 100 {
        counter_spacing = 3;
    } else if target_package_set.len() < 1000 {
        counter_spacing = 4;
    } else if target_package_set.len() < 10000 {
        counter_spacing = 5;
    } else if target_package_set.len() < 100000 {
        counter_spacing = 6;
    };

    let counter_spacing: u16 = counter_spacing;

    let mut blank_counter_spacing: String = String::from(" ");

    for _value in 1..counter_spacing {
        blank_counter_spacing.push(' ');
    }

    let name_length: u16 = 40;
    let version_length: u16 = 30;
    let architecture_length: u16 = 12;
    let description_length: u16 = 72;

    let message_header: String = format!(
        "{blank_counter_spacing} {} {} {} {} Size",
        space_and_truncate_string("Name", name_length),
        space_and_truncate_string("Version", version_length),
        space_and_truncate_string("Architecture", architecture_length),
        space_and_truncate_string("Description", description_length),
    );

    if message_config.color == true {
        println!("\n\x1b[01m{message_header}\x1b[00m");
    } else {
        println!("\n{message_header}");
    };

    let mut counter: u16 = 0;

    for package in &target_package_set {
        counter += 1;

        let package_counter: String =
            space_and_truncate_string(&format!("{counter}."), counter_spacing);
        let package_name: String = space_and_truncate_string(&package.name, name_length);
        let package_version: String = space_and_truncate_string(&package.version, version_length);
        let package_architecture: String =
            space_and_truncate_string(&package.architecture, architecture_length);
        let package_description: String =
            space_and_truncate_string(&package.description, description_length);
        let package_size: String = format!(
            "{:.3}",
            &Byte::from_f64_with_unit(*&package.file_size as f64, Unit::KiB)
                .unwrap()
                .get_appropriate_unit(UnitType::Binary),
        );

        let line_to_print: String = format!(
            "{} {} {} {} {} {}",
            package_counter,
            package_name,
            package_version,
            package_architecture,
            package_description,
            package_size,
        );

        println!("{line_to_print}");
    }

    //////////////////////////////////////////////

    println!(
        "\n{} initially, {} dependencies, {} packages total.",
        initial_package_set.len(),
        dependency_package_set.len(),
        target_package_set.len(),
    );

    //////////////////////////////////////////////

    print!("Do you want to proceed? [Y/n] ");

    stdout().flush().unwrap();

    let mut yes_or_no: String = String::new();

    if assume_user_input.is_none() == true {
        if std::io::stdin().read_line(&mut yes_or_no).is_err() == true {
            print_message("error", "invalid response.", &message_config);

            clean_up_on_exit(
                &workspace_directory,
                None,
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    } else {
        match &assume_user_input.unwrap() as &str {
            "assume-yes" => {
                yes_or_no = String::from("Y");
                println!("Y");
            }
            "assume-no" => {
                yes_or_no = String::from("N");
                println!("N");
            }
            _ => {}
        };
    };

    match &yes_or_no.to_ascii_lowercase().trim() as &str {
        "y" | "yes" => {}
        "n" | "no" => {
            if clean_up_on_exit(
                &workspace_directory,
                None,
                &target_actions_to_skip,
                &message_config,
            )
            .is_err()
                == true
            {
                return ExitCode::from(1);
            };

            return ExitCode::from(0);
        }
        _ => {
            if yes_or_no.is_empty() == false && yes_or_no != "\n" {
                print_message("error", "invalid response.", &message_config);

                clean_up_on_exit(
                    &workspace_directory,
                    None,
                    &target_actions_to_skip,
                    &message_config,
                )
                .unwrap_or(());

                return ExitCode::from(1);
            };
        }
    };

    ////////////////////////////////////////////////////////////////////////////////////////////////

    println!("Downloading packages:");

    let all_packages_directory: String = format!("{workspace_directory}/packages");

    print_message(
        "debug",
        &format!("creating directory \"{all_packages_directory}\""),
        &message_config,
    );

    if std::fs::create_dir(&all_packages_directory).is_err() == true {
        print_message(
            "error",
            &format!("failed to create directory \"{all_packages_directory}\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            None,
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    let downloaded_packages_directory: String = format!("{all_packages_directory}/downloaded");

    print_message(
        "debug",
        &format!("creating directory \"{downloaded_packages_directory}\""),
        &message_config,
    );

    if std::fs::create_dir(&downloaded_packages_directory).is_err() == true {
        print_message(
            "error",
            &format!("failed to create directory \"{downloaded_packages_directory}\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            None,
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    let mut counter_spacing: u16 = 0;

    if target_package_set.len() < 10 {
        counter_spacing = 6;
    } else if target_package_set.len() < 100 {
        counter_spacing = 8;
    } else if target_package_set.len() < 1000 {
        counter_spacing = 10;
    } else if target_package_set.len() < 10000 {
        counter_spacing = 12;
    } else if target_package_set.len() < 100000 {
        counter_spacing = 14;
    };

    let counter_spacing: u16 = counter_spacing;

    let mut counter: u64 = 0;

    for package in &target_package_set {
        counter += 1;

        let download_counter: String = space_and_truncate_string(
            &format!("({counter}/{}):", target_package_set.len(),),
            counter_spacing,
        );

        println!("{download_counter} {}", package.name);

        match tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(download_file(
                &format!("{}/{}", package.uri, package.file_name),
                &downloaded_packages_directory,
                &message_config,
            )) {
            Ok(..) => {}
            Err(message) => {
                print_message("error", &message, &message_config);

                clean_up_on_exit(
                    &workspace_directory,
                    None,
                    &target_actions_to_skip,
                    &message_config,
                )
                .unwrap_or(());

                return ExitCode::from(1);
            }
        };
    }

    //////////////////////////////////////////////

    if download_hooks.len() != 0 {
        run_hooks(
            "download",
            &download_hooks,
            &workspace_directory,
            Some(&all_packages_directory),
            None,
            &message_config,
        );
    };

    //////////////////////////////////////////////

    if only_action_then_exit == "download_packages" {
        match &target_output_format as &str {
            "directory" => {
                print_message(
                    "debug",
                    &format!("moving packages from \"{downloaded_packages_directory}\" to \"{target_output_directory}\""),
                    &message_config,
                );

                let mut downloaded_package_file_names: Vec<String> =
                    std::fs::read_dir(&downloaded_packages_directory)
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

                for package in downloaded_package_file_names {
                    if move_file(
                        format!("{downloaded_packages_directory}/{package}"),
                        format!("{target_output_directory}/{package}"),
                    )
                    .is_err()
                        == true
                    {
                        print_message(
                            "error",
                            &format!(
                                "failed to move file \"{downloaded_packages_directory}/{package}\""
                            ),
                            &message_config,
                        );

                        clean_up_on_exit(
                            &workspace_directory,
                            None,
                            &target_actions_to_skip,
                            &message_config,
                        )
                        .unwrap_or(());

                        return ExitCode::from(1);
                    };
                }
            }
            "tarball" => {
                println!("Packaging (this might take a while) ...");

                print_message(
                    "debug",
                    &format!("creating tarball \"{target_output_directory}/{target_output_file_name}.tar\""),
                    &message_config,
                );

                if run_cmd!(
                    bash -c "
                        cd '$downloaded_packages_directory'
                        tar --create --file='$target_output_directory/$target_output_file_name.tar' *.deb
                    " 2> /dev/stdout
                )
                .is_err()
                    == true
                {
                    print_message(
                        "error",
                        &format!("failed to create tarball \"{target_output_directory}/{target_output_file_name}.tar\""),
                        &message_config,
                    );

                    clean_up_on_exit(
                        &workspace_directory,
                        None,
                        &target_actions_to_skip,
                        &message_config,
                    ).unwrap_or(());

                    return ExitCode::from(1);
                };
            }
            _ => {}
        };

        if clean_up_on_exit(
            &workspace_directory,
            None,
            &target_actions_to_skip,
            &message_config,
        )
        .is_err()
            == true
        {
            return ExitCode::from(1);
        };

        return ExitCode::from(0);
    };

    ////////////////////////////////////////////////////////////////////////////////////////////////

    let target_bootstrap_directory: String;

    match &target_output_format as &str {
        "directory" => {
            target_bootstrap_directory = target_output_directory.clone();
        }
        _ => {
            target_bootstrap_directory = format!("{workspace_directory}/target");
        }
    };

    if Path::new(&target_bootstrap_directory).exists() == false {
        print_message(
            "debug",
            &format!("creating directory \"{target_bootstrap_directory}\""),
            &message_config,
        );

        if std::fs::create_dir(&target_bootstrap_directory).is_err() == true {
            print_message(
                "error",
                &format!("failed to create directory \"{target_bootstrap_directory}\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    };

    //////////////////////////////////////////////

    let initial_packages_directory: String = format!("{all_packages_directory}/initial");

    let remaining_packages_directory: String = format!("{all_packages_directory}/remaining");

    print_message(
        "debug",
        &format!("creating directory \"{initial_packages_directory}\""),
        &message_config,
    );

    if std::fs::create_dir(&initial_packages_directory).is_err() == true {
        print_message(
            "error",
            &format!("failed to create directory \"{initial_packages_directory}\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    let mut downloaded_package_file_names: Vec<String> =
        std::fs::read_dir(&downloaded_packages_directory)
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

    if extract_only_essentials == true {
        println!("Preparing to extract essentials ...");

        let mut initial_essential_subset: Vec<String> = Vec::new();

        let mut is_awk_present: bool = false;

        for package in &target_package_set {
            if package.is_essential == true {
                initial_essential_subset.push(package.name.clone());
            };

            if is_awk_present == false {
                match &package.name as &str {
                    "mawk" => {
                        packages_to_consider_essential.push(String::from("mawk"));
                        is_awk_present = true;
                    }
                    "original-awk" => {
                        packages_to_consider_essential.push(String::from("original-awk"));
                        is_awk_present = true;
                    }
                    "gawk" => {
                        packages_to_consider_essential.push(String::from("gawk"));
                        is_awk_present = true;
                    }
                    _ => {}
                };
            };

            if package.name == "usrmerge" {
                packages_to_consider_essential.push(String::from("usrmerge"));
            };
        }

        if is_awk_present == false {
            print_message(
                "error",
                "no packages that provide \"awk\" are present.",
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };

        packages_to_consider_essential.sort_unstable();
        packages_to_consider_essential.dedup();

        if packages_to_consider_essential.len() != 0 {
            for included in &packages_to_consider_essential {
                if initial_essential_subset.contains(included) == true {
                    print_message(
                        "debug",
                        &format!("essential package present: \"{included}\""),
                        &message_config,
                    );
                } else {
                    print_message(
                        "debug",
                        &format!("essential package added:   \"{included}\""),
                        &message_config,
                    );
                    initial_essential_subset.push(String::from(included));
                };
            }

            initial_essential_subset.sort_unstable();
        };

        if packages_to_consider_non_essential.len() != 0 {
            for excluded in &packages_to_consider_non_essential {
                if initial_essential_subset.contains(excluded) == true {
                    for package in initial_essential_subset.iter_mut() {
                        if &*package == excluded {
                            *package = String::new();

                            print_message(
                                "debug",
                                &format!("package removed: \"{excluded}\""),
                                &message_config,
                            );

                            break;
                        };
                    }
                } else {
                    print_message(
                        "debug",
                        &format!("package absent:  \"{excluded}\""),
                        &message_config,
                    );
                };
            }

            initial_essential_subset.sort_unstable();

            while initial_essential_subset.contains(&String::new()) == true {
                initial_essential_subset.remove(0);
            }
        };

        let initial_essential_subset: Vec<String> = initial_essential_subset;

        //////////////////////////////////////////

        print_message("debug", "calculating essential subset.", &message_config);

        let target_essential_subset: Vec<Package>;

        match resolve_dependencies(
            &package_database,
            &initial_essential_subset,
            &consider_recommends,
            &packages_to_prohibit,
            &message_config,
        ) {
            Ok(result) => {
                target_essential_subset = result;
            }
            Err(..) => {
                clean_up_on_exit(
                    &workspace_directory,
                    None,
                    &target_actions_to_skip,
                    &message_config,
                )
                .unwrap_or(());
                return ExitCode::from(1);
            }
        };

        println!("Separating essentials from non-essentials ...");

        for package in &downloaded_package_file_names {
            let mut package_name: String = String::new();

            match extract_deb_control_field(
                &target_extractor,
                &format!("{downloaded_packages_directory}/{package}"),
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
                    clean_up_on_exit(
                        &workspace_directory,
                        Some(&target_bootstrap_directory),
                        &target_actions_to_skip,
                        &message_config,
                    )
                    .unwrap_or(());

                    return ExitCode::from(1);
                }
            };

            let mut is_package_essential: bool = false;

            for package in &target_essential_subset {
                if &package.name == &package_name {
                    is_package_essential = true;
                    break;
                };
            }

            if is_package_essential == true {
                print_message(
                    "debug",
                    &format!("moving essential package \"{package}\" to \"{initial_packages_directory}\""),
                    &message_config,
                );

                if std::fs::rename(
                    format!("{downloaded_packages_directory}/{package}"),
                    format!("{initial_packages_directory}/{package}"),
                )
                .is_err()
                    == true
                {
                    print_message(
                        "error",
                        &format!(
                            "failed to move file \"{downloaded_packages_directory}/{package}\""
                        ),
                        &message_config,
                    );

                    clean_up_on_exit(
                        &workspace_directory,
                        Some(&target_bootstrap_directory),
                        &target_actions_to_skip,
                        &message_config,
                    )
                    .unwrap_or(());

                    return ExitCode::from(1);
                };
            } else {
                if Path::new(&remaining_packages_directory).exists() == false {
                    print_message(
                        "debug",
                        &format!("creating directory \"{remaining_packages_directory}\""),
                        &message_config,
                    );

                    if std::fs::create_dir(&remaining_packages_directory).is_err() == true {
                        print_message(
                            "error",
                            &format!(
                                "failed to create directory \"{remaining_packages_directory}\""
                            ),
                            &message_config,
                        );

                        clean_up_on_exit(
                            &workspace_directory,
                            Some(&target_bootstrap_directory),
                            &target_actions_to_skip,
                            &message_config,
                        )
                        .unwrap_or(());

                        return ExitCode::from(1);
                    };
                };

                print_message(
                    "debug",
                    &format!("moving non-essential package \"{package}\" to \"{remaining_packages_directory}\""),
                    &message_config,
                );

                if std::fs::rename(
                    format!("{downloaded_packages_directory}/{package}"),
                    format!("{remaining_packages_directory}/{package}"),
                )
                .is_err()
                    == true
                {
                    print_message(
                        "error",
                        &format!(
                            "failed to move file \"{downloaded_packages_directory}/{package}\""
                        ),
                        &message_config,
                    );

                    clean_up_on_exit(
                        &workspace_directory,
                        Some(&target_bootstrap_directory),
                        &target_actions_to_skip,
                        &message_config,
                    )
                    .unwrap_or(());

                    return ExitCode::from(1);
                };
            };
        }
    } else {
        for package in &downloaded_package_file_names {
            print_message(
                "debug",
                &format!("moving package \"{package}\" to \"{initial_packages_directory}\""),
                &message_config,
            );

            if std::fs::rename(
                format!("{downloaded_packages_directory}/{package}"),
                format!("{initial_packages_directory}/{package}"),
            )
            .is_err()
                == true
            {
                print_message(
                    "error",
                    &format!("failed to move file \"{downloaded_packages_directory}/{package}\""),
                    &message_config,
                );

                clean_up_on_exit(
                    &workspace_directory,
                    Some(&target_bootstrap_directory),
                    &target_actions_to_skip,
                    &message_config,
                )
                .unwrap_or(());

                return ExitCode::from(1);
            };
        }
    };

    if print_debug == true {
        let list_of_initial_packages: Vec<String> = std::fs::read_dir(&initial_packages_directory)
            .unwrap()
            .map(|element| String::from(element.unwrap().path().to_string_lossy()))
            .collect::<Vec<String>>();

        print_message(
            "debug",
            &format!(
                "# of initial packages: \"{}\"",
                list_of_initial_packages.len()
            ),
            &message_config,
        );

        if Path::new(&remaining_packages_directory).exists() == true {
            let list_of_remaining_packages: Vec<String> =
                std::fs::read_dir(&remaining_packages_directory)
                    .unwrap()
                    .map(|element| String::from(element.unwrap().path().to_string_lossy()))
                    .collect::<Vec<String>>();

            print_message(
                "debug",
                &format!(
                    "# of remaining packages: \"{}\"",
                    list_of_remaining_packages.len(),
                ),
                &message_config,
            );
        };
    };

    print_message(
        "debug",
        &format!("removing directory \"{downloaded_packages_directory}\""),
        &message_config,
    );

    if std::fs::remove_dir(&downloaded_packages_directory).is_err() == true {
        print_message(
            "error",
            &format!("failed to remove directory \"{downloaded_packages_directory}\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    //////////////////////////////////////////////

    if merge_usr_directories == Some(String::from("manual_usr_merge")) {
        if manually_merge_usr_directories(
            &target_bootstrap_directory,
            &target_architectures,
            &message_config,
        )
        .is_err()
            == true
        {
            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    };

    //////////////////////////////////////////////

    println!("Extracting packages:");

    print_message(
        "debug",
        &format!("extracting all packages in \"{initial_packages_directory}\" with \"{target_extractor}\""),
        &message_config,
    );

    let mut packages_to_extract: Vec<String> = std::fs::read_dir(&initial_packages_directory)
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

    packages_to_extract.sort_unstable();

    let mut counter_spacing: u16 = 0;

    if packages_to_extract.len() < 10 {
        counter_spacing = 6;
    } else if packages_to_extract.len() < 100 {
        counter_spacing = 8;
    } else if packages_to_extract.len() < 1000 {
        counter_spacing = 10;
    } else if packages_to_extract.len() < 10000 {
        counter_spacing = 12;
    } else if packages_to_extract.len() < 100000 {
        counter_spacing = 14;
    };

    let counter_spacing: u16 = counter_spacing;

    let mut counter: u16 = 0;

    for package in &packages_to_extract {
        counter += 1;

        let extract_counter: String = space_and_truncate_string(
            &format!("({counter}/{}):", packages_to_extract.len(),),
            counter_spacing,
        );

        println!("{extract_counter} {package}");

        if extract_deb_data(
            &target_extractor,
            &format!("{initial_packages_directory}/{package}"),
            &target_bootstrap_directory,
            &message_config,
        )
        .is_err()
            == true
        {
            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////

    print_message(
        "debug",
        &format!("creating directory \"{target_bootstrap_directory}/packages\""),
        &message_config,
    );

    if std::fs::create_dir(&format!("{target_bootstrap_directory}/packages")).is_err() == true {
        print_message(
            "error",
            &format!("failed to create directory \"{target_bootstrap_directory}/packages\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    print_message(
        "debug",
        &format!("creating directory \"{target_bootstrap_directory}/packages/initial\""),
        &message_config,
    );

    if std::fs::create_dir(&format!("{target_bootstrap_directory}/packages/initial")).is_err()
        == true
    {
        print_message(
            "error",
            &format!(
                "failed to create directory \"{target_bootstrap_directory}/packages/initial\""
            ),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    let mut initial_package_file_names: Vec<String> =
        std::fs::read_dir(&initial_packages_directory)
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

    initial_package_file_names.sort_unstable();

    print_message(
        "debug",
        &format!("moving packages from \"{initial_packages_directory}\" to \"{target_bootstrap_directory}/packages/initial\""),
        &message_config,
    );

    for package in &initial_package_file_names {
        if move_file(
            format!("{initial_packages_directory}/{package}"),
            format!("{target_bootstrap_directory}/packages/initial/{package}"),
        )
        .is_err()
            == true
        {
            print_message(
                "error",
                &format!("failed to move file \"{initial_packages_directory}/{package}\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    }

    print_message(
        "debug",
        &format!("removing directory \"{initial_packages_directory}\""),
        &message_config,
    );

    if std::fs::remove_dir(&initial_packages_directory).is_err() == true {
        print_message(
            "error",
            &format!("failed to remove directory \"{initial_packages_directory}\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    if Path::new(&remaining_packages_directory).exists() == true {
        print_message(
            "debug",
            &format!("creating directory \"{target_bootstrap_directory}/packages/remaining\""),
            &message_config,
        );

        if std::fs::create_dir(&format!("{target_bootstrap_directory}/packages/remaining")).is_err()
            == true
        {
            print_message(
                "error",
                &format!("failed to create directory \"{target_bootstrap_directory}/packages/remaining\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };

        let mut remaining_package_file_names: Vec<String> =
            std::fs::read_dir(&remaining_packages_directory)
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

        print_message(
            "debug",
            &format!("moving packages from \"{remaining_packages_directory}\" to \"{target_bootstrap_directory}/packages/remaining\""),
            &message_config,
        );

        for package in &remaining_package_file_names {
            if move_file(
                format!("{remaining_packages_directory}/{package}"),
                format!("{target_bootstrap_directory}/packages/remaining/{package}"),
            )
            .is_err()
                == true
            {
                print_message(
                    "error",
                    &format!("failed to move file \"{remaining_packages_directory}/{package}\""),
                    &message_config,
                );

                clean_up_on_exit(
                    &workspace_directory,
                    Some(&target_bootstrap_directory),
                    &target_actions_to_skip,
                    &message_config,
                )
                .unwrap_or(());

                return ExitCode::from(1);
            };
        }

        print_message(
            "debug",
            &format!("removing directory \"{remaining_packages_directory}\""),
            &message_config,
        );

        if std::fs::remove_dir(&remaining_packages_directory).is_err() == true {
            print_message(
                "error",
                &format!("failed to remove directory \"{remaining_packages_directory}\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    };

    let all_packages_directory: String = format!("{target_bootstrap_directory}/packages");

    let initial_packages_directory: String = format!("{all_packages_directory}/initial");

    let remaining_packages_directory: String = format!("{all_packages_directory}/remaining");

    //////////////////////////////////////////////

    if target_package_set.contains(package_database.get("dash").unwrap()) == false {
        print_message(
            "debug",
            &format!("temporarily linking \"{target_bootstrap_directory}/bin/bash\" to \"{target_bootstrap_directory}/bin/sh\""),
            &message_config,
        );

        if run_cmd!(ln --symbolic --relative "$target_bootstrap_directory/bin/bash" "$target_bootstrap_directory/bin/sh" 2> /dev/stdout).is_err() == true {
            print_message(
                "error",
                &format!("failed to create link \"{target_bootstrap_directory}/bin/sh\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            ).unwrap_or(());

            return ExitCode::from(1);
        };

        print_message(
            "debug",
            &format!("temporarily linking \"{target_bootstrap_directory}/bin/bash\" to \"{target_bootstrap_directory}/bin/dash\""),
            &message_config,
        );

        if run_cmd!(ln --symbolic --relative "$target_bootstrap_directory/bin/bash" "$target_bootstrap_directory/bin/dash" 2> /dev/stdout).is_err() == true {
            print_message(
                "error",
                &format!("failed to create link \"{target_bootstrap_directory}/bin/dash\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            ).unwrap_or(());

            return ExitCode::from(1);
        };
    };

    if target_package_set.contains(package_database.get("mawk").unwrap()) == true {
        print_message(
            "debug",
            &format!("temporarily linking \"{target_bootstrap_directory}/usr/bin/mawk\" to \"{target_bootstrap_directory}/usr/bin/awk\""),
            &message_config,
        );

        if run_cmd!(ln --symbolic --relative "$target_bootstrap_directory/usr/bin/mawk" "$target_bootstrap_directory/usr/bin/awk" 2> /dev/stdout).is_err() == true {
            print_message(
                "error",
                &format!("failed to create link \"{target_bootstrap_directory}/usr/bin/awk\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            ).unwrap_or(());

            return ExitCode::from(1);
        };
    } else if target_package_set.contains(package_database.get("original-awk").unwrap()) == true {
        print_message(
            "debug",
            &format!("temporarily linking \"{target_bootstrap_directory}/usr/bin/original-awk\" to \"{target_bootstrap_directory}/usr/bin/awk\""),
            &message_config,
        );

        if run_cmd!(ln --symbolic --relative "$target_bootstrap_directory/usr/bin/original-awk" "$target_bootstrap_directory/usr/bin/awk" 2> /dev/stdout).is_err() == true {
            print_message(
                "error",
                &format!("failed to create link \"{target_bootstrap_directory}/usr/bin/awk\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            ).unwrap_or(());

            return ExitCode::from(1);
        };
    } else if target_package_set.contains(package_database.get("gawk").unwrap()) == true {
        print_message(
            "debug",
            &format!("temporarily linking \"{target_bootstrap_directory}/usr/bin/gawk\" to \"{target_bootstrap_directory}/usr/bin/awk\""),
            &message_config,
        );

        if run_cmd!(ln --symbolic --relative "$target_bootstrap_directory/usr/bin/gawk" "$target_bootstrap_directory/usr/bin/awk" 2> /dev/stdout).is_err() == true {
            print_message(
                "error",
                &format!("failed to create link \"{target_bootstrap_directory}/usr/bin/awk\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            ).unwrap_or(());

            return ExitCode::from(1);
        };
    };

    //////////////////////////////////////////////

    if create_file(
        &format!("{target_bootstrap_directory}/var/lib/dpkg/status"),
        "",
        &message_config,
    )
    .is_err()
        == true
    {
        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    if create_file(
        &format!("{target_bootstrap_directory}/var/lib/dpkg/available"),
        "",
        &message_config,
    )
    .is_err()
        == true
    {
        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    if create_file(
        &format!("{target_bootstrap_directory}/var/lib/dpkg/arch"),
        "",
        &message_config,
    )
    .is_err()
        == true
    {
        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    for architecture in &target_architectures {
        print_message(
            "debug",
            &format!("adding architecture \"{architecture}\" to \"{target_bootstrap_directory}/var/lib/dpkg/arch\""),
            &message_config,
        );

        if std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(format!("{target_bootstrap_directory}/var/lib/dpkg/arch"))
            .unwrap()
            .write_all(format!("{architecture}\n").as_bytes())
            .is_err()
            == true
        {
            print_message(
                "error",
                &format!(
                    "failed to write to file \"{target_bootstrap_directory}/var/lib/dpkg/arch\""
                ),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    }

    //////////////////////////////////////////////

    print_message(
        "debug",
        &format!("creating default file \"{target_bootstrap_directory}/etc/fstab\""),
        &message_config,
    );

    if create_file(
        &format!("{target_bootstrap_directory}/etc/fstab"),
        "\
# UNCONFIGURED FSTAB FOR BASE SYSTEM

# See fstab(5) for more information.
",
        &message_config,
    )
    .is_err()
        == true
    {
        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    if run_cmd!(chown root:root "$target_bootstrap_directory/etc/fstab" 2> /dev/stdout).is_err()
        == true
    {
        print_message(
            "error",
            &format!(
                "failed to change permissions of file \"{target_bootstrap_directory}/etc/fstab\""
            ),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    if run_cmd!(chmod 0644 "$target_bootstrap_directory/etc/fstab" 2> /dev/stdout).is_err() == true
    {
        print_message(
            "error",
            &format!(
                "failed to change permissions of file \"{target_bootstrap_directory}/etc/fstab\""
            ),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    print_message(
        "debug",
        &format!("creating default file \"{target_bootstrap_directory}/etc/hosts\""),
        &message_config,
    );

    if create_file(
        &format!("{target_bootstrap_directory}/etc/hosts"),
        "\
# UNCONFIGURED HOSTS FOR BASE SYSTEM

# Example with hostname 'debian' for IPv4 and IPv6 capable hosts:
#127.0.0.1  localhost
#127.0.1.1  debian.localdomain  debian
#::1        localhost ip6-localhost ip6-loopback
#ff02::1    ip6-allnodes
#ff02::2    ip6-allrouters

# See hosts(5) for more information.
",
        &message_config,
    )
    .is_err()
        == true
    {
        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    //////////////////////////////////////////////

    if target_package_set.contains(package_database.get("apt").unwrap()) == true {
        match &sources_list_format as &str {
            "deb822-style" => {
                if Path::new(&format!(
                    "{target_bootstrap_directory}/etc/apt/sources.list.d"
                ))
                .exists()
                    == false
                {
                    print_message(
                        "debug",
                        &format!("creating directory \"{target_bootstrap_directory}/etc/apt/sources.list.d\""),
                        &message_config,
                    );

                    if std::fs::create_dir_all(&format!(
                        "{target_bootstrap_directory}/etc/apt/sources.list.d"
                    ))
                    .is_err()
                        == true
                    {
                        print_message(
                            "error",
                            &format!("failed to create directory \"{target_bootstrap_directory}/etc/apt/sources.list.d\""),
                            &message_config,
                        );

                        clean_up_on_exit(
                            &workspace_directory,
                            Some(&target_bootstrap_directory),
                            &target_actions_to_skip,
                            &message_config,
                        )
                        .unwrap_or(());

                        return ExitCode::from(1);
                    };
                };

                print_message(
                    "debug",
                    &format!("creating default deb822-style sources list \"{target_bootstrap_directory}/etc/apt/sources.list.d/sources.sources\""),
                    &message_config,
                );

                if create_file(
                    &format!("{target_bootstrap_directory}/etc/apt/sources.list.d/sources.sources"),
                    &format!(
                        "\
Types: deb deb-src
URIs: {}
Suites: {}
Components: {}
",
                        format!("{:?}", &target_mirrors).replace(['[', ']', '"', ','], ""),
                        format!("{:?}", &target_suites).replace(['[', ']', '"', ','], ""),
                        format!("{:?}", &target_components).replace(['[', ']', '"', ','], ""),
                    ),
                    &message_config,
                )
                .is_err()
                    == true
                {
                    clean_up_on_exit(
                        &workspace_directory,
                        Some(&target_bootstrap_directory),
                        &target_actions_to_skip,
                        &message_config,
                    )
                    .unwrap_or(());

                    return ExitCode::from(1);
                };
            }
            "one-line-style" => {
                print_message(
                    "debug",
                    &format!("creating default one-line-style sources list \"{target_bootstrap_directory}/etc/apt/sources.list\""),
                    &message_config,
                );

                if create_file(
                    &format!("{target_bootstrap_directory}/etc/apt/sources.list"),
                    "",
                    &message_config,
                )
                .is_err()
                    == true
                {
                    clean_up_on_exit(
                        &workspace_directory,
                        Some(&target_bootstrap_directory),
                        &target_actions_to_skip,
                        &message_config,
                    )
                    .unwrap_or(());

                    return ExitCode::from(1);
                };

                let mut mirror_counter: u16 = 0;

                for mirror in &target_mirrors {
                    mirror_counter += 1;

                    if mirror_counter != 1 {
                        if std::fs::OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(format!("{target_bootstrap_directory}/etc/apt/sources.list"))
                            .unwrap()
                            .write_all(b"\n")
                            .is_err()
                            == true
                        {
                            print_message(
                                "error",
                                &format!("failed to write to file \"{target_bootstrap_directory}/etc/apt/sources.list\""),
                                &message_config,
                            );

                            clean_up_on_exit(
                                &workspace_directory,
                                Some(&target_bootstrap_directory),
                                &target_actions_to_skip,
                                &message_config,
                            )
                            .unwrap_or(());

                            return ExitCode::from(1);
                        };
                    };

                    let mut suite_counter: u16 = 0;

                    for suite in &target_suites {
                        suite_counter += 1;

                        if suite_counter != 1 {
                            if std::fs::OpenOptions::new()
                                .write(true)
                                .append(true)
                                .open(format!("{target_bootstrap_directory}/etc/apt/sources.list"))
                                .unwrap()
                                .write_all(b"\n")
                                .is_err()
                                == true
                            {
                                print_message(
                                    "error",
                                    &format!("failed to write to file \"{target_bootstrap_directory}/etc/apt/sources.list\""),
                                    &message_config,
                                );

                                clean_up_on_exit(
                                    &workspace_directory,
                                    Some(&target_bootstrap_directory),
                                    &target_actions_to_skip,
                                    &message_config,
                                )
                                .unwrap_or(());

                                return ExitCode::from(1);
                            };
                        };

                        if std::fs::OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(format!("{target_bootstrap_directory}/etc/apt/sources.list"))
                            .unwrap()
                            .write_all(
                                format!(
                                    "deb-src {mirror} {suite} {}\ndeb {mirror} {suite} {}\n",
                                    format!("{:?}", &target_components)
                                        .replace(['[', ']', '"', ','], ""),
                                    format!("{:?}", &target_components)
                                        .replace(['[', ']', '"', ','], "")
                                )
                                .as_bytes(),
                            )
                            .is_err()
                            == true
                        {
                            print_message(
                                "error",
                                &format!("failed to write to file \"{target_bootstrap_directory}/etc/apt/sources.list\""),
                                &message_config,
                            );

                            clean_up_on_exit(
                                &workspace_directory,
                                Some(&target_bootstrap_directory),
                                &target_actions_to_skip,
                                &message_config,
                            )
                            .unwrap_or(());

                            return ExitCode::from(1);
                        };
                    }
                }
            }
            _ => {}
        };
    };

    //////////////////////////////////////////////

    if merge_usr_directories == None && is_split_usr_supported(&target_suites[0]) == false {
        print_message(
            "debug",
            &format!("creating warning file \"{target_bootstrap_directory}/etc/unsupported-skip-usrmerge-conversion\""),
            &message_config,
        );

        if create_file(
            &format!("{target_bootstrap_directory}/etc/unsupported-skip-usrmerge-conversion"),
            "this system will not be supported in the future",
            &message_config,
        )
        .is_err()
            == true
        {
            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    };

    //////////////////////////////////////////////

    if extract_hooks.len() != 0 {
        run_hooks(
            "extract",
            &extract_hooks,
            &workspace_directory,
            Some(&all_packages_directory),
            Some(&target_bootstrap_directory),
            &message_config,
        );
    };

    //////////////////////////////////////////////

    if only_action_then_exit == "extract_packages" {
        match &target_output_format as &str {
            "tarball" => {
                println!("Packaging (this might take a while) ...");

                print_message(
                    "debug",
                    &format!("creating tarball \"{target_output_directory}/{target_output_file_name}.tar\""),
                    &message_config,
                );

                if run_cmd!(
                    bash -c "
                        cd '$target_bootstrap_directory'
                        tar --create --file='$target_output_directory/$target_output_file_name.tar' *
                    " 2> /dev/stdout
                )
                .is_err()
                    == true
                {
                    print_message(
                        "error",
                        &format!("failed to create tarball \"{target_output_directory}/{target_output_file_name}.tar\""),
                        &message_config,
                    );

                    clean_up_on_exit(
                        &workspace_directory,
                        Some(&target_bootstrap_directory),
                        &target_actions_to_skip,
                        &message_config,
                    ).unwrap_or(());

                    return ExitCode::from(1);
                };
            }
            _ => {}
        };

        println!("Package extraction complete.");

        if clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .is_err()
            == true
        {
            return ExitCode::from(1);
        };

        return ExitCode::from(0);
    };

    ////////////////////////////////////////////////////////////////////////////////////////////////

    println!("Installing packages:");

    print_message(
        "debug",
        &format!("creating temporary file \"{target_bootstrap_directory}/usr/sbin/policy-rc.d\""),
        &message_config,
    );

    if create_file(
        &format!("{target_bootstrap_directory}/usr/sbin/policy-rc.d"),
        "#! /bin/sh\n\nexit 101\n",
        &message_config,
    )
    .is_err()
        == true
    {
        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    if run_cmd!(chmod 0755 "$target_bootstrap_directory/usr/sbin/policy-rc.d" 2> /dev/stdout)
        .is_err()
        == true
    {
        print_message(
            "error",
            &format!("failed to change permissions of file \"{target_bootstrap_directory}/usr/sbin/policy-rc.d\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    let mut start_stop_daemon_location: String = String::new();

    if Path::new(&format!(
        "{target_bootstrap_directory}/sbin/start-stop-daemon"
    ))
    .is_file()
        == true
    {
        start_stop_daemon_location = format!("{target_bootstrap_directory}/sbin/start-stop-daemon");
    } else if Path::new(&format!(
        "{target_bootstrap_directory}/usr/sbin/start-stop-daemon"
    ))
    .is_file()
        == true
    {
        start_stop_daemon_location =
            format!("{target_bootstrap_directory}/usr/sbin/start-stop-daemon");
    };

    let start_stop_daemon_location: String = start_stop_daemon_location;

    print_message(
        "debug",
        &format!("renaming file \"{start_stop_daemon_location}\" to \"{start_stop_daemon_location}.ORIGINAL\""),
        &message_config,
    );
    if std::fs::rename(
        format!("{start_stop_daemon_location}"),
        format!("{start_stop_daemon_location}.ORIGINAL"),
    )
    .is_err()
        == true
    {
        print_message(
            "error",
            &format!("failed to rename file \"{start_stop_daemon_location}\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    print_message(
        "debug",
        &format!("creating temporary file \"{start_stop_daemon_location}\""),
        &message_config,
    );

    if create_file(
        &format!("{start_stop_daemon_location}"),
        "#! /bin/sh\n\necho -e '\\nWarning: Fake start-stop-daemon called, doing nothing'\n\nexit 0\n",
        &message_config,
    ).is_err() == true {
        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        ).unwrap_or(());

        return ExitCode::from(1);
    };

    if run_cmd!(chmod 0755 "$start_stop_daemon_location" 2> /dev/stdout).is_err() == true {
        print_message(
            "error",
            &format!("failed to change permissions of file \"{start_stop_daemon_location}\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    //////////////////////////////////////////////

    if mount_virtual_kernel_file_systems(&target_bootstrap_directory, &message_config).is_err()
        == true
    {
        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    if target_package_set.contains(package_database.get("dash").unwrap()) == false {
        if run_cmd!(
            chroot "$target_bootstrap_directory" /usr/bin/env --ignore-environment bash -c "
export HOME='/root'
export TERM='$term_environment_variable'
export PATH

update-alternatives --force --install /bin/sh sh /bin/bash 999

update-alternatives --force --install /bin/dash dash /bin/bash 999
" 2> /dev/stdout
        )
        .is_err()
            == true
        {
            print_message(
                "error",
                &format!("failed to create alternatives for \"{target_bootstrap_directory}/bin/sh\" and \"{target_bootstrap_directory}/bin/dash\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    };

    //////////////////////////////////////////////

    print_message(
        "debug",
        &format!("installing all packages in \"{initial_packages_directory}\""),
        &message_config,
    );

    if run_cmd!(
        chroot "$target_bootstrap_directory" /usr/bin/env --ignore-environment bash -c "
export HOME='/root'
export TERM='$term_environment_variable'
export PATH
export DEBIAN_FRONTEND='$debian_frontend'
export DEBCONF_NONINTERACTIVE_SEEN='$debconf_noninteractive_seen'
export DEBCONF_NOWARNINGS='yes'
export DPKG_COLORS='$colorful_dpkg'

cd /packages/initial

dpkg --force-depends --force-confold --install *.deb
" 2> /dev/stdout
    )
    .is_err()
        == true
    {
        print_message(
            "error",
            &format!("failed to install packages in \"{initial_packages_directory}\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    //////////////////////////////////////////////

    if essential_hooks.len() != 0 {
        run_hooks(
            "essential",
            &essential_hooks,
            &workspace_directory,
            Some(&all_packages_directory),
            Some(&target_bootstrap_directory),
            &message_config,
        );
    };

    //////////////////////////////////////////////

    if Path::new(&remaining_packages_directory).exists() == true {
        print_message(
            "debug",
            &format!("installing all packages in \"{remaining_packages_directory}\""),
            &message_config,
        );

        if run_cmd!(
            chroot "$target_bootstrap_directory" /usr/bin/env --ignore-environment bash -c "
export HOME='/root'
export TERM='$term_environment_variable'
export PATH
export DEBIAN_FRONTEND='$debian_frontend'
export DEBCONF_NONINTERACTIVE_SEEN='$debconf_noninteractive_seen'
export DEBCONF_NOWARNINGS='yes'
export DPKG_COLORS='$colorful_dpkg'

cd /packages/remaining

dpkg --force-depends --force-confold --install *.deb
" 2> /dev/stdout
        )
        .is_err()
            == true
        {
            print_message(
                "error",
                &format!("failed to install packages in \"{remaining_packages_directory}\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    };

    //////////////////////////////////////////////

    if target_hooks.len() != 0 {
        run_hooks(
            "target",
            &target_hooks,
            &workspace_directory,
            Some(&all_packages_directory),
            Some(&target_bootstrap_directory),
            &message_config,
        );
    };

    //////////////////////////////////////////////

    if unmount_virtual_kernel_file_systems(&target_bootstrap_directory, &message_config).is_err()
        == true
    {
        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    print_message(
        "debug",
        &format!("removing temporary file \"{target_bootstrap_directory}/usr/sbin/policy-rc.d\""),
        &message_config,
    );

    if std::fs::remove_file(format!("{target_bootstrap_directory}/usr/sbin/policy-rc.d")).is_err()
        == true
    {
        print_message(
            "error",
            &format!("failed to remove file \"{target_bootstrap_directory}/usr/sbin/policy-rc.d\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    print_message(
        "debug",
        &format!("removing temporary file \"{start_stop_daemon_location}\""),
        &message_config,
    );

    if std::fs::remove_file(format!("{start_stop_daemon_location}")).is_err() == true {
        print_message(
            "error",
            &format!("failed to remove file \"{start_stop_daemon_location}\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    print_message(
        "debug",
        &format!("renaming file \"{start_stop_daemon_location}.ORIGINAL\" to \"{start_stop_daemon_location}\""),
        &message_config,
    );

    if std::fs::rename(
        format!("{start_stop_daemon_location}.ORIGINAL"),
        format!("{start_stop_daemon_location}"),
    )
    .is_err()
        == true
    {
        print_message(
            "error",
            &format!("failed to rename file \"{start_stop_daemon_location}.ORIGINAL\""),
            &message_config,
        );

        clean_up_on_exit(
            &workspace_directory,
            Some(&target_bootstrap_directory),
            &target_actions_to_skip,
            &message_config,
        )
        .unwrap_or(());

        return ExitCode::from(1);
    };

    //////////////////////////////////////////////

    if target_actions_to_skip.contains(&String::from("packages_removal")) == true {
        print_message(
            "debug",
            &format!("not removing directory \"{all_packages_directory}\""),
            &message_config,
        );
    } else {
        print_message(
            "debug",
            &format!("removing directory \"{all_packages_directory}\""),
            &message_config,
        );

        if std::fs::remove_dir_all(&all_packages_directory).is_err() == true {
            print_message(
                "error",
                &format!("failed to remove directory \"{all_packages_directory}\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    };

    if Path::new(&format!("{target_bootstrap_directory}/etc/machine-id")).exists() == true {
        print_message(
            "debug",
            &format!("removing file \"{target_bootstrap_directory}/etc/machine-id\""),
            &message_config,
        );

        if std::fs::remove_file(format!("{target_bootstrap_directory}/etc/machine-id")).is_err()
            == true
        {
            print_message(
                "error",
                &format!("failed to remove file \"{target_bootstrap_directory}/etc/machine-id\""),
                &message_config,
            );

            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };

        print_message(
            "debug",
            &format!("creating default file \"{target_bootstrap_directory}/etc/machine-id\" with value \"uninitialized\""),
            &message_config,
        );

        if create_file(
            &format!("{target_bootstrap_directory}/etc/machine-id"),
            "uninitialized\n",
            &message_config,
        )
        .is_err()
            == true
        {
            clean_up_on_exit(
                &workspace_directory,
                Some(&target_bootstrap_directory),
                &target_actions_to_skip,
                &message_config,
            )
            .unwrap_or(());

            return ExitCode::from(1);
        };
    };

    //////////////////////////////////////////////

    if done_hooks.len() != 0 {
        run_hooks(
            "done",
            &done_hooks,
            &workspace_directory,
            None,
            Some(&target_bootstrap_directory),
            &message_config,
        );
    };

    //////////////////////////////////////////////

    match &target_output_format as &str {
        "tarball" => {
            println!("Packaging (this might take a while) ...");

            print_message(
                "debug",
                &format!(
                    "creating tarball \"{target_output_directory}/{target_output_file_name}.tar\""
                ),
                &message_config,
            );

            if run_cmd!(
                bash -c "
                    cd '$target_bootstrap_directory'
                    tar --create --file='$target_output_directory/$target_output_file_name.tar' *
                " 2> /dev/stdout
            )
            .is_err()
                == true
            {
                print_message(
                    "error",
                    &format!(
                        "failed to create tarball \"{target_output_directory}/{target_output_file_name}.tar\""
                    ),
                    &message_config,
                );

                clean_up_on_exit(
                    &workspace_directory,
                    Some(&target_bootstrap_directory),
                    &target_actions_to_skip,
                    &message_config,
                )
                .unwrap_or(());

                return ExitCode::from(1);
            };
        }
        _ => {}
    };

    println!("Bootstrap complete.");

    if clean_up_on_exit(
        &workspace_directory,
        Some(&target_bootstrap_directory),
        &target_actions_to_skip,
        &message_config,
    )
    .is_err()
        == true
    {
        return ExitCode::from(1);
    };

    return ExitCode::from(0);
}
