use crate::prelude::Error;

#[derive(Debug)]
pub struct Notification {
    pub video_id: String,
    pub channel_id: String,
    pub video_title: String,
    pub channel_name: String,
    pub raw: String,
}

impl Notification {
    pub fn try_parse(xml: &str) -> Result<Self, Error> {
        let parsed = super::parse::parse_xml(xml);

        let video = Notification {
            video_id: parsed
                .get("yt:videoId")
                .ok_or_else(|| Error::NotificationError("yt:videoId".to_string()))?
                .to_string(),
            channel_id: parsed
                .get("yt:channelId")
                .ok_or_else(|| Error::NotificationError("yt:channelId".to_string()))?
                .to_string(),
            video_title: parsed
                .get("title")
                .ok_or_else(|| Error::NotificationError("title".to_string()))?
                .to_string(),
            channel_name: parsed
                .get("name")
                .ok_or_else(|| Error::NotificationError("name".to_string()))?
                .to_string(),
            raw: xml.to_string(),
        };

        Ok(video)
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
