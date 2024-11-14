use crate::{
    output::table::{construct_table, prepare_tera_for_table_render_console},
    utils::detectors_info::LintStore,
    finding::Finding,
};
use std::collections::HashMap;
use tera::{Context, Tera};
use terminal_color_builder::OutputFormatter;

const CONSOLE_TEMPLATE: &str = include_str!("./template.txt");

fn get_template_path() -> (String, String) {
    (
        env!("HOME").to_string() + "/.scout-audit/templates",
        "console.txt".to_string(),
    )
}

pub(crate) fn render_report(
    findings: &[Finding],
    crates: &HashMap<String, bool>,
    detectors_info: &LintStore,
) -> Result<(), tera::Error> {
    for finding in findings.iter() {
        print!("{}", finding.rendered());
    }

    let table = construct_table(findings, crates, detectors_info).to_json_table();

    let mut tera = Tera::default();
    let mut context = Context::new();
    tera.add_raw_template(
        "base_template",
        &crate::output::utils::get_template(get_template_path, CONSOLE_TEMPLATE),
    )?;
    prepare_tera_for_table_render_console(&mut tera, &mut context, &table, "summary");

    let result = tera.render("base_template", &context)?;

    println!("{}", result);

    if crates.iter().any(|(_, success)| !success) {
        let string = OutputFormatter::new()
            .fg()
            .red()
            .text_str("This report is incomplete because some crates failed to compile. Please resolve the errors and try again.")
            .print();
        println!("{}", string);
    }

    Ok(())
}
