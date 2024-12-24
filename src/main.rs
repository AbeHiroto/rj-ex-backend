use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use std::env;
use std::collections::HashMap;
use scraper::{Html, Selector};
use url::form_urlencoded;
use reqwest;
use dotenv::dotenv;

async fn api_endpoint(req: HttpRequest) -> impl Responder {
    let key = req.headers().get("API-Key");
    let env_key = env::var("API_KEY").unwrap();
    match key {
        Some(key) => {
            if *key == env_key {
                let query: HashMap<String, String> = form_urlencoded::parse(req.query_string().as_bytes())
                    .into_owned()
                    .collect();
                let word = query.get("word").unwrap();
                let data = get_data(word).await;
                data
            } else {
                HttpResponse::Unauthorized().finish()
            }
        }
        None => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_data(word: &str) -> HttpResponse {
    println!("Received data: {}", word);
    // "searchpageURL" has to be set to the URL of your target search page
    let url = format!("https://searchpageURL", word);
    let client = reqwest::Client::new();
    let res = client.get(url)
       .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/74.0.3729.169 Safari/537.3")
       .send()
       .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let html = Html::parse_document(&response.text().await.unwrap());
                let link_selector = Selector::parse("section.searchSerp > dl > dt > h4 > a").unwrap();
                let link = match html.select(&link_selector).next() {
                    Some(link) => link.value().attr("href"),
                    None => return HttpResponse::NotFound().finish(),
                };
                // Also edit this part for your target page
                let word_url = format!("https://yourtargetpage.com{}", link.unwrap());

                let res = client.get(word_url)
                   .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/74.0.3729.169 Safari/537.3")
                   .send()
                   .await;

                match res {
                    Ok(response) => {
                        if response.status().is_success() {
                            let html = Html::parse_document(&response.text().await.unwrap());
                            let head_selector = Selector::parse("div.ex > h3").unwrap();
                            let desc_selector = match Selector::parse("section.description > p") {
                                Ok(selector) => selector,
                                Err(err) => {
                                    eprintln!("Error parsing selector: {}", err);
                                    return HttpResponse::InternalServerError().finish();
                                }
                            };

                            let head = html.select(&head_selector).next().unwrap().text().collect::<String>();
                            let desc = html.select(&desc_selector).map(|x| x.text().collect::<String>());

                            let mut word_data = format!("{}{}", head, "\n\n");
                            for desc_element in desc {
                                word_data += &format!("{}\n", desc_element);
                            }
                            HttpResponse::Ok().body(word_data)
                        } else {
                            HttpResponse::NotFound().finish()
                        }
                    }
                    Err(err) => {
                        eprintln!("Error fetching word data: {}", err);
                        HttpResponse::InternalServerError().finish()
                    }
                }
            } else {
                HttpResponse::NotFound().finish()
            }
        }
        Err(err) => {
            eprintln!("Error fetching word data: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    HttpServer::new(move || {
        App::new()
           .wrap(Cors::permissive())
           .route("/api", web::get().to(api_endpoint))
    })
   .bind("127.0.0.1:8080")?
   .run()
   .await
}