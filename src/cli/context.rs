use crate::config::PlayhouseSettings;

pub struct Context<'a> {
    pub workspace: &'a str,
    pub settings: &'a PlayhouseSettings,
    pub json: bool,
}
