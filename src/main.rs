use std::os::unix::prelude::PermissionsExt;

fn pwd() {
    if let Ok(cwd) = std::env::current_dir() {
        println!("{}", cwd.display());
    } else {
        eprintln!("pwd: failed to get current directory");
    }
}

fn echo(args: &[String]) {
    todo!("echo")
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
    todo!("ln")
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
    todo!("rm")
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
