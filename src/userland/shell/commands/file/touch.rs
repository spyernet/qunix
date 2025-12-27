// touch - Create empty file

pub fn run(args: &[&str]) {
    if args.is_empty() {
        crate::serial_println!("Usage: touch <file>");
        return;
    }
    
    let mut vfs = crate::fs::vfs::VFS.lock();
    
    for filename in args {
        match vfs.create_file(filename, crate::fs::FileMode::new(0o644)) {
            Ok(_) => {
                crate::serial_println!("Created: {}", filename);
            }
            Err(e) => {
                crate::serial_println!("touch: error creating '{}': {:?}", filename, e);
            }
        }
    }
}
