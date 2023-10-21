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
                eprint!("Invalid command");
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
        eprint!("Invalid command");
        std::process::exit(-10);
    }
}

fn cat(args: &[String]) {
    for arg in args {
        if let Ok(contents) = std::fs::read_to_string(arg) {
            print!("{}", contents);
        } else {
            eprint!("cat: {}: No such file or directory", arg);
            std::process::exit(-20);
        }
    }
}

fn mkdir(args: &[String]) {
    for arg in args {
        if std::fs::create_dir(arg).is_err() {
            eprint!("mkdir: cannot create directory '{}'", arg);
            std::process::exit(-30);
        }
    }
}

fn mv(args: &[String]) {
    let [src, dst] = args else {
        eprint!("Usage: mv SOURCE DEST");
        std::process::exit(-40);
    };

    if std::fs::rename(src, dst).is_err() {
        eprint!("mv: cannot move '{}' to '{}'", src, dst);
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
                eprint!("Invalid command");
                std::process::exit(-50);
            }
        }
    }

    let [src, dst] = args.as_slice() else {
        eprint!("Usage: ln SOURCE DEST");
        std::process::exit(-50);
    };

    let ret_status = if symbolic {
        std::os::unix::fs::symlink(src, dst)
    } else {
        std::fs::hard_link(src, dst)
    };

    if ret_status.is_err() {
        eprint!("ln: cannot link '{}' to '{}'", src, dst);
        std::process::exit(-50);
    }
}

fn rmdir(args: &[String]) {
    for arg in args {
        if std::fs::remove_dir(arg).is_err() {
            eprint!("rmdir: failed to remove '{}'", arg);
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
                eprint!("Invalid command");
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
            eprint!("rm: failed to remove '{}'", arg);
            std::process::exit(-70);
        }
    }
}

fn ls(args: &[String]) {
    todo!("ls")
}

fn cp(args: &[String]) {
    todo!("cp")
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
                eprint!("chmod: invalid mode '{}'", mode_str);
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
        eprint!("Usage: chmod MODE FILE");
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
            eprint!("chmod: failed to access '{}'", path);
            std::process::exit(-25);
        }
    };

    let new_perm = std::fs::Permissions::from_mode(new_mode);
    if std::fs::set_permissions(path, new_perm).is_err() {
        eprint!("chmod: failed to set permissions for '{}'", path);
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
