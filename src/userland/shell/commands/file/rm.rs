// rm - Remove file

pub fn run(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rm <file>");
        return;
    }
    
    for filename in args {
        crate::println!("(rm would delete: {})", filename);
    }
}
