use std::{ops::Deref, sync::Arc, time::Instant};

use dashmap::DashMap;
use error::AppResult;
use futures::future::join_all;
use tokio::task::JoinHandle;
use twilight_http::Client;
use twilight_model::{
    channel::message::{
        embed::{EmbedAuthor, EmbedField, EmbedFooter},
        Embed,
    },
    id::{
        marker::{ChannelMarker, UserMarker},
        Id,
    },
    util::Timestamp,
};

#[derive(Clone)]
pub struct DiscordClient {
    inner: Arc<DiscordClientInner>,
}

pub struct DiscordClientInner {
    cache: DashMap<Id<UserMarker>, Id<ChannelMarker>>,
    client: Client,
}

#[derive(Clone)]
pub struct RaceData {
    pub championship_name: Box<str>,
    pub circuit: String,
    pub start_time: String,
}

impl Deref for DiscordClient {
    type Target = DiscordClientInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DiscordClient {
    pub fn new(token: String) -> Self {
        let inner = DiscordClientInner {
            client: Client::new(token),
            cache: DashMap::new(),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub async fn send_race_notification(&self, user_id: i64, race_data: RaceData) -> AppResult<()> {
        let id = Self::to_user_id(user_id);
        let channel_id = self.get_or_create_dm_channel(id).await?;
        let embed = Self::embed_message(race_data);

        self.client
            .create_message(channel_id)
            .embeds(&[embed])
            .unwrap()
            .await?;

        Ok(())
    }

    //TODO: Capture all errors
    pub async fn send_race_notifications(
        &self,
        user_ids: &[i64],
        race_data: RaceData,
    ) -> AppResult<()> {
        let tasks: Vec<JoinHandle<AppResult<()>>> = user_ids
            .iter()
            .map(|&user_id| {
                let self_clone = self.clone();
                let race_data_clone = race_data.clone();
                tokio::spawn(async move {
                    self_clone
                        .send_race_notification(user_id, race_data_clone)
                        .await
                })
            })
            .collect();

        let results = join_all(tasks).await;

        for result in results {
            result??;
        }

        Ok(())
    }

    #[inline]
    async fn get_or_create_dm_channel(
        &self,
        user_id: Id<UserMarker>,
    ) -> AppResult<Id<ChannelMarker>> {
        if let Some(entry) = self.cache.get(&user_id) {
            return Ok(*entry.value());
        }

        let channel = self
            .client
            .create_private_channel(user_id)
            .await?
            .model()
            .await?;

        self.cache.insert(user_id, channel.id);

        Ok(channel.id)
    }

    #[inline]
    fn embed_message(race_data: RaceData) -> Embed {
        Embed {
            author: Some(EmbedAuthor {
                name: race_data.championship_name.into_string(),
                icon_url: None,
                proxy_icon_url: None,
                url: None,
            }),
            title: Some("Upcoming Race".to_owned()),
            description: Some(
                "The race is about to begin. Get ready for the action on the track!".to_owned(),
            ),
            color: Some(0x00ff0),
            fields: vec![
                EmbedField {
                    name: "Circuit".to_owned(),
                    value: race_data.circuit,
                    inline: true,
                },
                EmbedField {
                    name: "Start Time".to_owned(),
                    value: race_data.start_time,
                    inline: true,
                },
            ],
            footer: Some(EmbedFooter {
                text: "Good luck and may the best driver win!".to_owned(),
                icon_url: None,
                proxy_icon_url: None,
            }),
            timestamp: Some(
                Timestamp::from_secs(Instant::now().elapsed().as_secs() as i64).unwrap(),
            ),
            video: None,
            image: None,
            provider: None,
            thumbnail: None,
            url: None,
            kind: "".to_owned(),
        }
    }

    #[inline]
    fn to_user_id(id: i64) -> Id<UserMarker> {
        Id::new(id as u64)
    }
}
