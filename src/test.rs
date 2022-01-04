const TEXT: &str = "&lt;p&gt;&lt;strong&gt;t;&gt; - &lt;/p&gt;";

#[test]
fn html2text() {
    use html2text::from_read;

    let res = from_read(TEXT.as_bytes(), 10000);
    let res = from_read(res.as_bytes(), 10000);

    println!("{}", res)
}

#[test]
fn feed() {
    let content = std::fs::read("data/miao.xml").unwrap();
    let res = feed_rs::parser::parse(&content[..]).unwrap();
    res.entries.into_iter().for_each(|x| {
        println!(
            "{}",
            /* x.summary.map(|x| x.content) */
            x.summary
                .map(|x| html2text::from_read(x.content.as_bytes(), 200))
                .unwrap()
        )
    })
}

#[test]
fn escape() {
    let content = std::fs::read("data/miao.xml").unwrap();
    let feed = feed_rs::parser::parse(&content[..]).unwrap();
    let entity = feed.entries.get(0).unwrap();
    let summary = entity.summary.as_ref().unwrap();
    let encoded = html_escape::encode_safe(&summary.content);
    println!("{}", encoded)
}
