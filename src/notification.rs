use time::{format_description::well_known::Iso8601, Duration, OffsetDateTime};

use crate::error::NotificationError;
use crate::prelude::Error;

#[derive(Debug)]
pub struct Notification {
    pub video_id: String,
    pub channel_id: String,
    pub video_title: String,
    pub channel_name: String,
    pub published: OffsetDateTime,
    pub updated: OffsetDateTime,
    pub raw: String,
}

impl Notification {
    pub fn try_parse(xml: &str) -> Result<Self, Error> {
        let parsed = super::parse::parse_xml(xml);

        let video = Notification {
            video_id: (*parsed
                .get("yt:videoId")
                .ok_or_else(|| NotificationError::MissingParameter("yt:videoId".to_string()))?)
            .to_string(),
            channel_id: (*parsed
                .get("yt:channelId")
                .ok_or_else(|| NotificationError::MissingParameter("yt:channelId".to_string()))?)
            .to_string(),
            video_title: (*parsed
                .get("title")
                .ok_or_else(|| NotificationError::MissingParameter("title".to_string()))?)
            .to_string(),
            channel_name: (*parsed
                .get("name")
                .ok_or_else(|| NotificationError::MissingParameter("name".to_string()))?)
            .to_string(),
            published: OffsetDateTime::parse(
                parsed
                    .get("published")
                    .ok_or_else(|| NotificationError::MissingParameter("published".to_string()))?,
                &Iso8601::DEFAULT,
            )
            .map_err(NotificationError::DateTimeError)?,
            updated: OffsetDateTime::parse(
                parsed
                    .get("updated")
                    .ok_or_else(|| NotificationError::MissingParameter("updated".to_string()))?,
                &Iso8601::DEFAULT,
            )
            .map_err(NotificationError::DateTimeError)?,
            raw: xml.to_string(),
        };

        Ok(video)
    }

    pub fn is_new(&self) -> bool {
        self.updated - self.published < Duration::minutes(5)
    }
}

impl std::fmt::Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = format!(
            "Title: {}\nAuthor: {}\n#id: {}; channel: {}",
            self.video_title, self.channel_name, self.video_id, self.channel_id
        );
        writeln!(f, "{text}")
    }
}
