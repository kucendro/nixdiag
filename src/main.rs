mod cli;
mod closures;
mod conf;
mod facts;
mod human;
mod render;
mod source;
mod text;
mod topology;
mod util;

fn main() -> anyhow::Result<()> {
    cli::run()
}
