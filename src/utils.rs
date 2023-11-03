//! Utilitary functions for commands.

use std::{
    io::BufRead,
    os::unix::prelude::{MetadataExt, PermissionsExt},
};

/// A clojure that returns true if current character matches the token.
/// Its inputs are the string to match and the current position in the string.
type TokenValidator = Box<dyn Fn(&Vec<char>, &mut usize) -> bool>;

type Pattern = Vec<TokenValidator>;

/// Split args into options (flags) and arguments.
pub fn extract_options(args: &[String]) -> (Vec<&String>, Vec<&String>) {
    args.iter().partition(|arg| arg.starts_with("-"))
}

/// Compile a pattern into a list of clojures.
pub fn compile_expr(pattern_str: &String) -> Pattern {
    let mut pattern = Pattern::new();

    for token in pattern_str.chars() {
        let validator: TokenValidator = match token {
            '^' => Box::new(|_, i| *i == 0),
            '$' => Box::new(|s, i| *i == s.len()),
            '*' => {
                // Take the previous clojure and create a
                // new one that matches it 0 or more times.
                let Some(prev_validator) = pattern.pop() else {
                    eprintln!("grep: invalid pattern '{}'", pattern_str);
                    std::process::exit(-100);
                };

                Box::new(move |s, i| {
                    // Match the previous clojure as many times as possible.
                    while prev_validator(s, i) && *i < s.len() {}

                    // Go back to the character that didn't match the
                    // wildcard so it can be matched with the next token.
                    *i -= 1;
                    true
                })
            }
            '.' => Box::new(|_, i| {
                *i += 1;

                // Match anything.
                true
            }),
            chr => Box::new(move |s, i| {
                *i += 1;
                s.get(*i - 1) == Some(&chr)
            }),
        };

        pattern.push(validator);
    }

    pattern
}

/// Try to match a pattern starting at the beginning of a string.
fn match_substr(pattern: &Pattern, string: &Vec<char>, start_pos: usize) -> bool {
    let mut pattern_cursor = pattern.iter();
    let mut i = start_pos;

    loop {
        let Some(token_action) = pattern_cursor.next() else {
            // Pattern ended.
            return true;
        };

        if !token_action(string, &mut i) {
            // Pattern didn't match.
            break;
        }
    }

    // Pattern didn't match or string
    // ended before the pattern.
    false
}

/// Try to match a pattern against a substring of `string`.
pub fn match_expr(pattern: &Pattern, string: &String) -> bool {
    // Convert to Vec<char> for O(1) random access (because
    // String contains variable size chars).
    let chars = string.chars().collect::<Vec<_>>();

    for i in 0..chars.len() {
        if match_substr(&pattern, &chars, i) {
            return true;
        }
    }

    false
}

/// Search the name of a user or group by its id in the file `path`.
/// The file must be formatted like `/etc/passwd` or `/etc/group`.
fn search_id_name(path: &str, target_id: u32) -> Option<String> {
    let Ok(file) = std::fs::File::open(path) else {
        return None;
    };

    for line in std::io::BufReader::new(file).lines() {
        let Ok(line) = line else {
            return None;
        };

        let mut fields = line.split(':');

        // `name` and `id` are fields 0 and 2.
        let name = fields.next();
        let id = fields.nth(1);

        let (Some(name), Some(id)) = (name, id) else {
            return None;
        };

        let Ok(id) = id.parse::<u32>() else {
            return None;
        };

        if id == target_id {
            return Some(name.to_owned());
        }
    }

    None
}

fn print_file_info(path: &String, long: bool) {
    if !long {
        println!("{}", path);
        return;
    }

    let Ok(metadata) = std::fs::metadata(path) else {
        eprintln!("ls: failed reading metadata for '{}'", path);
        std::process::exit(-80);
    };

    let file_type = metadata.file_type();
    let file_size = metadata.len();
    let file_mode = metadata.permissions().mode();

    let mut formatted_mode = String::new();
    formatted_mode.push(if file_type.is_symlink() {
        'l'
    } else if file_type.is_dir() {
        'd'
    } else {
        '-'
    });

    for group_mask in (0..3).rev() {
        let group_perm = file_mode >> (group_mask * 3);

        for (i, chr) in "rwx".chars().enumerate() {
            if group_perm & (1 << (2 - i)) != 0 {
                formatted_mode.push(chr);
            } else {
                formatted_mode.push('-');
            }
        }
    }

    let Some(owner) = search_id_name("/etc/passwd", metadata.uid()) else {
        std::process::exit(-80);
    };
    let Some(group) = search_id_name("/etc/group", metadata.gid()) else {
        std::process::exit(-80);
    };

    let Ok(mtime) = metadata.modified() else {
        std::process::exit(-80);
    };

    let mtime: chrono::DateTime<chrono::Local> = chrono::DateTime::from(mtime);

    println!(
        "{} {} {} {} {} {}",
        formatted_mode,
        owner,
        group,
        file_size,
        mtime.format("%b %-e %H:%M"),
        path
    );
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

/// Converts permision literals from "symbolic mode" into bits.
/// Also return whether they are to be added or removed.
/// Returns `None` if `mode_str` is invalid.
pub fn convert_mode(mode_str: &String) -> Option<(u32, bool)> {
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

    Some((user_mask & mode_mask, add_perms))
}
