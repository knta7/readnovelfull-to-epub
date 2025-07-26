use reqwest::blocking::get;
use scraper::{Html, Selector};
use serde_json::{json, Value};
use std::fs;
fn main() {
    let base_url = "https://readnovelfull.com";
    let chapter_list_url = format!("{}/ajax/chapter-archive?novelId=9", base_url);
    let chapters = get_chapter_list(&chapter_list_url);
    println!("{:#?}", chapters);

    for (id, chapter) in chapters.iter().enumerate() {
        let chapter_url = format!("{}{}", base_url, chapter.href);
        let response = get(chapter_url).unwrap();
        let body = response.text().unwrap();
        let document = Html::parse_document(&body);

        // Example: Select all anchor elements and extract their href and text
        let selector = Selector::parse("p").unwrap();

        let val = document.select(&selector).map(|ele| {
            let text = ele.text().collect::<Vec<_>>().join(" ").trim().to_string();
            text
        }).collect::<Vec<_>>();

        println!("{:#?}", val);

        break
    }


}

#[derive(Debug)]
struct Chapter {
    title: String,
    href: String,
    number: usize,
}

impl Chapter {
    pub fn new(title: String, href: String, number: usize) -> Chapter {
        Chapter {
            title,
            href,
            number,
        }
    }
}

fn get_chapter_list(url: &str) -> Vec<Chapter> {
    let response = get(url).unwrap();
    let body = response.text().unwrap();
    let document = Html::parse_document(&body);

    // Example: Select all anchor elements and extract their href and text
    let selector = Selector::parse("a").unwrap();
    document.select(&selector)
        // Gets rid of chapters that don't have numbers - to handle later
        .filter(|ele| {
            let href = ele.attr("href").unwrap_or("").to_string();
            let chapter = href.split("/chapter-").collect::<Vec<&str>>();
            chapter.len() == 2
        } )
        .map( |ele| {
            let href = ele.attr("href").unwrap_or("").to_string();
            let text = ele.text().collect::<Vec<_>>().join(" ").trim().to_string();
            // println!("{:?}", href.clone().split("/chapter-").collect::<Vec<&str>>());
            let chapter = href.clone().split("/chapter-")
                .collect::<Vec<&str>>()[1].split("-")
                .collect::<Vec<&str>>()[0].split(".")
                .collect::<Vec<&str>>()[0].split("-")
                .collect::<Vec<&str>>()[0].to_string().parse::<usize>().unwrap();
        Chapter::new(text, href, chapter)
    }).collect()
}


fn html_to_json(html_string: &str) -> Value {
    let document = Html::parse_document(html_string);
    let mut json_nodes = Vec::new();

    // Example: Select all div elements and extract their id and text
    let selector = Selector::parse("a").unwrap();
    for element in document.select(&selector) {
        let href = element.attr("href").unwrap_or("").to_string();
        let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
        json_nodes.push(json!({
            "href": href,
            "text": text,
        }));
    }

    json!({
        "elements": json_nodes
    })
}