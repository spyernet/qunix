// ls - List directory contents

pub fn run(args: &[&str]) {
    let dir = if args.is_empty() { "/" } else { args[0] };
    crate::println!("Listing directory: {}", dir);
    crate::println!("(filesystem read not fully implemented)");
}
