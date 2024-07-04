use colored::Colorize;

pub fn print_warning(message: &str) {
    println!("{}", pretty_warning(message));
}

pub fn print_error(message: &str) {
    println!("{}", pretty_error(message));
}

pub fn pretty_warning(message: &str) -> String {
    format!("{} {}", "[WARNING]".yellow(), message)
}

pub fn pretty_error(message: &str) -> String {
    format!("{} {}", "[ERROR]".red(), message)
}
