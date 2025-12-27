// cat - Display file contents

pub fn run(args: &[&str]) {
    if args.is_empty() {
        crate::serial_println!("Usage: cat <file>");
        return;
    }
    
    let vfs = crate::fs::vfs::VFS.lock();
    
    for filename in args {
        match vfs.lookup_path(filename) {
            Ok(node) => {
                if node.is_file() {
                    let mut buf = [0u8; 4096];
                    match node.read(0, &mut buf) {
                        Ok(len) => {
                            if let Ok(s) = core::str::from_utf8(&buf[..len]) {
                                crate::serial_print!("{}", s);
                            } else {
                                crate::serial_println!("(binary data)");
                            }
                        }
                        Err(e) => {
                            crate::serial_println!("cat: error reading '{}': {:?}", filename, e);
                        }
                    }
                } else {
                    crate::serial_println!("cat: '{}': Is a directory", filename);
                }
            }
            Err(e) => {
                crate::serial_println!("cat: cannot open '{}': {:?}", filename, e);
            }
        }
    }
}
