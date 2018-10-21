use crate::error::Result;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OidId {
    #[serde(rename = "$oid")]
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Id {
    Oid(OidId),
    Raw(String),
}

impl Id {
    pub fn raw(&self) -> String {
        match self {
            Id::Oid(oid) => oid.id.clone(),
            Id::Raw(id) => id.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SongInfo {
    #[serde(rename = "_id")]
    pub id: Id,
    pub service: String,
    pub identifier: String,
    pub length: f64,
    pub title: String,
}

impl SongInfo {
    pub fn id(&self) -> String {
        self.id.raw()
    }
}

#[derive(Clone, Debug)]
pub struct PlaylistItem {
    pub info: SongInfo,
    pub track: String,
}

#[derive(Clone, Debug)]
pub struct Library {
    pub name: String,
    pub items: Vec<PlaylistItem>,
}

#[derive(Clone, Debug)]
pub struct DiscordFm {
    pub list: String,
    pub libraries: HashMap<String, Library>,
}

impl DiscordFm {
    pub fn new() -> Result<Self> {
        let mut libraries = HashMap::new();

        trace!("Walking over D.FM lists");

        for entry in fs::read_dir("./discord-fm/json")? {
            trace!("Parsing entry");
            let entry = entry?;
            trace!("Entry: {:?}", entry);
            let path = entry.path();
            trace!("Entry path: {:?}", path);

            if path.is_dir() {
                warn!("Directory in discord FM: {:?}", entry);

                continue;
            }

            trace!("Opening playlist JSON file");
            let json_file = File::open(path)?;
            trace!("Opened playlist JSON file");
            trace!("Deserializing playlist JSON");
            let songs: Vec<SongInfo> = serde_json::from_reader(&json_file)?;
            trace!("Deserialized playlist JSON");

            let (name, legible) = match entry.file_name().into_string() {
                Ok(name) => {
                    let mut legible = name.clone();
                    legible.truncate(name.rfind('.')?);
                    legible = legible.replace('_', " ");

                    (name, legible)
                },
                Err(original) => {
                    warn!("{:?} cannot be String", original);

                    continue;
                },
            };

            let mut blob_path = Path::new("./discord-fm/blob").to_owned();
            blob_path.push(name);

            trace!("Opening playlist blob file");
            let blob_file = File::open(blob_path)?;
            trace!("Opened playlist blob file");
            trace!("Deserializing playlist blobs");
            let blobs: Vec<String> = serde_json::from_reader(&blob_file)?;
            trace!("Deserialized playlist blobs");

            let blob_len = blobs.len();
            trace!("JSONs: {}; blobs: {}", songs.len(), blob_len);
            let mut tracks = blobs.into_iter();

            let mut items = Vec::with_capacity(blob_len);

            trace!("Iterating over playlist songs for {}", legible);

            for song in songs.into_iter() {
                trace!("Getting next track");
                let track = match tracks.next() {
                    Some(track) => track,
                    None => {
                        warn!("Playlist '{}' has an unequal number of items", legible,);

                        break;
                    },
                };
                trace!("Got next track");

                items.push(PlaylistItem {
                    info: song,
                    track,
                });
            }

            trace!("Iterated over playlist songs");

            libraries.insert(
                legible.to_lowercase(),
                Library {
                    name: legible,
                    items,
                },
            );
        }

        // Calculate the list of names once to keep a cache and avoid
        // re-calculating it multiple times in the future.
        let mut names = libraries
            .values()
            .map(|library| library.name.clone())
            .collect::<Vec<_>>();
        names.sort();
        let list = names.join(", ");

        Ok(Self {
            list,
            libraries,
        })
    }
}
