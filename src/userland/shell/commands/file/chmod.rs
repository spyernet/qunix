// chmod - Change file permissions

pub fn run(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: chmod <mode> <file>");
        return;
    }
    
    crate::println!("(chmod {} {})", args[0], args[1]);
}
