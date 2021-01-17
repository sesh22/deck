use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use structopt::StructOpt;

use crate::error::Error;

mod error;
mod html;
mod server;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(long = "verbose", short = "v")]
    verbose: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Convert a markdown file containing the slides markup to a self-contained
    /// HTML file
    #[structopt(name = "build")]
    Build {
        /// Set the title of the webpage
        #[structopt(long = "title")]
        title: Option<String>,
        /// Set the theme used to highlight text within code blocks
        #[structopt(long = "theme")]
        theme: Option<String>,
        /// Add a directory to the paths searched for syntect themes (.tmTheme files)
        #[structopt(long = "theme-dir")]
        theme_dirs: Vec<PathBuf>,
        /// Add custom css from the given file
        #[structopt(long = "css")]
        css: Option<PathBuf>,
        /// Add custom javascript from the given file
        #[structopt(long = "js")]
        js: Option<PathBuf>,
        /// Output HTML file - when this is missing output HTML will be written to standard output
        #[structopt(long = "out", short = "o", parse(from_os_str))]
        out: Option<PathBuf>,
        /// Markdown file containing the slides markup, if this is missing it will be read from standard input
        #[structopt(parse(from_os_str))]
        markdown: Option<PathBuf>,
    },
    /// Serve a local markdown files containing the slides markup
    #[structopt(name = "serve")]
    Serve {
        /// Change the port used by the server
        #[structopt(long = "port", short = "p", default_value = "8000")]
        port: u16,
        /// Markdown file containing the slides markup
        #[structopt(parse(from_os_str))]
        input: PathBuf,
        /// Whether the input file, the custom css file or the custom js file should be watched for
        /// change
        #[structopt(long = "watch", short = "w")]
        watch: bool,
        /// Set the theme used to highlight text within the code blocks
        #[structopt(long = "theme")]
        theme: Option<String>,
        /// Add a directory to the paths searched for syntect themes (.tmTheme files)
        #[structopt(long = "theme-dir")]
        theme_dirs: Vec<PathBuf>,
        /// Add custom css from the given file
        #[structopt(long = "css")]
        css: Option<PathBuf>,
        /// Add custom js from the given file
        #[structopt(long = "js")]
        js: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::from_args();

    pretty_env_logger::formatted_builder()
        .filter_module(
            "deck",
            if cli.verbose {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            },
        )
        .init();

    match cli.cmd {
        Command::Build {
            theme,
            title,
            css,
            js,
            theme_dirs,
            out,
            markdown,
        } => {
            let mut input = String::new();
            if let Some(md_path) = markdown {
                fs::read_to_string(md_path)?;
            } else {
                // Read input from stdin
                io::stdin().read_to_string(&mut input)?;
            }

            let css = if let Some(path) = css {
                let s = fs::read_to_string(path)?;
                Some(s)
            } else {
                None
            };

            let js = if let Some(path) = js {
                let s = fs::read_to_string(path)?;
                Some(s)
            } else {
                None
            };

            // Render html to stdout
            let options = html::Options {
                title,
                theme,
                theme_dirs,
            };

            let renderer = html::Renderer::try_new(options)?;
            let html = renderer.render(input, css, js)?;
            if let Some(out_path) = out {
                fs::write(&out_path, format!("{}", html))?;
            } else {
                print!("{}", html);
            }
        }
        Command::Serve {
            port,
            input,
            watch,
            theme,
            theme_dirs,
            css,
            js,
        } => {
            let config = server::Config {
                port,
                watch,
                input,
                theme,
                theme_dirs,
                css,
                js,
            };
            server::start(config).await?;
        }
    }
    Ok(())
}
