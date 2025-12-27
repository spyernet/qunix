// cat - Display file contents

pub fn run(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: cat <file>");
        return;
    }
    
    for filename in args {
        crate::println!("(cat would read: {})", filename);
    }
}
