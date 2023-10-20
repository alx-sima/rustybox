fn pwd() {
    if let Ok(cwd) = std::env::current_dir() {
        println!("{}", cwd.display());
    } else {
        eprintln!("rustybox: pwd: failed to get current directory");
    }
}

fn cat(args: &[String]) {
    for arg in args {
        if let Ok(contents) = std::fs::read_to_string(arg) {
            print!("{}", contents);
        } else {
            std::process::exit(-20);
        }
    }
}

fn mkdir(args: &[String]) {
    for arg in args {
        if !std::fs::create_dir(arg).is_ok() {
            std::process::exit(-30);
        }
    }
}

fn mv(args: &[String]) {
    let [src, dst] = args else {
        std::process::exit(-40);
    };

    if !std::fs::rename(src, dst).is_ok() {
        std::process::exit(-40);
    }
}

fn rmdir(args: &[String]) {
    for arg in args {
        if !std::fs::remove_dir(arg).is_ok() {
            std::process::exit(-60);
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    // rustybox_exec always exists.
    let (rustybox_exec, rustybox_command) = args.split_first().unwrap();

    if let Some((command, args)) = rustybox_command.split_first() {
        match command.as_str() {
            "pwd" => pwd(),
            "cat" => cat(args),
            "mkdir" => mkdir(args),
            "mv" => mv(args),
            "rmdir" => rmdir(args),
            _ => eprintln!("rustybox: {}: unknown command", command),
        }
    } else {
        eprintln!("Usage: {} COMMAND [ARGS]...", rustybox_exec);
    }
}
