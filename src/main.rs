mod cli;
mod closures;
mod facts;
mod render;
mod source;
mod text;
mod util;

fn main() -> anyhow::Result<()> {
    cli::run()
}
