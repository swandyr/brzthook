use quick_xml::{events::Event, Reader};
use std::{borrow::Cow, collections::HashMap};

pub(super) fn parse(xml: &str) -> Result<Vec<Cow<'_, str>>, Box<dyn std::error::Error>> {
    // Should handle utf16? (quick_xml --features encoding)

    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut parsed = vec![];
    loop {
        match reader.read_event()? {
            Event::Eof => break,
            Event::Text(txt) => parsed.push(txt.unescape()?),
            _ => {}
        }
    }

    //dbg!(&parsed);

    Ok(parsed)
}

pub(super) fn my_parse(xml: &str) -> HashMap<&str, &str> {
    let mut v = HashMap::new();
    let tags = &["<yt:videoId>", "<yt:channelId>", "<title>", "<name>"];
    let mut lines = xml.lines().map(|l| l.trim());

    for &t in tags {
        let l = lines.find(|l| l.starts_with(t));
        if let Some(l) = l {
            // length of tag
            let tl = t.len();
            // length of line
            let ll = l.len();

            // <tag> without brackets
            let t = &t[1..(tl - 1)];
            // <tag>value</tag>
            // value starts at index = tag.len() (here 4)
            // value ends at index = tag.len - line.len - 1 (-1 because </tag> is one char longer
            // than <tag>)
            let l = &l[tl..(ll - tl - 1)];

            v.insert(t, l);
        }
    }

    v
}
