use crate::error::Result;
use std::{
    collections::HashMap,
    fs::File,
    io::Read as _,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct List {
    radio_stations: HashMap<String, String>,
}

pub struct Station {
    pub name: String,
    pub url: String,
}

impl Station {
    fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self::_new(name.into(), url.into())
    }

    fn _new(name: String, url: String) -> Self {
        Self {
            name,
            url,
        }
    }
}

pub struct RadioList {
    pub messages: Vec<String>,
    pub stations: HashMap<String, Station>,
}

impl RadioList {
    pub fn new() -> Result<Self> {
        let contents = {
            let mut file = File::open("./radios/list.toml")?;

            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;

            contents
        };

        let list = toml::from_slice::<List>(&contents)?;

        let mut stations = HashMap::with_capacity(list.radio_stations.len());

        for (name, url) in list.radio_stations {
            let slug = name.to_lowercase();
            let station = Station::new(name, url);

            stations.insert(slug, station);
        }

        Ok(Self {
            messages: vec![],
            stations,
        })
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<&Station> {
        self._get(name.as_ref())
    }

    fn _get(&self, name: &str) -> Option<&Station> {
        self.stations.get(&name.to_lowercase())
    }
}
