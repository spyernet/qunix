// rm - Remove file

pub fn run(args: &[&str]) {
    if args.is_empty() {
        crate::serial_println!("Usage: rm <file>");
        return;
    }
    
    let mut vfs = crate::fs::vfs::VFS.lock();
    
    for filename in args {
        match vfs.remove_file(filename) {
            Ok(_) => {
                crate::serial_println!("Removed: {}", filename);
            }
            Err(e) => {
                crate::serial_println!("rm: error removing '{}': {:?}", filename, e);
            }
        }
    }
}
