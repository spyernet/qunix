// echo - Print text

pub fn run(args: &[&str]) {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 { 
            crate::serial_print!(" "); 
        }
        crate::serial_print!("{}", arg);
    }
    crate::serial_println!();
}
