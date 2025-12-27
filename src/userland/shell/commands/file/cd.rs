// cd - Change directory

pub fn run(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: cd <directory>");
        return;
    }
    
    let newdir = args[0];
    crate::println!("(cd would change to: {})", newdir);
}
