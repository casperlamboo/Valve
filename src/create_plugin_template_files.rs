use minijinja::{context, Environment};

use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = Environment::new();

    let settings_prefix = format!(
        "_plugin__{plugin_name}__{major}_{minor}_{patch}__",
        plugin_name = env!("CARGO_PKG_NAME").to_string(),
        major = env!("CARGO_PKG_VERSION_MAJOR").to_string(),
        minor = env!("CARGO_PKG_VERSION_MINOR").to_string(),
        patch = env!("CARGO_PKG_VERSION_PATCH").to_string(),
    );

    let plugin_json_jinja = fs::read_to_string("templates/plugin.json.jinja")?;
    env.add_template("plugin.json", plugin_json_jinja.as_str())?;

    let package_json_jinja = fs::read_to_string("templates/package.json.jinja")?;
    env.add_template("package.json", package_json_jinja.as_str())?;

    let constants_py_jinja = fs::read_to_string("templates/constants.py.jinja")?;
    env.add_template("constants.py", constants_py_jinja.as_str())?;

    env.get_template("plugin.json")?.render_to_write(
        context! {
            name => env!("CARGO_PKG_NAME").to_string(),
            author => env!("CARGO_PKG_AUTHORS").to_string(),
            version => env!("CARGO_PKG_VERSION").to_string(),
            description => env!("CARGO_PKG_DESCRIPTION").to_string(),
        },
        &mut fs::File::create("valve/plugin.json")?,
    )?;

    env.get_template("package.json")?.render_to_write(
        context! {
            author => env!("CARGO_PKG_AUTHORS").to_string(),
            homepage => env!("CARGO_PKG_HOMEPAGE").to_string(),
            repository => env!("CARGO_PKG_REPOSITORY").to_string(),
            name => env!("CARGO_PKG_NAME").to_string(),
            package_id => env!("CARGO_PKG_NAME").to_string(),
            version => env!("CARGO_PKG_VERSION").to_string(),
            description => env!("CARGO_PKG_DESCRIPTION").to_string(),
        },
        &mut fs::File::create("valve/package.json")?,
    )?;

    env.get_template("constants.py")?.render_to_write(
        context! {
            name => env!("CARGO_PKG_NAME").to_string(),
            version => env!("CARGO_PKG_VERSION").to_string(),
            settings_prefix => settings_prefix,
        },
        &mut fs::File::create("valve/constants.py")?,
    )?;

    Ok(())
}
