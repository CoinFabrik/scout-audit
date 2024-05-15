extern crate tera;

use tera::{Context, Result, Tera};

const TEMPLATE_STR: &str = include_str!("./template.md");

#[derive(Debug)]
pub struct MdEngine {
    tera: Tera,
}

impl MdEngine {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();
        tera.add_raw_template("base_template", TEMPLATE_STR)?;
        Ok(MdEngine { tera })
    }

    pub fn render_template(&self, contexts: Vec<Context>) -> Result<String> {
        let context = Self::merge_contexts(contexts);
        self.tera.render("base_template", &context)
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
