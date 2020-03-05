use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Query {
    pub id_path: Option<String>,
    pub end: Option<String>,
    pub limit: Option<usize>,
    pub until_key: Option<String>,
    pub skip: Option<usize>,
    pub direction_reverse: Option<bool>,
    pub channel: Option<String>,
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}\nend: {}\nlimit: {}\nuntil_key: {}\nskip: {}\ndirection_forward: {}\nchannel: {}\n",
               self.id_path.clone().unwrap_or_default(),
               self.end.clone().unwrap_or_default(),
               self.limit.clone().unwrap_or_default(),
               self.until_key.clone().unwrap_or_default(),
               self.skip.clone().unwrap_or_default(),
               self.direction_reverse.clone().unwrap_or_default(),
               self.channel.clone().unwrap_or_default())
    }
}