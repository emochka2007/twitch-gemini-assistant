use crate::pg::pg::PgConnect;

#[derive(Debug)]
pub struct WebsiteConfig {
    pub(crate) sound_name: String,
    pub(crate) theme: String,
    pub(crate) alert: String,
    pub(crate) prompt: String,
}

impl WebsiteConfig {
    pub async fn init_config() -> anyhow::Result<()> {
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        let query =
            "INSERT INTO website_config (sound_name, theme, alert, prompt) VALUES ($1, $2, $3, $4)";
        let rows = client
            .query(query, &[&"samsung", &"default", &"none", &"none"])
            .await?;
        Ok(())
    }
    pub async fn update_config(config: WebsiteConfig) -> anyhow::Result<()> {
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        let query = "UPDATE website_config SET sound_name = $1, theme = $2, alert = $3";
        let rows = client
            .query(query, &[&config.sound_name, &config.theme, &config.alert])
            .await?;
        Ok(())
    }

    pub async fn update_prompt(prompt: &str) -> anyhow::Result<()> {
        let pool = PgConnect::create_pool_from_env()?;
        let client = pool.get().await?;
        let query = "UPDATE website_config SET prompt = $1";
        let rows = client.query(query, &[&prompt]).await?;
        Ok(())
    }

    pub async fn get_config() -> Self {
        let pool = PgConnect::create_pool_from_env().unwrap();
        let client = pool.get().await.unwrap();
        let query = "SELECT * from website_config LIMIT 1";
        match client.query_one(query, &[]).await {
            Ok(row) => Self {
                sound_name: row.try_get("sound_name").unwrap(),
                theme: row.try_get("theme").unwrap(),
                alert: row.try_get("alert").unwrap(),
                prompt: row.try_get("prompt").unwrap(),
            },
            Err(e) => {
                Self::init_config().await.unwrap();
                Box::pin(Self::get_config()).await
            }
        }
    }
}
