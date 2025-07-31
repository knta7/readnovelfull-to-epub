use std::borrow::ToOwned;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use serde_json::{json, Value};
use std::fs;
use std::string::ToString;
use epub_builder::{EpubBuilder, ZipLibrary};


const BASE_URL: &str = "https://readnovelfull.com";
fn main() {
    console_log::init_with_level(log::Level::Info).expect("Cannot init console_log");

    let novel_id = "9";
    let chapters = get_chapter_list(novel_id);
    // println!("{:#?}", chapters);

    let image_bytes = fs::read("emperors domination cover.jpg").unwrap();

    let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    builder.metadata("author", "Yao Bi Xiao Sheng").unwrap()
        .metadata("title", "Emperor's Domination").unwrap()
        .add_cover_image("cover.jpg", &image_bytes[..], "image/jpeg").unwrap();


    for (id, chapter) in chapters.iter().enumerate() {
        let chapter_url = format!("{}{}", BASE_URL, chapter.href);
        let mut body = download_if_missing(&chapter.title, &chapter_url);
        let document = Html::parse_document(&body);

        let selector = Selector::parse("div#chr-content").unwrap();
        if let Some(ele) = document.select(&selector).next() {
            let data = ele.inner_html().replace("<br>", "<p><br /></p>");
            // println!("{}", data);
            // println!("{}", chapter.title);

            body = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
            <html xmlns=\"http://www.w3.org/1999/xhtml\">
                <head>
                    <title>{}</title>
                </head>
                <body>
                <h1>{}</h1>{}
                </body>
            </html>", chapter.title, chapter.title, data);
        }
        builder
            .add_content(epub_builder::EpubContent::new(&format!("{}.xhtml", sanitize_names(&chapter.title)), body.as_bytes())).unwrap();
    }

    let mut output: Vec<u8> = vec![];

    builder.generate(&mut output).unwrap();
    fs::write("output.epub", &output).unwrap();
}


fn does_file_exist(file: &str) -> bool {
    fs::exists(&format!("./data/{}", file)).unwrap()
}

fn get_file(file: &str) -> Option<String> {
    match fs::read_to_string(format!("./data/{}", file)) {
        Ok(contents) => Some(contents),
        Err(_) => None,
    }
}


fn download_if_missing(file_name: &str, url: &str) -> String {
    let sanitized_name = sanitize_names(file_name);
     match get_file(&sanitized_name) {
        Some(contents) => {
            println!("Found file {}, skipping download", sanitized_name);
            contents
        },
        None => {
            println!("Missing file {}, downloading", sanitized_name);
            let response = get(url).unwrap();
            let data = response.text().unwrap();
            fs::write(&format!("./data/{}", sanitized_name), &data).expect("Cannot write file");
            data
        }
    }
}

fn sanitize_names(name: &str) -> String {
    let invalid_chars  = vec!['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

    let mut output = name.to_owned();
    for c in invalid_chars.iter() {
        output = output.replace(*c, "");
    }
    output
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

fn get_chapter_list(novel_id: &str) -> Vec<Chapter> {
    let url = format!("{}/ajax/chapter-archive?novelId={}", BASE_URL, novel_id);
    let chapter_list_id = format!("NovelID_{}", novel_id);
    let body: String = download_if_missing(&chapter_list_id, &url);

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