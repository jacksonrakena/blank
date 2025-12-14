#[derive(Debug)]
pub struct Rule {
    pub name: String,
    pub url: String,
    pub opts: RuleOptions,
}

#[derive(Debug)]
pub enum RedirectionMode {
    Permanent,
    Temporary,
}

#[derive(Debug)]
pub struct RuleOptions {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub redirect: RedirectionMode,
}

impl Default for RuleOptions {
    fn default() -> Self {
        RuleOptions {
            description: None,
            tags: vec![],
            redirect: RedirectionMode::Temporary,
        }
    }
}