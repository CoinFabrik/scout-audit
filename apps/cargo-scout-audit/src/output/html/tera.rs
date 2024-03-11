extern crate tera;

use tera::{Context, Result, Tera};

lazy_static! {
    pub static ref TEMPLATES: Option<Tera> = Tera::new("src/templates/**/*").ok();
}

pub fn render_template(template_name: &str, contexts: Vec<Context>) -> Result<String> {
    if let Some(tera) = &*TEMPLATES {
        let context = merge_contexts(contexts);
        tera.render(template_name, &context)
    } else {
        Err("Template engine was not initialized".into())
    }
}

pub fn create_context(key: &str, context: impl serde::Serialize) -> Context {
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
