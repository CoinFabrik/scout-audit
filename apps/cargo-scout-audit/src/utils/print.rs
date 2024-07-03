use colored::Colorize;

pub fn print_warning(message: &str) {
    println!("{} {}", "[WARNING]".yellow(), message);
}

pub fn print_error(message: &str) {
    println!("{} {}", "[ERROR]".red(), message);
}
