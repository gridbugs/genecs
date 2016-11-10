extern crate tomson;
extern crate handlebars;

use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::fs::File;

use tomson::Toml;
use handlebars::Handlebars;

const TEMPLATE: &'static str = r#"

use std::collections::{HashMap, HashSet};

{{#each imports}}
use {{ this }};
{{/each}}

pub type EntityId = u64;

pub struct ComponentEntityTable {
{{#each components}}
    {{@key}}: {{#if type}} HashMap<EntityId, {{type}}> {{else}} HashSet<EntityId> {{/if}},
{{/each}}
}

"#;

fn generate_code(mut toml: String) -> String {
    // turn the toml string into json for compatibility with handlebars
    let json = Toml::as_json(&mut toml).unwrap();

    let mut handlebars = Handlebars::new();

    // prevent xml escaping
    handlebars.register_escape_fn(|input| input.to_string());
    handlebars.template_render(TEMPLATE, &json).unwrap()
}

fn read_file_to_string<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();

    string
}

pub fn generate_ecs<P: AsRef<Path>, Q: AsRef<Path>>(in_path: P, out_path: Q) {

    let string = read_file_to_string(in_path);

    let output_string = generate_code(string);

    let mut outfile = File::create(out_path).unwrap();
    write!(outfile, "{}", output_string).unwrap();
}
