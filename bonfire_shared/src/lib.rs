use serde::{Serialize, Deserialize};

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
  pub user: String,
  pub content: String,
}
