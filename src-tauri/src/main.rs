#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use anyhow::Result;
use base64::encode;
use crunchyroll_rs::feed::RecommendationOptions;
use crunchyroll_rs::media::MediaCollection;
use crunchyroll_rs::search::QueryOptions;
use crunchyroll_rs::{Crunchyroll, Episode, Media, Season, Series};
use rsubs_lib::ssa;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{CustomMenuItem, Menu, Submenu};
use tauri::{Manager, State};
use tokio::sync::Mutex;

#[tauri::command]
async fn login(
    username: &str,
    password: &str,
    crunchyroll: State<'_, ViewerContext>,
) -> Result<String, String> {
    println!("Hello, {}!", username);
    *crunchyroll.session.lock().await = Some(
        match Crunchyroll::builder()
            .login_with_credentials(username, password)
            .await
        {
            Ok(cr) => cr,
            Err(_e) => {
                println!("{}", _e);
                return Err(format!("Failed to log in.{}", _e));
            }
        },
    );
    let sess = crunchyroll.clone();
    let aid = sess
        .session
        .lock()
        .await
        .clone()
        .expect("Failed to log In.");
    let aid2 = aid
        .account()
        .await
        .expect("Failed get account details.")
        .email;
    Ok(format!(
        "Welcome {}, you have logged in. {} ",
        username, aid2
    ))
}

#[tauri::command]
async fn login_anon(crunchyroll: State<'_, ViewerContext>) -> Result<String, String> {
    println!("Hello, Anon!!");
    *crunchyroll.session.lock().await =
        Some(match Crunchyroll::builder().login_anonymously().await {
            Ok(cr) => cr,
            Err(_e) => {
                println!("{}", _e);
                return Err(format!("Failed to log in.{}", _e));
            }
        });
    Ok(format!("Welcome {}, you have logged in.", "Anon"))
}

#[derive(Serialize, Debug, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct SearchResult {
    pub name: String,
    pub id: String,
    pub desc: String,
    pub img: Option<String>,
}

#[tauri::command]
async fn search_crunchy(
    series_name: &str,
    crunchyroll: State<'_, ViewerContext>,
) -> Result<Vec<SearchResult>, String> {
    let ses = &(*crunchyroll);
    let aid = ses.session.lock().await;
    let aid2 = if aid.as_ref().is_some() {
        Ok(aid.as_ref().expect("Failed login"))
    } else {
        Err("Not Logged in.")
    };
    if aid2.is_err() {
        return Err("You are not logged in.".to_string());
    }
    let result: Vec<Media<Series>>;
    let query_res = aid2?
        .query(series_name, QueryOptions::default())
        .await
        .expect("Failed to Query");
    result = query_res.series.expect("Failed to find a series").items;
    let mut a: Vec<SearchResult> = vec![];
    for r in result {
        a.push(SearchResult {
            name: r.title,
            id: r.id,
            desc: r.description,
            img: Some(if let Some(image) = r.images {
                if let Some(pic) = image.thumbnail {
                    let mut p: Vec<String> = vec![];
                    for mut i in pic {
                        i.sort_by(|l, j| j.height.cmp(&l.height));
                        p.push(i.first().unwrap().clone().source);
                    }
                    p.first().unwrap().to_string()
                } else if let Some(pic) = image.poster_wide {
                    let mut p: Vec<String> = vec![];
                    for mut i in pic {
                        i.sort_by(|l, j| j.height.cmp(&l.height));
                        p.push(i.first().unwrap().clone().source);
                    }
                    p.first().unwrap().to_string()
                } else if let Some(pic) = image.promo_image {
                    let mut p: Vec<String> = vec![];
                    for mut i in pic {
                        i.sort_by(|l, j| j.height.cmp(&l.height));
                        p.push(i.first().unwrap().clone().source);
                    }
                    p.first().unwrap().to_string()
                } else if let Some(pic) = image.poster_tall {
                    let mut p: Vec<String> = vec![];
                    for mut i in pic {
                        i.sort_by(|l, j| j.height.cmp(&l.height));
                        p.push(i.first().unwrap().clone().source);
                    }
                    p.first().unwrap().to_string()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }),
        });
    }

    Ok(a)
}
#[tauri::command]
async fn get_seasons(
    series_name: &str,
    crunchyroll: State<'_, ViewerContext>,
) -> Result<Vec<SearchResult>, String> {
    let ses = &(*crunchyroll);
    let aid = ses.session.lock().await;
    let aid2 = if aid.as_ref().is_some() {
        Ok(aid.as_ref().unwrap())
    } else {
        Err("Not Logged in.")
    };
    if aid2.is_err() {
        return Err("You are not logged in.".to_string());
    }
    let result: Vec<Media<Season>>;
    let query_res: Media<Series> = aid2?.media_from_id(series_name).await.unwrap();
    result = query_res.seasons().await.unwrap();
    let mut a: Vec<SearchResult> = vec![];
    for r in result {
        let i: Option<String> = if let Some(pic) = query_res.images.clone() {
            let a = pic.poster_tall.filter(|x| !x.is_empty()).map(|x| {
                let mut p: Vec<String> = vec![];
                for mut i in x {
                    i.sort_by(|l, j| j.height.cmp(&l.height));
                    p.push(i.first().unwrap().clone().source);
                }
                p.join(" ")
            });
            if a.is_some() {
                a
            } else {
                Some("".to_string())
            }
        } else {
            Some("".to_string())
        };
        a.push(SearchResult {
            name: r.title,
            id: r.id,
            desc: r.description,
            img: i,
        });
    }
    Ok(a)
}

#[derive(Serialize, Debug, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct EpisodeResult {
    pub title: String,
    pub id: String,
    pub number: i32,
    pub desc: String,
    pub url: String,
    pub img: String,
    pub subs: HashMap<String, String>,
    pub nep: Option<Vec<Ep>>,
}

#[derive(Serialize, Debug, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Ep {
    pub ep_id: String,
    pub ep_type: String,
}

#[derive(Serialize, Debug, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct PlayResult {
    pub audio_locale: String,
    pub subtitles: HashMap<String, Sub>,
    pub streams: HashMap<String, HashMap<String, Strems>>,
    pub QoS: Qos,
}

#[derive(Serialize, Debug, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Sub {
    pub locale: String,
    pub url: String,
    pub format: String,
}

#[derive(Serialize, Debug, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Strems {
    pub hardsub_locale: String,
    pub url: String,
    pub vcodec: String,
}

#[derive(Serialize, Debug, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Qos {
    pub region: String,
    pub cloudFrontRequestId: String,
    pub lambdaRunTime: i32,
}

#[tauri::command]
async fn get_recs(crunchyroll: State<'_, ViewerContext>) -> Result<Vec<SearchResult>, String> {
    let ses = &(*crunchyroll);
    let aid = ses.session.lock().await;
    let aid2 = if aid.as_ref().is_some() {
        Ok(aid.as_ref().expect("Failed login"))
    } else {
        Err("Not Logged in.")
    };
    if aid2.is_err() {
        return Err("You are not logged in.".to_string());
    }
    let mut result: Vec<Media<Series>> = vec![];
    let query_res = aid2?
        .recommendations(RecommendationOptions::default())
        .await
        .expect("Failed to Query");
    for i in query_res.items.iter() {
        let resul2t: Media<Series> = match i {
            MediaCollection::Series(s) => s.clone(),
            _ => continue,
        };
        result.push(resul2t);
    }
    let mut a: Vec<SearchResult> = vec![];
    for r in result {
        a.push(SearchResult {
            name: r.title,
            id: r.id,
            desc: r.description,
            img: Some(if let Some(image) = r.images {
                if let Some(pic) = image.thumbnail {
                    let mut p: Vec<String> = vec![];
                    for mut i in pic {
                        i.sort_by(|l, j| j.height.cmp(&l.height));
                        p.push(i.first().unwrap().clone().source);
                    }
                    p.first().unwrap().to_string()
                } else if let Some(pic) = image.poster_wide {
                    let mut p: Vec<String> = vec![];
                    for mut i in pic {
                        i.sort_by(|l, j| j.height.cmp(&l.height));
                        p.push(i.first().unwrap().clone().source);
                    }
                    p.first().unwrap().to_string()
                } else if let Some(pic) = image.promo_image {
                    let mut p: Vec<String> = vec![];
                    for mut i in pic {
                        i.sort_by(|l, j| j.height.cmp(&l.height));
                        p.push(i.first().unwrap().clone().source);
                    }
                    p.first().unwrap().to_string()
                } else if let Some(pic) = image.poster_tall {
                    let mut p: Vec<String> = vec![];
                    for mut i in pic {
                        i.sort_by(|l, j| j.height.cmp(&l.height));
                        p.push(i.first().unwrap().clone().source);
                    }
                    p.first().unwrap().to_string()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }),
        });
    }

    Ok(a)
}

#[tauri::command(rename_all = "snake_case")]
async fn view_episode(
    ep_id: String,
    ep_type: String,
    crunchyroll: State<'_, ViewerContext>,
) -> Result<EpisodeResult, String> {
    let ses = &(*crunchyroll);
    let aid = ses.session.lock().await;
    let aid2 = if aid.as_ref().is_some() {
        Ok(aid.as_ref().unwrap())
    } else {
        Err("Not Logged in.")
    };
    if aid2.is_err() {
        return Err("You are not logged in.".to_string());
    }
    let query_res: Media<Episode> = aid2?.media_from_id(ep_id.clone()).await.unwrap();
    let next_ep = query_res.season().await.unwrap().episodes().await.unwrap();
    let nepp = next_ep
        .clone()
        .into_iter()
        .filter(|x| x.metadata.episode_number == query_res.metadata.episode_number + 1)
        .collect::<Vec<Media<Episode>>>();
    let mut nep2: Vec<Ep> = vec![];
    for i in nepp {
        nep2.push(Ep {
            ep_id: i.clone().id,
            ep_type: String::from("secondary"),
        });
    }
    let mut subs: HashMap<String, String> = HashMap::new();
    let a = query_res.streams().await.unwrap();
    for item in a.clone().subtitles {
        let a = "data:text/vtt;base64,".to_string()
            + &encode(
                (ssa::parse(
                    aid2?
                        .client()
                        .get(item.1.url.to_string())
                        .send()
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap(),
                )
                .unwrap()
                .to_vtt()
                .to_string())
                .as_bytes(),
            );
        subs.insert(item.0.to_human_readable(), a);
    }
    let b = query_res.streams().await.unwrap();
    for item in b.closed_captions {
        let a = "data:text/vtt;base64,".to_string()
            + &encode(
                (aid2?
                    .client()
                    .get(item.1.url.to_string())
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap())
                .as_bytes(),
            );
        subs.insert(item.0.to_human_readable(), a);
    }
    let url = b
        .variants
        .get(&crunchyroll_rs::Locale::Custom("".to_string()))
        .unwrap()
        .clone()
        .adaptive_hls
        .unwrap()
        .url;
    Ok(EpisodeResult {
        title: query_res.title,
        id: query_res.id,
        number: query_res.metadata.episode_number as i32,
        desc: query_res.description,
        url,
        subs,
        nep: Some(nep2),
        ..Default::default()
    })
}

#[derive(Serialize, Debug, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct epOrSeries {
    pub series_id: Option<String>,
    pub ep_id: Option<String>,
}

#[tauri::command]
async fn get_episodes(
    series_id: Option<String>,
    ep_id: Option<String>,
    crunchyroll: State<'_, ViewerContext>,
) -> Result<Vec<SearchResult>, String> {
    let ses = &(*crunchyroll);
    let aid = ses.session.lock().await;
    let aid2 = if aid.as_ref().is_some() {
        Ok(aid.as_ref().unwrap())
    } else {
        Err("Not Logged in.")
    };
    if aid2.is_err() {
        return Err("You are not logged in.".to_string());
    }
    let result: Vec<Media<Episode>>;
    let sr_ep: MediaCollection = if series_id.is_some() {
        aid2?
            .media_collection_from_id(series_id.unwrap())
            .await
            .unwrap()
    } else {
        aid2?
            .media_collection_from_id(ep_id.unwrap())
            .await
            .unwrap()
    };
    let mut sr2: Option<Media<Season>> = None;
    let mut ep2: Option<Media<Episode>> = None;
    match sr_ep {
        MediaCollection::Series(_) => {}
        MediaCollection::Season(s) => sr2 = Some(s),
        MediaCollection::Episode(e) => ep2 = Some(e),
        MediaCollection::MovieListing(_) => {}
        MediaCollection::Movie(_) => {}
    }
    let mut a: Vec<SearchResult> = vec![];
    if sr2.is_some() {
        let eps = sr2.unwrap().episodes().await;
        result = eps.unwrap();
        for r in result {
            let i: Option<String> = if let Some(pic) = r.images.clone() {
                let imgs = if let Some(a) = pic.thumbnail {
                    a
                } else if let Some(a) = pic.poster_tall {
                    a
                } else if let Some(a) = pic.poster_wide {
                    a
                } else if let Some(a) = pic.promo_image {
                    a
                } else {
                    panic!("no pics");
                };
                let mut ba = Some("".to_string());
                for mut pics in imgs {
                    pics.sort_by(|l, j| j.height.cmp(&l.height));
                    ba = Some(pics.first().unwrap().source.clone());
                }
                if ba.is_some() {
                    ba
                } else {
                    Some("".to_string())
                }
            } else {
                Some("".to_string())
            };
            a.push(SearchResult {
                name: r.title,
                id: r.id,
                desc: r.description,
                img: i,
            });
        }
    } else if ep2.is_some() {
        let ep3 = ep2.unwrap();
        let i: Option<String> = if let Some(pic) = ep3.images.clone() {
            let imgs = if let Some(a) = pic.thumbnail {
                a
            } else if let Some(a) = pic.poster_tall {
                a
            } else if let Some(a) = pic.poster_wide {
                a
            } else if let Some(a) = pic.promo_image {
                a
            } else {
                panic!("no pics");
            };
            let mut ba = Some("".to_string());
            for mut pics in imgs {
                pics.sort_by(|l, j| j.height.cmp(&l.height));
                ba = Some(pics.first().unwrap().source.clone());
            }
            if ba.is_some() {
                ba
            } else {
                Some("".to_string())
            }
        } else {
            Some("".to_string())
        };
        a.push(SearchResult {
            name: ep3.title,
            id: ep3.id,
            desc: ep3.description,
            img: i,
        });
    }
    Ok(a)
}

#[derive(Default)]
pub struct ViewerContext {
    pub session: Mutex<Option<Crunchyroll>>,
}

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let fullscreen = CustomMenuItem::new("fullscreen".to_string(), "Toggle Fullscreen");
    let devtools = CustomMenuItem::new("devtools".to_string(), "Toggle devtools");
    let submenu = Submenu::new(
        "File",
        Menu::new()
            .add_item(fullscreen)
            .add_item(devtools)
            .add_item(quit),
    );
    let menu = Menu::new().add_submenu(submenu);
    tauri::Builder::default()
        .setup(|app| {
            use tauri::GlobalShortcutManager;
            let app2 = app.app_handle();
            app2.app_handle()
                .global_shortcut_manager()
                .register("F11", move || {
                    let decorated = app2.get_window("main").unwrap().is_decorated().unwrap();
                    app2.get_window("main")
                        .unwrap()
                        .set_decorations(!decorated)
                        .unwrap();
                    app2.get_window("main")
                        .unwrap()
                        .set_fullscreen(decorated)
                        .unwrap();
                })?;
            Ok(())
        })
        .manage(ViewerContext {
            session: Default::default(),
        })
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            "quit" => {
                std::process::exit(0);
            }
            "fullscreen" => {
                if event.window().is_fullscreen().unwrap() {
                    event.window().set_fullscreen(false).unwrap();
                } else {
                    event.window().set_fullscreen(true).unwrap();
                }
            }
            "devtools" => {
                if event.window().is_devtools_open() {
                    event.window().close_devtools();
                } else {
                    event.window().open_devtools();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            login,
            login_anon,
            get_seasons,
            get_episodes,
            view_episode,
            search_crunchy,
            get_recs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
