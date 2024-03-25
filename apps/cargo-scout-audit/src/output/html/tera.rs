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
impl Default for HtmlEngine {
    fn default() -> Self {
        let mut engine = HtmlEngine {
            tera: Tera::default(),
        };
        let _ = engine.load_templates();
        engine
    }
}
impl HtmlEngine {
    pub fn new() -> HtmlEngine {
        HtmlEngine::default()
    }

    fn load_templates(&mut self) -> Result<()> {
        self.tera.add_raw_templates(vec![
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
        ])
    }

    pub fn render_template(&self, template_name: &str, contexts: Vec<Context>) -> Result<String> {
        let context = Self::merge_contexts(contexts);
        self.tera.render(template_name, &context)
    }

    pub fn create_context(&self, key: &str, context: impl serde::Serialize) -> Context {
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
