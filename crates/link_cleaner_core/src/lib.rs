use regex::Regex;
use std::sync::OnceLock;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanReport {
    pub output: String,
    pub urls_found: usize,
    pub urls_modified: usize,
    pub params_removed: usize,
}

pub fn clean_text(input: &str) -> String {
    clean_text_with_report(input).output
}

pub fn clean_text_with_report(input: &str) -> CleanReport {
    let mut output = String::with_capacity(input.len());
    let mut last = 0;
    let mut urls_found = 0;
    let mut urls_modified = 0;
    let mut params_removed = 0;

    for m in url_regex().find_iter(input) {
        urls_found += 1;

        output.push_str(&input[last..m.start()]);

        let matched = m.as_str();
        let (url_part, trailing) = split_url_and_trailing_punctuation(matched);
        let (cleaned, modified, removed) = clean_single_url(url_part);

        if modified {
            urls_modified += 1;
            params_removed += removed;
        }

        output.push_str(&cleaned);
        output.push_str(trailing);

        last = m.end();
    }

    output.push_str(&input[last..]);

    CleanReport {
        output,
        urls_found,
        urls_modified,
        params_removed,
    }
}

fn url_regex() -> &'static Regex {
    static URL_RE: OnceLock<Regex> = OnceLock::new();
    URL_RE.get_or_init(|| Regex::new(r"https?://[^\s]+").expect("valid URL regex"))
}

fn split_url_and_trailing_punctuation(candidate: &str) -> (&str, &str) {
    let mut cut = candidate.len();

    while cut > 0 {
        let ch = candidate[..cut]
            .chars()
            .next_back()
            .expect("non-empty while loop");

        if !is_trailing_punctuation(ch) {
            break;
        }

        if ch == ')' && !has_unmatched_closing_paren(&candidate[..cut]) {
            break;
        }

        cut -= ch.len_utf8();
    }

    (&candidate[..cut], &candidate[cut..])
}

fn is_trailing_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '.' | ',' | ':' | ';' | '!' | '?' | ')' | ']' | '}' | '"' | '\''
    )
}

fn has_unmatched_closing_paren(text: &str) -> bool {
    let mut open = 0usize;
    let mut close = 0usize;

    for ch in text.chars() {
        if ch == '(' {
            open += 1;
        } else if ch == ')' {
            close += 1;
        }
    }

    close > open
}

fn clean_single_url(raw_url: &str) -> (String, bool, usize) {
    let mut url = match Url::parse(raw_url) {
        Ok(url) => url,
        Err(_) => return (raw_url.to_string(), false, 0),
    };

    let mut modified = false;
    let mut removed = 0usize;
    let is_twitter_url = rewrite_twitter_to_xcancel(&mut url);

    if is_twitter_url {
        modified = true;
    }

    if strip_amazon_ref_path(&mut url) {
        modified = true;
    }

    if let Some(pairs) = collect_query_pairs(url.query()) {
        let mut kept = Vec::with_capacity(pairs.len());

        for (key, value) in pairs {
            if should_remove_param(&key) || (is_twitter_url && should_remove_twitter_param(&key)) {
                removed += 1;
            } else {
                kept.push((key, value));
            }
        }

        if removed > 0 {
            modified = true;
            if kept.is_empty() {
                url.set_query(None);
            } else {
                let mut serializer = url::form_urlencoded::Serializer::new(String::new());
                for (key, value) in kept {
                    serializer.append_pair(&key, &value);
                }
                let new_query = serializer.finish();
                url.set_query(Some(&new_query));
            }
        }
    }

    if modified {
        (url.into(), true, removed)
    } else {
        (raw_url.to_string(), false, 0)
    }
}

fn rewrite_twitter_to_xcancel(url: &mut Url) -> bool {
    let host = match url.host_str() {
        Some(host) => host.to_ascii_lowercase(),
        None => return false,
    };

    if !is_twitter_host(&host) {
        return false;
    }

    url.set_host(Some("xcancel.com")).is_ok()
}

fn is_twitter_host(host: &str) -> bool {
    matches!(
        host,
        "x.com"
            | "www.x.com"
            | "mobile.x.com"
            | "m.x.com"
            | "twitter.com"
            | "www.twitter.com"
            | "mobile.twitter.com"
            | "m.twitter.com"
    )
}

fn should_remove_twitter_param(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "s" | "t" | "cxt" | "cn" | "twclid" | "ref_src" | "ref_url"
    )
}

fn collect_query_pairs(query: Option<&str>) -> Option<Vec<(String, String)>> {
    let query = query?;
    let pairs = url::form_urlencoded::parse(query.as_bytes())
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect::<Vec<_>>();
    Some(pairs)
}

fn should_remove_param(key: &str) -> bool {
    let key = key.to_ascii_lowercase();

    key.starts_with("utm_")
        || key.starts_with("hsa_")
        || key.starts_with("ga_")
        || key.starts_with("pk_")
        || key.starts_with("mc_")
        || matches!(
            key.as_str(),
            "gclid"
                | "gclsrc"
                | "dclid"
                | "msclkid"
                | "fbclid"
                | "twclid"
                | "wbraid"
                | "gbraid"
                | "gad_source"
                | "srsltid"
                | "mc_cid"
                | "mc_eid"
                | "_hsenc"
                | "_hsmi"
                | "__hsfp"
                | "__hssc"
                | "__hstc"
                | "mkt_tok"
                | "oly_anon_id"
                | "oly_enc_id"
                | "rb_clickid"
                | "vero_conv"
                | "vero_id"
                | "wickedid"
                | "yclid"
                | "ref"
                | "ref_src"
                | "ref_url"
                | "igshid"
                | "igsh"
                | "si"
                | "tag"
                | "linkcode"
        )
}

fn strip_amazon_ref_path(url: &mut Url) -> bool {
    let host = match url.host_str() {
        Some(host) => host.to_ascii_lowercase(),
        None => return false,
    };

    if !host.contains("amazon.") {
        return false;
    }

    let path = url.path().to_string();
    let Some(index) = path.find("/ref=") else {
        return false;
    };

    let new_path = &path[..index];
    if new_path.is_empty() {
        return false;
    }

    url.set_path(new_path);
    true
}

#[cfg(test)]
mod tests {
    use super::clean_text;

    #[test]
    fn golden_cases_v0() {
        let cases = [
            (
                "Check out this video https://youtu.be/IPPTgd2cdvs?si=xe9oYk8nfQ1HxSbb",
                "Check out this video https://youtu.be/IPPTgd2cdvs",
            ),
            (
                "Here: https://example.com/landing?utm_source=newsletter&utm_medium=email&utm_campaign=spring&utm_content=button",
                "Here: https://example.com/landing",
            ),
            (
                "Deal: https://shop.example.com/p/123?gclid=EAIaIQobChMI&fbclid=IwAR0abc123",
                "Deal: https://shop.example.com/p/123",
            ),
            (
                "Link https://example.com/a?mc_cid=1234567890&mc_eid=abcdef1234",
                "Link https://example.com/a",
            ),
            (
                "Amazon: https://www.amazon.de/dp/B09XYZ1234/ref=sr_1_1?crid=ABCDEF&keywords=foo&tag=mytag-21&linkCode=sl1",
                "Amazon: https://www.amazon.de/dp/B09XYZ1234?crid=ABCDEF&keywords=foo",
            ),
            (
                "IG: https://www.instagram.com/reel/CrAbCdEfGhi/?utm_source=ig_web_copy_link&igsh=MzRlODBiNWFlZA==",
                "IG: https://www.instagram.com/reel/CrAbCdEfGhi/",
            ),
            (
                "TikTok: https://www.tiktok.com/@user/video/1234567890?utm_source=copy_link&utm_medium=android&utm_campaign=client_share",
                "TikTok: https://www.tiktok.com/@user/video/1234567890",
            ),
            (
                "X: https://twitter.com/user/status/1234567890123456789?ref_src=twsrc%5Etfw&t=20",
                "X: https://xcancel.com/user/status/1234567890123456789",
            ),
            (
                "X mobile: https://x.com/tristan0x/status/2023437922150871104?s=46",
                "X mobile: https://xcancel.com/tristan0x/status/2023437922150871104",
            ),
            (
                "X keep params: https://x.com/user/status/1?s=46&t=abc",
                "X keep params: https://xcancel.com/user/status/1",
            ),
            (
                "X strip more: https://twitter.com/user/status/1?cxt=HHwWgoCz8f&t=abc&cn=ZmxleGlibGVfcmVjcw==&twclid=abc123",
                "X strip more: https://xcancel.com/user/status/1",
            ),
            (
                "Ads params: https://example.com/p?gclid=1&dclid=2&msclkid=3&wbraid=4&gbraid=5&gad_source=6&srsltid=7",
                "Ads params: https://example.com/p",
            ),
            (
                "Marketing params: https://example.com/page?mkt_tok=abc&rb_clickid=def&vero_id=ghi&oly_anon_id=jkl",
                "Marketing params: https://example.com/page",
            ),
            (
                "Non-X keeps t: https://example.com/video?t=120&v=abc",
                "Non-X keeps t: https://example.com/video?t=120&v=abc",
            ),
            (
                "Maps: https://www.google.com/maps/place/Berlin/?utm_source=share&api=1&query=Berlin",
                "Maps: https://www.google.com/maps/place/Berlin/?api=1&query=Berlin",
            ),
            (
                "Two links: (https://youtu.be/IPPTgd2cdvs?si=abc), and https://example.com/?utm_source=x. End.",
                "Two links: (https://youtu.be/IPPTgd2cdvs), and https://example.com/. End.",
            ),
            ("Just text without a URL.", "Just text without a URL."),
            (
                "Doc: https://example.com/page?utm_source=a#section-2",
                "Doc: https://example.com/page#section-2",
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(clean_text(input), expected, "failed for input: {input}");
        }
    }
}
