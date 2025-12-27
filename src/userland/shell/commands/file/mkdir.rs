// mkdir - Create directory

pub fn run(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: mkdir <directory>");
        return;
    }
    
    for dirname in args {
        crate::println!("(mkdir would create: {})", dirname);
    }
}
