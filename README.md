# txts.rs
Rust library for interacting with txts

# Examples
## Initializing client
```rs
let host = Url::parse("https://txts.sudokoko.xyz/").unwrap();
let client = TxtsClient::new(host)?;
```

## Getting page info
```rs
let page = client.get("username").await?;
println!("Verified: {}", page.verified);
println!("HTML: {}", page.html_content);
```

## Getting page content in markdown
```rs
let markdown = client.get_markdown("username").await?;
```

## Creating page
```rs
let (page, secret) = client.create("username", "content").await?;
```

## Editing page
```rs
let secret = Uuid::parse_str("Secret here").unwrap();
let edited_page = client.edit("username", &secret, "content").await?;
```
