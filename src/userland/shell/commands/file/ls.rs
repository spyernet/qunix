// ls - List directory contents

pub fn run(args: &[&str]) {
    let dir = if args.is_empty() { "." } else { args[0] };
    
    let vfs = crate::fs::vfs::VFS.lock();
    match vfs.lookup_path(dir) {
        Ok(node) => {
            if node.is_dir() {
                match node.readdir() {
                    Ok(entries) => {
                        for entry in entries {
                            if entry.name == "." || entry.name == ".." {
                                continue;
                            }
                            crate::serial_println!("{}", entry.name);
                        }
                    }
                    Err(e) => {
                        crate::serial_println!("Error reading directory: {:?}", e);
                    }
                }
            } else {
                crate::serial_println!("ls: cannot access '{}': Not a directory", dir);
            }
        }
        Err(e) => {
            crate::serial_println!("ls: cannot access '{}': {:?}", dir, e);
        }
    }
}
