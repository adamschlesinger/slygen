//! todo

use std::error::Error;
use std::fs;

use clap::Parser;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;

use slygen::{cwd, generate_cli, generate_main, load_content};
use crate::terminal::logger;
use crate::terminal::logger::LogLevel;

mod terminal;

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Project name
    name: String,

    /// File path or URL to the OpenAPI spec
    spec: String,

    /// Path to output the project to
    #[arg(short, long)]
    output: Option<String>,

    /// Build and install to PATH (NYI)
    #[arg(long)]
    install: bool,

    /// Log debug messages
    #[arg(short, long, value_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    logger::init(cli.log_level);

    header!("slygen");

    let out_path = format!("{}/{}", cli.output.unwrap_or(cwd()?), cli.name);
    debug!("Creating output path: {out_path}");
    fs::create_dir_all(&out_path)?;

    info!("Getting spec from {}", cli.spec);
    let oas = load_content(&cli.spec).await?;

    debug!("Writing out: {out_path}");
    fs::write(format!("{out_path}/spec.json"), oas)?; // todo - yaml support

    // todo - can we avoid outsourcing to the OAG CLI?
    info!("Building spec to rust lib");
    sh!("openapi-generator generate -i {out_path}/spec.json -g rust -o {out_path}/openapi")?;

    info!("Generating CLI files");
    generate_cli(&out_path)?;
    generate_main(&out_path)?;

    info!("Generating Cargo.toml and README.md");
    let data = json!({"name": cli.name});
    let handlebars = Handlebars::new();
    let entries = fs::read_dir("res")?;

    for entry in entries {
        let template_path = entry?.path().display().to_string();
        let template_str = fs::read_to_string(&template_path)?;
        let rendered_file = handlebars.render_template(&template_str, &data)?;

        let outfile = template_path.strip_suffix(".hbs")
            .unwrap_or(&template_path)
            .strip_prefix("res/")
            .unwrap()
            .to_string();

        fs::write(format!("{out_path}/{outfile}"), rendered_file)?;
    }

    Ok(())
}
