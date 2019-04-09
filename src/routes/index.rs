use rocket_contrib::templates::Template;
use std::collections::HashMap;

#[get("/")]
pub fn index() -> Template {
    let mut context = HashMap::new();
    context.insert("dump", true);
    Template::render("index", &context)
}
