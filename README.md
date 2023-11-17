# Config file
webhook.toml

```toml
[callback]
port = PORT
host = CALLBACK_URL

[youtube]
hub = HUB_URL
topic = TOPIC_URL
```

> *topic* is the URL without the channel id.
>
> For youtube, it should be https://www.youtube.com/xml/feeds/videos.xml?channel_id=

[Pubsubhubbuh 0.4 specification](https://pubsubhubbub.github.io/PubSubHubbub/pubsubhubbub-core-0.4.html#discovery)
