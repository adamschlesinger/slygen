//! todo

use std::error::Error;
use std::{env, fs};
use std::path::Path;
use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use reqwest::Url;
use syn::{Item, ItemFn, ItemMod};

pub fn cwd() -> Result<String, Box<dyn Error>> {
    let path = env::current_dir()?
        .display()
        .to_string();

    Ok(path)
}

pub async fn load_content(path: &str) -> Result<String, Box<dyn Error>> {
    let content = match Url::parse(path) {
        Err(_) => fs::read_to_string(path)?,
        Ok(url) => reqwest::get(url).await?.text().await?
    };

    Ok(content)
}

pub fn write_code<P: AsRef<Path>>(file_path: P, code: &TokenStream) -> Result<(), Box<dyn Error>> {
    let code_str: &str = &*code.to_string();
    let code_file = syn::parse_file(code_str)?;
    let code_formatted = prettyplease::unparse(&code_file);
    fs::write(file_path, code_formatted)?;
    Ok(())
}

pub fn generate_main(out_path:&str) -> Result<(), Box<dyn Error>> {
    let main_code = quote! {
        use cli::Commands;
        use std::error::Error;
        use clap::Parser;
        use serde::{Deserialize, Serialize};

        mod cli;

        #[derive(Parser, Debug, Serialize, Deserialize)]
        #[command(author, version, about, long_about = None)]
        struct Cli {
            #[command(subcommand)]
            cmd: Commands
        }

        #[tokio::main]
        async fn main() -> Result<(), Box<dyn Error>> {
            let cli = Cli::parse();
            Ok(())
        }
    };

    write_code(format!("{out_path}/src/main.rs"), &main_code)
}

pub fn generate_cli(out_path: &str) -> Result<(), Box<dyn Error>> {
    let oas_dir = format!("{out_path}/openapi");
    let apis_dir = format!("{oas_dir}/src/apis");

    let src_out = format!("{out_path}/src");
    let cli_out = format!("{src_out}/cli");
    fs::create_dir_all(&cli_out)?;

    let oa_mod_content = fs::read_to_string(format!("{apis_dir}/mod.rs"))?;
    let oa_mod_file = syn::parse_file(&oa_mod_content)?;

    let apis: Vec<&ItemMod> = oa_mod_file
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Mod(item_mod) if item_mod.ident.to_string().ends_with("_api") => Some(item_mod),
            _ => None,
        }).collect();

    let mut mod_decls: Vec<TokenStream> = vec![];
    let mut use_decls: Vec<TokenStream> = vec![];
    let mut api_commands: Vec<TokenStream> = vec![];

    for api in apis {
        let api_name = api.ident.to_string();
        let api_name_stripped = api_name.strip_suffix("_api").unwrap().to_string();
        let api_name_stripped_ident = format_ident!("{}", api_name_stripped);
        let api_name_formatted_ident = format_ident!("{}", api_name_stripped.to_case(Case::Pascal));
        let enum_ident = format_ident!("{}Api", api_name_formatted_ident);

        let api_content = fs::read_to_string(format!("{apis_dir}/{api_name}.rs"))?;
        let api_file = syn::parse_file(&api_content)?;

        let fns: Vec<&ItemFn> = api_file.items.iter().filter_map(|item| match item {
            Item::Fn(item_fn) => Some(item_fn),
            _ => None
        }).collect();

        let fn_name_idents: Vec<Ident> = fns
            .iter()
            .map(|api_fn| {
                let fn_name = api_fn.sig.ident.to_string();
                let fn_name_formatted = fn_name.to_case(Case::Pascal);
                format_ident!("{}", fn_name_formatted)
            }).collect();

        mod_decls.push(quote! {
            pub mod #api_name_stripped_ident;
        });

        use_decls.push(quote! {
            use crate::cli::#api_name_stripped_ident::{#enum_ident,#(#fn_name_idents),*};
        });

        api_commands.push(quote! {
            #[command(subcommand)]
            #api_name_formatted_ident(#enum_ident),
        });

        let api_code = quote! {
            use std::error::Error;
            use clap::{Parser, Subcommand};
            use enum_dispatch::enum_dispatch;
            use serde::{Deserialize, Serialize};
            use crate::Cli;
            use crate::cli::CliCommand;

            #[enum_dispatch(CliCommand)]
            #[derive(Subcommand, Debug, Deserialize, Serialize)]
            pub enum #enum_ident {
                #(#fn_name_idents,)*
            }

            #(#[derive(Parser, Debug, Deserialize, Serialize)] pub struct #fn_name_idents;)*

            #(impl CliCommand for #fn_name_idents {
                async fn exec(&self, _cli: Cli) -> Result<(), Box<dyn Error>> {
                    todo!()
                }
            })*
        };

        write_code(format!("{cli_out}/{api_name_stripped}.rs"), &api_code)?;
    }

    let mod_code = quote! {
        use std::error::Error;
        use clap::{Parser, Subcommand};
        use enum_dispatch::enum_dispatch;
        use serde::{Deserialize, Serialize};
        use openapi;
        use crate::Cli;
        #(#use_decls)*
        #(#mod_decls)*

        #[enum_dispatch]
        pub trait CliCommand {
            async fn exec(&self, _cli: Cli) -> Result<(), Box<dyn Error>>;
        }

        #[derive(Parser, Debug, Deserialize, Serialize)]
        pub enum Commands {
            #(#api_commands)*
        }
    };

    write_code(format!("{cli_out}/mod.rs"), &mod_code)
}