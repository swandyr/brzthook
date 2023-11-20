use std::collections::HashMap;

pub(super) fn parse_xml(xml: &str) -> HashMap<&str, &str> {
    let mut v = HashMap::new();
    let tags = &[
        "<yt:videoId>",
        "<yt:channelId>",
        "<title>",
        "<name>",
        "<published>",
        "<updated>",
    ];
    let mut lines = xml.lines().map(str::trim);

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
