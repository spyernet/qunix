// chmod - Change file permissions

pub fn run(args: &[&str]) {
    if args.len() < 2 {
        crate::serial_println!("Usage: chmod <mode> <file>");
        return;
    }
    
    let mode_str = args[0];
    let path = args[1];
    
    let mode = if mode_str.starts_with('0') || mode_str.starts_with('1') || 
                  mode_str.starts_with('2') || mode_str.starts_with('3') ||
                  mode_str.starts_with('4') || mode_str.starts_with('5') ||
                  mode_str.starts_with('6') || mode_str.starts_with('7') {
        u16::from_str_radix(mode_str, 8).unwrap_or(0o644)
    } else {
        0o644
    };
    
    let mut vfs = crate::fs::vfs::VFS.lock();
    match vfs.chmod(path, mode) {
        Ok(_) => {
            crate::serial_println!("Changed permissions of '{}' to {:o}", path, mode);
        }
        Err(e) => {
            crate::serial_println!("chmod: error changing '{}': {:?}", path, e);
        }
    }
}
