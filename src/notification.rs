#[derive(Debug)]
pub struct Notification {
    pub video_id: String,
    pub channel_id: String,
    pub video_title: String,
    pub channel_name: String,
    pub raw: String,
}

impl Notification {
    pub fn try_parse(xml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed = super::parse::parse(xml)?;

        let video = Notification {
            video_id: parsed[3].to_string(),
            channel_id: parsed[4].to_string(),
            video_title: parsed[5].to_string(),
            channel_name: parsed[6].to_string(),
            raw: parsed.iter().map(|s| s.to_string()).collect(),
        };

        Ok(video)
    }

    pub fn try_my_parse(xml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed = super::parse::my_parse(xml);

        let video = Notification {
            video_id: parsed.get("yt:videoId").ok_or("no videoId")?.to_string(),
            channel_id: parsed
                .get("yt:channelId")
                .ok_or("no channelId")?
                .to_string(),
            video_title: parsed.get("title").ok_or("no title")?.to_string(),
            channel_name: parsed.get("name").ok_or("no name")?.to_string(),
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
