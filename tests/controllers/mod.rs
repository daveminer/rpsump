use linkify::{LinkFinder, LinkKind};
use reqwest::Url;

pub mod auth;

pub fn link_from_email_text<'a>(text: &str) -> Vec<String> {
    let finder = LinkFinder::new();
    let links: Vec<_> = finder.links(text).collect();

    let mut found_links = vec![];
    for link in links {
        if link.kind() == &LinkKind::Url {
            found_links.push(link.as_str().to_string())
        }
    }

    return found_links;
}

pub fn param_from_email_text<'a>(text: &str, param: &str) -> Vec<String> {
    let finder = LinkFinder::new();
    let links: Vec<_> = finder.links(text).collect();

    let mut found_params = vec![];
    for link in links {
        if link.kind() == &LinkKind::Url {
            let url = Url::parse(link.as_str()).unwrap();
            let query_pairs = url.query_pairs();
            for pair in query_pairs {
                if pair.0 == param {
                    found_params.push(pair.1.into_owned());
                }
            }
        }
    }

    return found_params;
}
