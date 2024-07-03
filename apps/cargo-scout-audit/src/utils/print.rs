use colored::Colorize;

pub fn pretty_warning(message: &str) -> String {
    format!("{} {}", "[WARNING]".yellow(), message)
}

pub fn pretty_error(message: &str) -> String {
    format!("{} {}", "[ERROR]".red(), message)
}
