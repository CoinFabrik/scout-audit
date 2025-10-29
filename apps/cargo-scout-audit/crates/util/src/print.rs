use anyhow::Error;
use colored::Colorize;

pub fn print_warning(message: &str) {
    println!("{}", pretty_warning(message));
}

pub fn print_error(message: &str) {
    println!("{}", pretty_error(message));
}

pub fn print_full_error(e: &Error) {
    for i in e.chain() {
        print_error(&i.to_string());
    }
}

pub fn print_info(message: &str) {
    eprintln!("{}", pretty_info(message));
}

pub fn pretty_warning(message: &str) -> String {
    format!("{} {}", "[WARNING]".yellow(), message)
}

pub fn pretty_error(message: &str) -> String {
    format!("{} {}", "[ERROR]".red(), message)
}

pub fn pretty_info(message: &str) -> String {
    format!("{} {}", "[INFO]".blue(), message)
}
