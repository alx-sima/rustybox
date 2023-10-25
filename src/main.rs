use std::os::unix::prelude::PermissionsExt;

/// Split args into options (flags) and arguments.
fn extract_options(args: &[String]) -> (Vec<&String>, Vec<&String>) {
    args.iter().partition(|arg| arg.starts_with("-"))
}

fn pwd() {
    if let Ok(cwd) = std::env::current_dir() {
        println!("{}", cwd.display());
    } else {
        eprintln!("pwd: failed to get current directory");
    }
}

fn echo(args: &[String]) {
    let (opts, args) = extract_options(args);
    let mut endline = true;

    for opt in opts {
        match opt.as_str() {
            "-n" => endline = false,
            _ => {
                eprintln!("Invalid command");
                std::process::exit(-10);
            }
        }
    }

    if let Some((first, args)) = args.split_first() {
        print!("{}", first);

        for arg in args {
            print!(" {}", arg);
        }

        if endline {
            println!();
        }
    } else {
        eprintln!("Invalid command");
        std::process::exit(-10);
    }
}

fn cat(args: &[String]) {
    for arg in args {
        if let Ok(contents) = std::fs::read_to_string(arg) {
            print!("{}", contents);
        } else {
            eprintln!("cat: {}: No such file or directory", arg);
            std::process::exit(-20);
        }
    }
}

fn mkdir(args: &[String]) {
    for arg in args {
        if std::fs::create_dir(arg).is_err() {
            eprintln!("mkdir: cannot create directory '{}'", arg);
            std::process::exit(-30);
        }
    }
}

fn mv(args: &[String]) {
    let [src, dst] = args else {
        eprintln!("Usage: mv SOURCE DEST");
        std::process::exit(-40);
    };

    if std::fs::rename(src, dst).is_err() {
        eprintln!("mv: cannot move '{}' to '{}'", src, dst);
        std::process::exit(-40);
    }
}

fn ln(args: &[String]) {
    let (opts, args) = extract_options(args);
    let mut symbolic = false;

    for opt in opts {
        match opt.as_str() {
            "-s" | "--symbolic" => symbolic = true,
            _ => {
                eprintln!("Invalid command");
                std::process::exit(-50);
            }
        }
    }

    let [src, dst] = args.as_slice() else {
        eprintln!("Usage: ln [OPTION]... SOURCE DEST");
        std::process::exit(-50);
    };

    let ret_status = if symbolic {
        std::os::unix::fs::symlink(src, dst)
    } else {
        std::fs::hard_link(src, dst)
    };

    if ret_status.is_err() {
        eprintln!("ln: cannot link '{}' to '{}'", src, dst);
        std::process::exit(-50);
    }
}

fn rmdir(args: &[String]) {
    for arg in args {
        if std::fs::remove_dir(arg).is_err() {
            eprintln!("rmdir: failed to remove '{}'", arg);
            std::process::exit(-60);
        }
    }
}

fn rm(args: &[String]) {
    let (opts, args) = extract_options(args);
    let mut recursive = false;
    let mut rmdir = false;

    for opt in opts {
        match opt.as_str() {
            "-r" | "--recursive" => recursive = true,
            "-d" | "--dir" => rmdir = true,
            _ => {
                eprintln!("Invalid command");
                std::process::exit(-70);
            }
        }
    }

    for arg in args {
        let ret_status = if recursive {
            std::fs::remove_dir_all(arg)
        } else if rmdir {
            std::fs::remove_dir(arg)
        } else {
            std::fs::remove_file(arg)
        };

        if ret_status.is_err() {
            eprintln!("rm: failed to remove '{}'", arg);
            std::process::exit(-70);
        }
    }
}

fn list_dirs_rec(path_root: &String, dir_name: &String, all: bool, recursive: bool) -> Vec<String> {
    let full_path = format!("{}/{}/", path_root, dir_name);

    let Ok(contents) = std::fs::read_dir(&full_path) else {
        eprintln!("ls: failed reading files from '{}'", full_path);
        std::process::exit(-80);
    };

    let mut dirs = Vec::new();
    let mut hidden_files = if all {
        vec![format!("{}.", dir_name), format!("{}..", dir_name)]
    } else {
        vec![]
    };

    for entry in contents {
        let Ok(entry) = entry else {
            eprintln!("ls: failed reading files from '{}'", full_path);
            std::process::exit(-80);
        };

        if let Some(entry_name) = entry.file_name().to_str() {
            let entry_full_name = format!("{}{}", dir_name, entry_name);

            if entry_name.starts_with(".") {
                if all {
                    hidden_files.push(entry_full_name.to_owned());
                }

                continue;
            }

            println!("{}", entry_full_name);

            if !recursive {
                continue;
            }

            let Ok(file_type) = entry.file_type() else {
                eprintln!("ls: failed reading metadata of '{}'", entry_full_name);
                std::process::exit(-80);
            };

            if file_type.is_dir() {
                dirs.push(entry_full_name);
            }
        } else {
            eprintln!("ls: unsupported filename encoding in '{}'", full_path);
            std::process::exit(-80);
        }
    }

    for dir in dirs {
        let dir_path = format!("{}/", dir);
        hidden_files.append(&mut list_dirs_rec(&path_root, &dir_path, all, recursive));
    }

    hidden_files
}

fn list_file(path: &String, all: bool, recursive: bool) {
    if let Ok(file_metadata) = std::fs::metadata(path) {
        if file_metadata.is_file() {
            println!("{}", path);
            return;
        }
    } else {
        eprintln!("ls: failed reading metadata for '{}'", path);
        std::process::exit(-80);
    }

    let empty = String::from("");

    let remaining_hidden_files = list_dirs_rec(path, &empty, all, recursive);
    for file in remaining_hidden_files {
        println!("{}", file);
    }
}

fn ls(args: &[String]) {
    let (opts, args) = extract_options(args);
    let mut recursive = false;
    let mut all = false;

    for opt in opts {
        match opt.as_str() {
            "-R" | "--recursive" => recursive = true,
            "-a" | "--all" => all = true,
            "-l" => todo!(),
            _ => {
                eprintln!("Invalid command");
                std::process::exit(-80);
            }
        }
    }

    // ls with no dirs lists current directory.
    if args.is_empty() {
        list_file(&String::from("."), all, recursive);
    }

    for arg in args {
        list_file(arg, all, recursive);
    }
}

fn copy_dir(src_root: &String, dest_root: &String, dir: &String) {
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

fn cp(args: &[String]) {
    let (opts, args) = extract_options(args);
    let mut recursive = false;

    for opt in opts {
        match opt.as_str() {
            "-R" | "-r" | "--recursive" => recursive = true,
            _ => {
                eprintln!("Invalid command");
                std::process::exit(-90);
            }
        }
    }

    let [src, dest] = args.as_slice() else {
        eprintln!("Usage: cp [OPTION]... SOURCE DEST");
        std::process::exit(-90);
    };

    let actual_dest = match std::fs::metadata(dest) {
        // If the destination exists and is a directory,
        // we copy the source *inside* it and it will be
        // named as the *basename* of the source.
        Ok(metadata) => {
            if metadata.is_dir() {
                // The basename is the last thing after a slash.
                let Some(basename) = src.rsplit("/").next() else {
                    eprintln!("cp: failed to get basename of '{}'", src);
                    std::process::exit(-90);
                };

                format!("{}/{}", dest, basename)
            } else {
                dest.to_string()
            }
        }
        // If dest doesn't exist, the destination is a file.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => dest.to_string(),
        Err(_) => {
            eprintln!("cp: failed to access '{}'", dest);
            std::process::exit(-90);
        }
    };

    if let Ok(file_metadata) = std::fs::metadata(src) {
        if file_metadata.is_dir() {
            if recursive {
                if std::fs::create_dir(&actual_dest).is_err() {
                    eprintln!("cp: failed to create directory '{}'", actual_dest);
                    std::process::exit(-90);
                }
                copy_dir(src, &actual_dest, &String::from("."));
                return;
            } else {
                eprintln!("cp: omitting directory '{}'", src);
                std::process::exit(-90);
            }
        } else {
            if std::fs::copy(src, &actual_dest).is_err() {
                eprintln!("cp: failed to move {} to {}", src, actual_dest);
                std::process::exit(-90);
            }
        }
    } else {
        eprintln!("cp: failed to access '{}'", src);
        std::process::exit(-90);
    }
}

fn touch(args: &[String]) {
    todo!("touch")
}

fn convert_mode(mode: u32, mode_str: &String) -> u32 {
    let mut user_mask = 0o000;
    let mut mode_mask = 0o000;
    let mut add_perms = true;

    for c in mode_str.chars() {
        match c {
            'u' => user_mask |= 0o700,
            'g' => user_mask |= 0o070,
            'o' => user_mask |= 0o007,
            'a' => user_mask |= 0o777,
            '+' => {}
            '-' => add_perms = false,
            'r' => mode_mask |= 0o444,
            'w' => mode_mask |= 0o222,
            'x' => mode_mask |= 0o111,
            _ => {
                eprintln!("chmod: invalid mode '{}'", mode_str);
                std::process::exit(-25);
            }
        }
    }

    let new_mask = user_mask & mode_mask;
    if add_perms {
        mode | new_mask
    } else {
        mode & !new_mask
    }
}

fn chmod(args: &[String]) {
    let [mode, path] = args else {
        eprintln!("Usage: chmod MODE FILE");
        std::process::exit(-25);
    };

    // Try to parse mode as an octal number. If this fails,
    // parse as "symbolic mode" (u+rwx).
    let new_mode = if let Ok(mode) = u32::from_str_radix(mode, 8) {
        mode
    } else {
        if let Ok(metadata) = std::fs::metadata(path) {
            convert_mode(metadata.permissions().mode(), mode)
        } else {
            eprintln!("chmod: failed to access '{}'", path);
            std::process::exit(-25);
        }
    };

    let new_perm = std::fs::Permissions::from_mode(new_mode);
    if std::fs::set_permissions(path, new_perm).is_err() {
        eprintln!("chmod: failed to set permissions for '{}'", path);
        std::process::exit(-25);
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    // rustybox_exec always exists.
    let (rustybox_exec, rustybox_command) = args.split_first().unwrap();

    if let Some((command, args)) = rustybox_command.split_first() {
        match command.as_str() {
            "pwd" => pwd(),
            "echo" => echo(args),
            "cat" => cat(args),
            "mkdir" => mkdir(args),
            "mv" => mv(args),
            "ln" => ln(args),
            "rmdir" => rmdir(args),
            "rm" => rm(args),
            "ls" => ls(args),
            "cp" => cp(args),
            "touch" => touch(args),
            "chmod" => chmod(args),
            _ => eprintln!("Invalid command"),
        }
    } else {
        eprintln!("Usage: {} COMMAND [ARGS]...", rustybox_exec);
    }
}
