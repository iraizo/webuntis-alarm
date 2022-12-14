use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version)]
pub struct Configuration {
    #[arg(long, env, required = true)]
    pub url: String,

    #[arg(long, env, required = true)]
    pub username: String,

    #[arg(long, env, required = true)]
    pub password: String,
}

pub fn parse() -> Configuration {
    Configuration::parse()
}
