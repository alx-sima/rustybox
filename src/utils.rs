//! Utilitary functions for commands.

/// Split args into options (flags) and arguments.
pub fn extract_options(args: &[String]) -> (Vec<&String>, Vec<&String>) {
    args.iter().partition(|arg| arg.starts_with("-"))
}

fn print_file_info(path: &String, long: bool) {
    if !long {
        println!("{}", path);
        return;
    }

    todo!();
}

fn list_dir(root: &String, dir: &String, all: bool, recursive: bool, long: bool) {
    let full_path = format!("{}/{}/", root, dir);

    // If '-a' is set, list current and parent directories as well.
    if all {
        print_file_info(&format!("{}.", dir), long);
        print_file_info(&format!("{}..", dir), long);
    }

    let Ok(contents) = std::fs::read_dir(&full_path) else {
        eprintln!("ls: failed reading files from '{}'", full_path);
        std::process::exit(-80);
    };

    for entry in contents {
        let Ok(entry) = entry else {
            eprintln!("ls: failed reading files from '{}'", full_path);
            std::process::exit(-80);
        };

        if let Some(file_name) = entry.file_name().to_str() {
            // Skip hidden files unless '-a' option is present.
            if file_name.starts_with('.') && !all {
                continue;
            }

            let full_name = dir.to_string() + file_name;
            print_file_info(&full_name, long);

            let Ok(file_type) = entry.file_type() else {
                eprintln!("ls: failed retrieving metadata of '{}'", full_name);
                std::process::exit(-80);
            };

            // Recurse into directories if '-r' option is present.
            if file_type.is_dir() && recursive {
                list_dir(root, &(full_name + "/"), all, recursive, long);
            }
        } else {
            eprintln!("ls: unsupported filename encoding in '{}'", full_path);
            std::process::exit(-80);
        }
    }
}

/// List contents of a file or a directory.
pub fn list_file(path: &String, all: bool, recursive: bool, long: bool) {
    if let Ok(file_metadata) = std::fs::metadata(path) {
        if file_metadata.is_file() {
            print_file_info(path, long);
            return;
        }
    } else {
        eprintln!("ls: failed reading metadata for '{}'", path);
        std::process::exit(-80);
    }

    list_dir(path, &String::from(""), all, recursive, long);
}

/// Copy contents of 'src_root/dir/' to 'dest_root/dir/'.
pub fn copy_dir(src_root: &String, dest_root: &String, dir: &String) {
    let full_path = format!("{}/{}", src_root, dir);
    let Ok(contents) = std::fs::read_dir(&full_path) else {
        eprintln!("ls: failed reading files from '{}'", full_path);
        std::process::exit(-90);
    };

    for entry in contents {
        let Ok(entry) = entry else {
            eprintln!("ls: failed reading files from '{}'", full_path);
            std::process::exit(-90);
        };

        if let Some(file_name) = entry.file_name().to_str() {
            let full_file_name = format!("{}/{}", full_path, file_name);
            let file_name = format!("{}/{}", dir, file_name);
            let full_dest_name = format!("{}/{}", dest_root, file_name);

            let Ok(metadata) = std::fs::metadata(&full_file_name) else {
                eprintln!("ls: failed reading metadata of '{}'", full_file_name);
                std::process::exit(-90);
            };

            if metadata.is_dir() {
                if std::fs::create_dir(&full_dest_name).is_err() {
                    eprintln!("cp: failed to create directory '{}'", full_dest_name);
                    std::process::exit(-90);
                }

                copy_dir(src_root, dest_root, &file_name);
            } else {
                if std::fs::copy(&full_file_name, &full_dest_name).is_err() {
                    eprintln!(
                        "cp: failed to move {} to {}",
                        full_file_name, full_dest_name
                    );
                    std::process::exit(-90);
                }
            }
        } else {
            eprintln!("ls: unsupported filename encoding in '{}'", full_path);
            std::process::exit(-90);
        }
    }
}

/// Modifies the current `mode` with the permissions specified in `mode_str`.
/// Returns `None` if `mode_str` is invalid.
pub fn convert_mode(mode: u32, mode_str: &String) -> Option<u32> {
    let mut user_mask = 0o000;
    let mut mode_mask = 0o000;

    let mut modes = mode_str.split_inclusive(['+', '-']);
    let users = modes.next();
    let perms = modes.next();

    let (Some(user_mode), Some(perm_mode)) = (users, perms) else {
        return None;
    };

    let mut user_mode = user_mode.chars();
    let perm_mode = perm_mode.chars();

    // Check if these permissions are to be added or removed. The control
    // character will be the last of `user_mode` (because of `split_inclusive`).
    let Some(change_mode) = user_mode.next_back() else {
        return None;
    };
    let add_perms = change_mode == '+';

    for c in user_mode {
        match c {
            'u' => user_mask |= 0o700,
            'g' => user_mask |= 0o070,
            'o' => user_mask |= 0o007,
            'a' => user_mask |= 0o777,
            _ => {
                return None;
            }
        }
    }

    for c in perm_mode {
        match c {
            'r' => mode_mask |= 0o444,
            'w' => mode_mask |= 0o222,
            'x' => mode_mask |= 0o111,
            _ => {
                return None;
            }
        }
    }

    let mask = user_mask & mode_mask;
    if add_perms {
        Some(mode | mask)
    } else {
        Some(mode & !mask)
    }
}
