use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde_json::{json, Value, Map};

macro_rules! h200 {
  ($resp:expr) => {
    HttpResponse::Ok().body($resp)
  }
}

macro_rules! h403 {
  ($resp:expr) => {
    HttpResponse::BadRequest().body($resp)
  }
}

macro_rules! tplt {
  ($api:tt) => {
    match std::fs::read_to_string(format!("api/{api}.json", api=$api)) {
      Ok(v) => {v},
      Err(_) => {"".to_string()}
    }
  }
}

macro_rules! api {
  ($api:tt, $func:ident) => {
    #[post($api)]
    async fn $func(cache: web::Data<Map<String, Value>>) -> impl Responder {
      match serde_json::to_string(&cache[(stringify!($func))]) {
        Ok(v) => h200! {v},
        Err(_) => h403! {"Invalid Template"}
      }
    }
  }
}

lazy_static::lazy_static! {
  static ref CONFIG: Value = {
    match std::fs::read_to_string("config.json") {
      Ok(v) => {
        match serde_json::from_str(&v) {
          Ok(v) => v,
          _ => json!({})
        }
      },
      _ => json!({})
    }
  };
}

#[post("/api/api/login")]
async fn alogin() -> impl Responder {
  h200! {r#"
    {
      "page": "user_login",
      "error_flg": 0,
    }
    "#
  }
}

api!{"/title/login", tlogin}
api!{"/mypage/mypage", mypage}
api!{"/library/librarytop/midoga_story/", midoga_story}
api!{"/library/librarytop/story/", story}

#[post("/library/librarytop")]
async fn library(params: web::Form<Map<String, Value>>, cache: web::Data<Map<String, Value>>) -> impl Responder {
  match params.get("view_type") {
    Some(Value::String(vtype)) => {
      match cache["library"].as_object() {
        Some(v) => {
          let mut p = v.clone();
          let mut characters: Vec<Value> = Vec::new();
          let mut cards: Vec<Value> = Vec::new();
          match params["character_id"].as_str() {
            Some(cid) => {
              let serif;
              let flavor;
              if vtype == "2" {
                cards.push(cache[&format!("card_{cid}")].clone());
                serif = "";
                flavor = "";
              } else {
                characters.push(cache[&format!("chara_{cid}")].clone());
                serif = match cache[&format!("serif_{cid}")].as_str() {
                  Some(v) => v,
                  _ => ""
                };
                flavor = match cache[&format!("flavor_{cid}")].as_str() {
                  Some(v) => v,
                  _ => ""
                };
              }
              p.insert("m_text".to_string(), json!({
                "serif": {
                  "text": serif 
                },
                "flavor": {
                  "text": flavor
                }
              }));
            },
            _ => {}
          }
          p.insert("chara_datas".to_string(), Value::Array(characters));
          p.insert("card_datas".to_string(), Value::Array(cards));
          match serde_json::to_string(&p) {
            Ok(v) => h200! {v},
            Err(_) => h403! {"Invalid Template"}
          }
        },
        _ => h403! {"Missing Template"}
      }
    },
    _ => {
      match cache.get("library") {
        Some(s) => {
          match serde_json::to_string(s) {
            Ok(v) => h200! {v},
            Err(_) => h403! {"Invalid Template"}
          }
        },
        _ => h403! {"Missing Template"}
      }
    }
  }
}

#[actix_web::main]
#[allow(unused_must_use)]
async fn main() -> std::io::Result<()> {
  let host = match CONFIG["host"].as_str() {
    Some(v) => v,
    _ => "127.0.0.1:443"
  };
  let wks = match CONFIG["workers"].as_u64() {
    Some(v) => v as usize,
    _ => 1
  };
  println!("Game server started, plz check the following url address");
  println!("http://{}/******", host);
  HttpServer::new(|| {
    let mut engine = upon::Engine::new();
    let mut cache: Map<String, Value> = Map::new();
    for x in ["tlogin", "mypage", "midoga_story", "story", "library"] {
      engine.add_template(x.to_string(), tplt!{x});
      match engine.get_template(x) {
        Some(template) => {
          match template.render(CONFIG.clone()) {
            Ok(s) => {
              match serde_json::from_str(&s) {
                Ok(Value::Object(j)) => {
                  if x == "library" {
                    match j["chara_datas"].as_array() {
                      Some(arr) => {
                        for chara in arr.iter() {
                          match chara["character_id"].as_str() {
                            Some(cid) => {
                              cache.insert(format!("chara_{cid}"), chara.clone());
                              cache.insert(format!("serif_{cid}"), chara["serif"].clone());
                              cache.insert(format!("flavor_{cid}"), chara["flavor"].clone());
                            },
                            _ => {}
                          };
                        }
                      },
                      _ => {}
                    }
                    match j["card_datas"].as_array() {
                      Some(arr) => {
                        for chara in arr.iter() {
                          match chara["character_id"].as_str() {
                            Some(cid) => {
                              cache.insert(format!("card_{cid}"), chara.clone());
                            },
                            _ => {}
                          };
                        }
                      },
                      _ => {}
                    }
                  }
                  cache.insert(x.to_string(), Value::Object(j));
                }
                _ => {}
              }
            },
            Err(_) => {}
          }
        },
        _ => {}
      }
    };
    App::new()
      .app_data(web::Data::new(cache))
      .service(alogin)
      .service(tlogin)
      .service(mypage)
      .service(midoga_story)
      .service(story)
      .service(library)
      .service(actix_files::Files::new("/", "."))
  })
  .workers(wks)
  .bind(host)?
  .run()
  .await
}
