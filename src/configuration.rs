use eyre::{Result, WrapErr};

const CONFIGURATION_FILE: &str = "configuration.yaml";

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

pub fn get_configuration() -> Result<Settings> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            CONFIGURATION_FILE,
            config::FileFormat::Yaml,
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()
        .context("Building settings")?;
    settings
        .try_deserialize::<Settings>()
        .context("Deserializing Settings")
}
