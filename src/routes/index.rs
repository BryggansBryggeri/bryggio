use bryggio::brewery::BrewState;
use rocket::State;
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;
use serde_json;
use std::collections::HashMap;

#[get("/")]
pub fn index(brew_state: State<BrewState>) -> Template {
    Template::render("index", &brew_state.clone())
}
