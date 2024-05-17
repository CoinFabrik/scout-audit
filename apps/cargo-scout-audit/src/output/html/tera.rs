extern crate tera;

use tera::{Context, Result, Tera};

const TEMPLATE_BASE: &str = include_str!("./templates/base.html");
const TEMPLATE_CATEGORIES: &str = include_str!("./templates/categories.html");
const TEMPLATE_FINDINGS: &str = include_str!("./templates/findings_list.html");
const TEMPLATE_MODAL: &str = include_str!("./templates/modal.html");
const TEMPLATE_VULNERABILITY_DETAILS: &str = include_str!("./templates/vulnerability_details.html");
const JS_MODAL_HANDLER: &str = include_str!("./build/modal-handler.js");
const JS_CATEGORY_FILTER: &str = include_str!("./build/category-filter.js");
const JS_VULNERABILITY_DETAILS: &str = include_str!("./build/vulnerability-details-display.js");
const JS_VULNERABILITY_EXPANSION: &str = include_str!("./build/vulnerability-expansion.js");
const STYLES: &str = include_str!("./build/styles.css");

pub struct HtmlEngine {
    tera: Tera,
}

impl HtmlEngine {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("base.html", TEMPLATE_BASE),
            ("modal.html", TEMPLATE_MODAL),
            ("categories.html", TEMPLATE_CATEGORIES),
            ("findings_list.html", TEMPLATE_FINDINGS),
            ("vulnerability_details.html", TEMPLATE_VULNERABILITY_DETAILS),
            ("modal.js", JS_MODAL_HANDLER),
            ("category-filter.js", JS_CATEGORY_FILTER),
            ("vulnerability-expansion.js", JS_VULNERABILITY_EXPANSION),
            ("vulnerability-details-display.js", JS_VULNERABILITY_DETAILS),
            ("styles.css", STYLES),
        ])?;
        Ok(HtmlEngine { tera })
    }

    pub fn render_template(&self, contexts: Vec<Context>) -> Result<String> {
        let context = Self::merge_contexts(contexts);
        self.tera.render("base.html", &context)
    }

    pub fn create_context<T: serde::Serialize>(&self, key: &str, context: T) -> Context {
        let mut ctx = Context::new();
        ctx.insert(key, &context);
        ctx
    }

    fn merge_contexts(contexts: Vec<Context>) -> Context {
        contexts.into_iter().fold(Context::new(), |mut acc, c| {
            acc.extend(c);
            acc
        })
    }
}
