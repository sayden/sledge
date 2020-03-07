use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Query {
    pub field_path: Option<String>,
    pub end: Option<String>,
    pub limit: Option<usize>,
    pub until_key: Option<String>,
    pub skip: Option<usize>,
    pub direction_reverse: Option<bool>,
    pub channel: Option<String>,
    pub include_id: Option<bool>,
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}\nend: {}\nlimit: {}\nuntil_key: {}\nskip: {}\ndirection_forward: {}\nchannel: {}\ninclude id: {}\n",
               self.field_path.clone().unwrap_or_default(),
               self.end.clone().unwrap_or_default(),
               self.limit.clone().unwrap_or_default(),
               self.until_key.clone().unwrap_or_default(),
               self.skip.clone().unwrap_or_default(),
               self.direction_reverse.clone().unwrap_or_default(),
               self.channel.clone().unwrap_or_default(),
               self.include_id.clone().unwrap_or_default())
    }
}