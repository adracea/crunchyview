use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::time::Duration;
use std::vec;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::HtmlElement;
use web_sys::HtmlInputElement;
use web_sys::HtmlTrackElement;
use web_sys::HtmlVideoElement;
use web_sys::TextTrackMode;
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::prelude::*;

const FIVE_SEC: Duration = Duration::from_secs(5);
const ONE_SEC: Duration = Duration::from_secs(1);
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"],js_name = invoke,catch)]
    async fn invoke_checked(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"],js_name = invoke,catch)]
    async fn invoke_checked_no_arg(cmd: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = console,js_name= log)]
    fn log_obj(s: JsValue);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub async fn second_tick() {
    sleep(ONE_SEC).await;
}
pub async fn tick_x(x: f64) {
    if x < 1_f64 {
        sleep(Duration::from_millis((x * 1000_f64) as u64)).await;
    } else {
        sleep(Duration::from_secs_f64(x)).await;
    }
}

pub async fn initialize_atomic_clocks() {
    sleep(FIVE_SEC).await;
}

#[derive(Serialize, Deserialize)]
struct LoginArgs<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Properties)]
#[serde(rename_all = "camelCase")]
struct SeriesName<'a> {
    series_name: &'a str,
}

#[derive(PartialEq, Properties)]
pub struct LoginProps {
    pub username: String,
    pub password: String,
    pub state: bool,
    pub on_logged_in: Callback<(), ()>,
}

#[derive(Serialize, Debug, Deserialize, Default, Clone, PartialEq, Eq, Properties)]
pub struct SearchResult {
    pub name: String,
    pub id: String,
    pub desc: String,
    pub img: Option<String>,
}
#[function_component]
pub fn Login(props: &LoginProps) -> Html {
    let LoginProps {
        username,
        password,
        state,
        on_logged_in,
    } = props;
    let login_input_ref = use_node_ref();
    let login_input_ref2 = use_node_ref();
    let state = use_state(|| *state);
    let hidenotif = use_state(|| true);
    let username = use_state(|| username.clone());
    let password = use_state(|| password.clone());
    let login = {
        let username = username.clone();
        let password = password.clone();
        let login_input_ref = login_input_ref.clone();
        let login_input_ref2 = login_input_ref2.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                if let Some(input) = login_input_ref.cast::<HtmlInputElement>() {
                    username.set(input.value());
                };
                if let Some(input) = login_input_ref2.cast::<HtmlInputElement>() {
                    password.set(input.value());
                };
            }
        })
    };
    let login_msg = use_state(String::new);
    {
        let login_msg = login_msg.clone();
        let on_logged_in = on_logged_in.clone();
        let username = username;
        let username2 = username.clone();
        let password = password;
        let hidenotif = hidenotif.clone();
        let state = state.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    if !(username.is_empty() && password.is_empty()) {
                        let new_msg: Result<JsValue, JsValue> = if password.is_empty() {
                            invoke_checked_no_arg("login_anon").await
                        } else {
                            // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
                            invoke_checked(
                                "login",
                                to_value(&LoginArgs {
                                    username: &username,
                                    password: &password,
                                })
                                .unwrap(),
                            )
                            .await
                        };
                        let b = match new_msg {
                            Ok(c) => Some(c),
                            Err(c) => {
                                log(&c.as_string().unwrap());
                                None
                            }
                        };
                        if let Some(bs) = b {
                            let a: Result<String, serde_wasm_bindgen::Error> =
                                serde_wasm_bindgen::from_value(bs);
                            match a {
                                Ok(mess) => {
                                    log(&mess);
                                    login_msg.set(mess);
                                    state.set(true);
                                    hidenotif.set(false);
                                    on_logged_in.emit(());
                                    initialize_atomic_clocks().await;
                                    hidenotif.set(true);
                                }
                                Err(e) => log(&format!("{e}")),
                            }
                        }
                    }
                });

                || {}
            },
            username2,
        );
    }
    html! {<>
                        if *state{}else{
                                <form class="login" onkeypress={login} action="javascript:void(0);">
                                  <input class="login-input" autocomplete="username" ref={login_input_ref} type="username" placeholder="Enter a username..." />


                                <input class="login-input" autocomplete="password" type="password" ref={login_input_ref2}
                                    placeholder="Enter a password..." />
                                  <button type="submit" class="loginbtn" >{"Log In"}</button>
                               </form>

                        }
                    if *hidenotif{
                    } else {
                        <p id="loginotif"><b>{ &*login_msg }</b></p>}
                        </>
    }
}

#[derive(Serialize, Debug, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EpOrSeries {
    pub series_id: Option<String>,
    pub ep_id: Option<String>,
}

#[function_component]
pub fn Episodes(props: &SearchResult) -> Html {
    let SearchResult { id, .. } = props;
    let search_result: UseStateHandle<Vec<SearchResult>> = use_state(Vec::new);
    {
        let search_result = search_result.clone();
        let series_name = id.clone();
        let series_name2 = id.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    if series_name.is_empty() {
                        return;
                    }

                    let new_msg = invoke_checked(
                        "get_episodes",
                        to_value(&EpOrSeries {
                            series_id: Some(series_name),
                            ep_id: Some("".to_string()),
                        })
                        .unwrap(),
                    )
                    .await;
                    let b = match new_msg {
                        Ok(c) => Some(c),
                        Err(c) => {
                            log(&c.as_string().unwrap());
                            None
                        }
                    };
                    if let Some(bs) = b {
                        let a: Result<Vec<SearchResult>, serde_wasm_bindgen::Error> =
                            serde_wasm_bindgen::from_value(bs);
                        match a {
                            Ok(mess) => {
                                for i in (mess).iter() {
                                    log(&i.name);
                                }
                                search_result.set(mess);
                            }
                            Err(e) => log(&format!("{e}")),
                        }
                    }
                });
            },
            series_name2,
        );
    }
    let selected_episode: UseStateHandle<String> = use_state(String::new);

    let scoped_sr = selected_episode;
    let curr_sr = use_state(|| false);
    let curr_sr2 = curr_sr.clone();
    if *curr_sr2 {
        curr_sr2.set(false);
        return html! {<ViewEp ep_id={scoped_sr.to_string()} ep_type={"main".to_string()} cb={
        {
            let scoped_sr2 = scoped_sr;
            Callback::from(move |name: String| {
                curr_sr.set(true);
                scoped_sr2.set(name)
            })
        }}/>};
    }
    let mut fin = vec![html!()];
    let srs = search_result;
    if scoped_sr.is_empty() {
        for i in srs.iter() {
            let y = i.clone();
            fin.push(html! {<div onclick={
            let y2=y.clone();
                let scoped_sr2=scoped_sr.clone();
                Callback::from(move |_| {
                    let y3=y2.clone();
                    let scoped_sr=scoped_sr2.clone();
                    scoped_sr.set(y3.id.clone().to_owned());
                })
            }
                        id={y.clone().name}
                        class="episode">
                        if y.clone().img.is_some()
                            { <b class="episodeName" style={"background: url(".to_owned()+y.clone().img.unwrap().split(' ').collect::<Vec<&str>>().first().unwrap().to_string().as_str()+")"}>{y.clone().name} </b>}
                            else{<b>{y.clone().name} </b>}
                        </div>})
        }
    }
    let a = html! {
        if scoped_sr.is_empty() {
            {fin}
        }
        else{
            <ViewEp ep_id={scoped_sr.to_string()} ep_type={"main".to_string()} cb={
                {
                    let scoped_sr2 = scoped_sr;
                    Callback::from(move |name: String| {
                        curr_sr.set(true);
                        scoped_sr2.set(name)
                    })
                }}/>
        }
    };
    a
}
#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, Eq, Properties)]
pub struct EpisodeViewProps {
    title: String,
    id: String,
    number: i32,
    desc: String,
    subs: HashMap<String, String>,
    url: String,
    nep: Option<Vec<Ep>>,
}

impl Default for EpisodeViewProps {
    fn default() -> Self {
        EpisodeViewProps {
            title: String::new(),
            id: String::new(),
            number: 0,
            desc: String::new(),
            subs: HashMap::new(),
            url: String::new(),
            nep: Some(vec![Ep::default(); 0]),
        }
    }
}

#[derive(Serialize, Debug, Deserialize, Default, Clone, PartialEq, Eq, Properties)]
#[serde(rename_all = "snake_case")]
pub struct Ep {
    ep_id: String,
    ep_type: String,
}

#[derive(Debug, Default, Clone, PartialEq, Properties)]
pub struct Nep {
    ep_id: String,
    ep_type: String,
    cb: Callback<String>,
}

#[wasm_bindgen(module = "/public/dist/hls.esm.js")]
extern "C" {
    #[wasm_bindgen(js_name=default)]
    type Hls;
    #[wasm_bindgen(constructor,js_class=default)]
    fn new() -> Hls;
    #[wasm_bindgen(method)]
    fn attachMedia(this: &Hls, element: JsValue);
    #[wasm_bindgen(method)]
    fn loadSource(this: &Hls, source: JsValue);
    #[wasm_bindgen(method)]
    fn destroy(this: &Hls);
    #[wasm_bindgen(method, getter)]
    fn levels(this: &Hls) -> JsValue;
    #[wasm_bindgen(method, getter)]
    fn loadLevel(this: &Hls) -> i32;
    #[wasm_bindgen(method, setter)]
    fn set_loadLevel(this: &Hls, level: i32);
    #[wasm_bindgen(method, getter)]
    fn currentLevel(this: &Hls) -> JsValue;
    #[wasm_bindgen(method, setter)]
    fn set_currentLevel(this: &Hls, level: i32);

}
impl TearDown for Hls {
    fn tear_down(self) {
        self.destroy()
    }
}

impl Clone for Hls {
    fn clone(&self) -> Self {
        Self {
            obj: self.obj.clone(),
        }
    }
}
impl Debug for Hls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hls").field("obj", &self.obj).finish()
    }
}

impl Hls {
    pub fn set_level(self, level: i32) -> Hls {
        self.set_currentLevel(level);
        self
    }
    pub fn init(self, url: String) -> Hls {
        self.attachMedia(JsValue::from(
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("mainvideo")
                .unwrap(),
        ));
        self.loadSource(JsValue::from(url));
        self
    }
}

#[function_component]
pub fn ViewEp(props: &Nep) -> Html {
    let Nep {
        ep_id,
        ep_type: _,
        cb,
    } = props;
    let is_loading = use_state(|| true);
    let current_view_ep = use_state(|| ep_id.to_string());
    let search_result: UseStateHandle<EpisodeViewProps> = use_state(|| EpisodeViewProps {
        ..Default::default()
    });
    let hls: UseStateHandle<Hls> = use_state(Hls::new);
    {
        let search_result2 = search_result.clone();
        let series_name = current_view_ep;
        let is_loaded = is_loading.clone();
        let hls2 = hls.clone();
        use_effect_with_deps(
            move |series_name| {
                let sr = series_name.clone();
                spawn_local(async move {
                    if (sr).is_empty() {
                        return;
                    }
                    let new_msg = invoke_checked(
                        "view_episode",
                        to_value(&Ep {
                            ep_id: (sr).to_string(),
                            ep_type: "main".to_string(),
                        })
                        .unwrap(),
                    )
                    .await;
                    let b = match new_msg {
                        Ok(c) => Some(c),
                        Err(c) => {
                            log(&c.as_string().unwrap());
                            None
                        }
                    };
                    if let Some(bs) = b {
                        let a: Result<EpisodeViewProps, serde_wasm_bindgen::Error> =
                            serde_wasm_bindgen::from_value(bs);
                        match a {
                            Ok(mess) => {
                                log(mess.id.as_str());
                                search_result2.set(mess);
                                is_loaded.set(false);
                            }
                            Err(e) => log(&format!("{e}")),
                        }
                    }
                });
                move || hls2.destroy()
            },
            series_name,
        );
    }
    let mut b = vec![html! {}];
    let mut c = vec![html! {}];
    let nep = search_result.nep.clone().unwrap();
    for (_i, j) in nep.iter().enumerate() {
        let j4 = j.clone();
        c.push(html! {<NextEps ep_id={j4.ep_id} ep_type={j4.ep_type} cb={cb.clone()}/>});
    }
    for (iterar, item) in search_result.subs.iter().enumerate() {
        b.push(html! {<track id={"sub-".to_owned()+&iterar.to_string()}
        label={item.0.to_string()}
        kind="subtitles"
        srclang={item.0.to_string()}
        src={item.1.to_string()} type="text/plain" />});
    }
    let tempaa = use_state(|| hls.loadLevel().to_string());
    let levels = use_state(Levs::default);
    {
        let is_loading2 = is_loading.clone();
        let levels = levels.clone();
        let hls2 = hls.clone();
        use_effect_with_deps(
            move |is_loading2| {
                let hls3 = hls2.clone();
                let levels2 = levels.clone();
                let hls4 = hls2.clone();
                if !(**is_loading2) {
                    spawn_local(async move {
                        let hls5: &Hls = &hls4;
                        let sr = &search_result.url.clone();
                        let hls_inter = hls5.to_owned().init(sr.to_string());
                        hls_inter.set_loadLevel(0);
                        second_tick().await;
                        second_tick().await;
                        second_tick().await;
                        log_obj(hls_inter.levels());
                        let a: Result<Levs, serde_wasm_bindgen::Error> =
                            serde_wasm_bindgen::from_value(hls_inter.levels());
                        let b = a.unwrap();
                        levels2.set(b.clone());
                        log(b.0.len().to_string().as_str());
                        hls3.set(hls_inter.to_owned());
                    });
                }
            },
            is_loading2,
        );
    }
    {
        let tempa = tempaa.clone();
        let hls = hls.clone();
        let tempa2 = tempaa.clone();
        use_effect_with_deps(
            move |_| {
                let hls5: &Hls = &hls;
                let hls2 = hls.clone();
                hls2.set(hls5.clone().set_level(tempa.parse::<i32>().unwrap()));
            },
            tempa2,
        );
    }
    let mut level_list = vec![html!()];
    for i in 0..((levels).0.len() as i32) {
        let ival = i;
        let tempa = tempaa.clone();
        let hls3 = hls.clone();
        level_list.push(html! {<a onclick={Callback::from(move|_|{
            let hls5: &Hls = &hls3;
        let hls2 =hls3.clone();
        tempa.set(ival.to_string());
        hls2.set(hls5.clone().set_level(tempa.parse::<i32>().unwrap()));})} class="quality">{levels.0.get(i as usize).unwrap()}</a>});
    }

    let onplaypause = Callback::from(move |_| {
        let a = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("mainvideo")
            .unwrap()
            .dyn_into::<HtmlVideoElement>()
            .unwrap();
        if a.paused() {
            spawn_local(async move {
                _ = a.play().unwrap();
            });
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("playpause-btn")
                .unwrap()
                .set_class_name("pause-button");
        } else {
            a.pause().unwrap();
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("playpause-btn")
                .unwrap()
                .set_class_name("play-button");
        }
    });
    let a = html! {
        <div id="ep_view"><div id="videomain"><video id="mainvideo" class="mainvideo" controls={false} onclick={onplaypause.clone()} crossorigin="credentials">
        {b}

    </video>
    <Controls lev_list={level_list} loading={*is_loading} playpausecb={onplaypause.clone()} progress={0_f64}/>

    </div>
    <div class="nextepsGrid"><b>{"Next Episodes :"}</b>{c}</div>
    </div>
    };
    a
}

#[derive(PartialEq, Properties)]
pub struct ControlsProps {
    lev_list: Vec<yew::virtual_dom::VNode>,
    loading: bool,
    playpausecb: Callback<MouseEvent>,
    progress: f64,
}

#[function_component]
pub fn Controls(props: &ControlsProps) -> Html {
    let dur = use_state(String::new);
    let dur2 = dur.clone();
    let dtrig = use_state(video_progress);
    let dtrig2 = dtrig;
    let b2 = dur2;
    let progress = use_state(|| props.progress);
    let prog2 = progress.clone();
    let buffprog = use_state(|| 0_f64);
    let buffprog2 = buffprog.clone();
    let volume = use_state(|| 1_f64);
    let vol2 = volume.clone();
    let onvaluechanged = Callback::from(move |ev: InputEvent| {
        vol2.set(
            ev.target_dyn_into::<web_sys::HtmlInputElement>()
                .unwrap()
                .value()
                .parse::<f64>()
                .unwrap(),
        );
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("mainvideo")
            .unwrap()
            .dyn_into::<HtmlVideoElement>()
            .unwrap()
            .set_volume(
                ev.target_dyn_into::<web_sys::HtmlInputElement>()
                    .unwrap()
                    .value()
                    .parse::<f64>()
                    .unwrap(),
            )
    });
    let progresscb = Callback::from(move |_| {
        dtrig2.set(
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("mainvideo")
                .unwrap()
                .dyn_into::<HtmlVideoElement>()
                .unwrap()
                .current_time(),
        );
        let a = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("mainvideo")
            .unwrap()
            .dyn_into::<HtmlVideoElement>()
            .unwrap();
        let ct = a.current_time();
        if ct.is_nan() {
            b2.set("".to_string());
        } else if ct < 1_f64 {
            prog2.set(video_progress());
            buffprog2.set(video_buffered());
            b2.set(format!(
                "{:0>2}:{:0>2}/{:0>2}:{:0>2}",
                (Duration::from_millis((a.current_time() * 1000_f64) as u64)
                    .as_secs()
                    .checked_div(60)
                    .unwrap_or(0)),
                Duration::from_millis((a.current_time() * 1000_f64) as u64).as_secs()
                    - (Duration::from_millis((a.current_time() * 1000_f64) as u64)
                        .as_secs()
                        .checked_div(60)
                        .unwrap_or(0)
                        * 60),
                (Duration::from_secs_f64(if a.duration().is_nan() {
                    0_f64
                } else {
                    a.duration()
                })
                .as_secs()
                .checked_div(60)
                .unwrap_or(0)),
                Duration::from_secs_f64(if a.duration().is_nan() {
                    0_f64
                } else {
                    a.duration()
                })
                .as_secs()
                    - (Duration::from_secs_f64(if a.duration().is_nan() {
                        0_f64
                    } else {
                        a.duration()
                    })
                    .as_secs()
                    .checked_div(60)
                    .unwrap_or(0)
                        * 60)
            ));
        } else {
            prog2.set(video_progress());
            buffprog2.set(video_buffered());
            b2.set(format!(
                "{:0>2}:{:0>2}/{:0>2}:{:0>2}",
                (Duration::from_secs_f64(a.current_time())
                    .as_secs()
                    .checked_div(60)
                    .unwrap_or(0)),
                Duration::from_secs_f64(a.current_time()).as_secs()
                    - (Duration::from_secs_f64(a.current_time())
                        .as_secs()
                        .checked_div(60)
                        .unwrap_or(0)
                        * 60),
                (Duration::from_secs_f64(a.duration())
                    .as_secs()
                    .checked_div(60)
                    .unwrap_or(0)),
                Duration::from_secs_f64(a.duration()).as_secs()
                    - (Duration::from_secs_f64(a.duration())
                        .as_secs()
                        .checked_div(60)
                        .unwrap_or(0)
                        * 60)
            ));
        }
    });
    let progcb3 = progresscb;
    use_effect(move || {
        let progcb2 = progcb3;
        spawn_local(async move {
            tick_x(0.1_f64).await;
            progcb2.emit(());
        });
    });
    let fsstate = use_state(|| false);
    let onclickfs = Callback::from(move |_| {
        if !*fsstate {
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("videomain")
                .unwrap()
                .request_fullscreen()
                .unwrap();
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("mainvideo")
                .unwrap()
                .set_class_name("mainvideofs");
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("fs-btn")
                .unwrap()
                .set_class_name("fs-button-full");
            fsstate.set(true);
        } else {
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .exit_fullscreen();
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("mainvideo")
                .unwrap()
                .set_class_name("mainvideo");
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("fs-btn")
                .unwrap()
                .set_class_name("fs-button");
            fsstate.set(false);
        }
    });
    let onclickscrub = Callback::from(move |e: MouseEvent| {
        let a = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("mainvideo")
            .unwrap()
            .dyn_into::<HtmlVideoElement>()
            .unwrap();
        a.set_current_time(
            (e.offset_x() as f64
                / web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .get_element_by_id("progress")
                    .unwrap()
                    .dyn_into::<HtmlElement>()
                    .unwrap()
                    .offset_width() as f64)
                * a.duration(),
        );
    });
    let active_track = use_state(|| 0_u32);
    let as_set = use_state(|| false);
    let tracklist = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("mainvideo")
        .unwrap()
        .dyn_into::<HtmlVideoElement>()
        .unwrap()
        .text_tracks()
        .unwrap();
    let active_trackselect = active_track;
    let settrack = Callback::from(move |e: MouseEvent| {
        let target = e.target_dyn_into::<HtmlElement>().unwrap();
        if *as_set {
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id(&("sub-".to_owned() + &(*active_trackselect).to_string()))
                .unwrap()
                .dyn_into::<HtmlTrackElement>()
                .unwrap()
                .track()
                .unwrap()
                .set_mode(TextTrackMode::Hidden);
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id(&("sub-".to_owned() + &target.id()))
                .unwrap()
                .dyn_into::<HtmlTrackElement>()
                .unwrap()
                .track()
                .unwrap()
                .set_mode(TextTrackMode::Showing);
            active_trackselect.set(target.id().parse::<u32>().unwrap());
        } else {
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id(&("sub-".to_owned() + &target.id()))
                .unwrap()
                .dyn_into::<HtmlTrackElement>()
                .unwrap()
                .track()
                .unwrap()
                .set_mode(TextTrackMode::Showing);
            active_trackselect.set(target.id().parse::<u32>().unwrap());
            as_set.set(true);
        }
    });
    let mut tracks = vec![html! {}];
    for i in 0..tracklist.length() {
        tracks.push(html!{<a onclick={settrack.clone()} id={i.to_string()} class="subtitle">{tracklist.get(i).unwrap().label()}</a>});
    }
    let a = html! {<>
        <div id="video-controls" class="controls display-control">
              <button id="quality-btn" class="dropbtn btn-settings">
              <div id="quality" class="dropdown-content">{props.lev_list.clone()}</div></button>
              <button id="fs-btn" class="fs-button" onclick={onclickfs}></button>
              <button id="vol-btn" class="dropbtn volume"><input oninput={onvaluechanged} type="range"  min="0" max="1" step="0.01" value={volume.to_string()} class="volume-slider" id="volume"/></button>
              <button id="language-btn" class="dropbtn ccbtn btn-settings"><div id="quality" class="dropdown-content">{tracks}</div></button>
            </div>
        <div id="controls-right" class="controls clr">
        <button id="playpause-btn" class="play-button" onclick={props.playpausecb.clone()}></button>
        <div id="duration" class="duration">{dur.to_string()}</div></div>
        <div class="controls progress-control"><progress id="progress" class="progress" onclick={onclickscrub} value={progress.to_string()} max={"100"}></progress>
        <progress id="progress2" class="progress2" value={buffprog.to_string()} max={"100"}></progress>
    </div>
        </>
    };
    a
}

fn video_progress() -> f64 {
    if let Some(a) = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("mainvideo")
    {
        match a.dyn_into::<HtmlVideoElement>() {
            Ok(a) => {
                if !a.duration().is_nan() {
                    return (a.current_time() / a.duration()) * 100_f64;
                }
            }
            Err(_) => log("err"),
        }
    }
    0_f64
}
fn video_buffered() -> f64 {
    if let Some(a) = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("mainvideo")
    {
        match a.dyn_into::<HtmlVideoElement>() {
            Ok(a) => {
                if !a.buffered().end(0).unwrap_or(f64::NAN).is_nan() {
                    return (a.buffered().end(0).unwrap() / a.duration()) * 100_f64;
                }
            }
            Err(_) => log("err"),
        }
    }
    0_f64
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub struct Lev {
    pub bitrate: i32,
    pub width: i32,
    pub height: i32,
}
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Levs(pub Vec<Lev>);

impl Display for Levs {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.iter().fold(Ok(()), |result, album| {
            result.and_then(|_| writeln!(f, "{album}"))
        })
    }
}
impl Display for Lev {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}p", self.height)
    }
}

#[function_component]
pub fn NextEps(props: &Nep) -> Html {
    let search_result: UseStateHandle<Vec<SearchResult>> = use_state(|| {
        vec![SearchResult {
            ..Default::default()
        }]
    });
    let is_loading = use_state(|| true);
    let is_clicked = use_state(|| false);
    let Nep {
        ep_id,
        ep_type: _,
        cb,
    } = props;
    {
        let ep_id2 = ep_id.clone();
        let is_loading2 = is_loading.clone();
        let search_result2 = search_result.clone();
        let search_result3 = search_result.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    if ep_id2.is_empty() {
                        return;
                    }
                    let new_msg = invoke_checked(
                        "get_episodes",
                        to_value(&EpOrSeries {
                            series_id: None,
                            ep_id: Some(ep_id2),
                        })
                        .unwrap(),
                    )
                    .await;
                    let b = match new_msg {
                        Ok(c) => Some(c),
                        Err(c) => {
                            log(&c.as_string().unwrap());
                            None
                        }
                    };
                    if let Some(bs) = b {
                        let a: Result<Vec<SearchResult>, serde_wasm_bindgen::Error> =
                            serde_wasm_bindgen::from_value(bs);
                        match a {
                            Ok(mess) => {
                                search_result2.set(mess);
                                is_loading2.set(false);
                            }
                            Err(e) => log(&format!("{e}")),
                        }
                    }
                });
            },
            search_result3,
        );
    }
    let search_result3 = search_result;
    let onclick = {
        let is_clk = is_clicked;
        let cb2 = cb.clone();
        let sr2 = ep_id.clone();
        Callback::from(move |_| {
            is_clk.clone().set(true);
            cb2.clone().emit(sr2.clone());
            log(sr2.clone().as_str());
        })
    };
    let is_loaded = is_loading;
    html! {
        if !*is_loaded{
        <b onclick={onclick} class="secondary" style={let sr2 = search_result3.clone();"background: url(".to_owned()+sr2.clone().first().unwrap().img.to_owned().unwrap().split(' ').collect::<Vec<&str>>().first().unwrap().to_string().as_str()+")"}>{{let sr2 = search_result3.clone();sr2.clone().first().unwrap().name.to_owned()} }</b>
        }
    }
}

#[derive(Eq, PartialEq, Properties)]
pub struct SearchProps {
    pub search_string: String,
}

#[function_component]
pub fn Series(props: &SearchResult) -> Html {
    let SearchResult { id, .. } = props;
    let search_result: UseStateHandle<Vec<SearchResult>> = use_state(Vec::new);
    {
        let search_result = search_result.clone();
        let series_name = id.clone();
        let series_name2 = id.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    if series_name.is_empty() {
                        return;
                    }
                    let new_msg = invoke_checked(
                        "get_seasons",
                        to_value(&SeriesName {
                            series_name: &series_name,
                        })
                        .unwrap(),
                    )
                    .await;
                    let b = match new_msg {
                        Ok(c) => Some(c),
                        Err(c) => {
                            log(&c.as_string().unwrap());
                            None
                        }
                    };
                    if let Some(bs) = b {
                        let a: Result<Vec<SearchResult>, serde_wasm_bindgen::Error> =
                            serde_wasm_bindgen::from_value(bs);
                        match a {
                            Ok(mess) => {
                                for i in (mess).iter() {
                                    log(&i.name);
                                }
                                search_result.set(mess);
                            }
                            Err(e) => log(&format!("{e}")),
                        }
                    }
                });
            },
            series_name2,
        );
    }
    let selected_episode = use_state(|| (String::from(""), String::from("")));

    let scoped_sr = selected_episode;
    let mut a = vec![html! {}];
    let srs = search_result;

    if scoped_sr.1.is_empty() {
        for i in srs.iter() {
            let y = i.clone();
            a.push(html! {<div onclick={
                let y2=y.clone();
                    let scoped_sr2=scoped_sr.clone();
                    Callback::from(move |_| {
                        let y3=y2.clone();
                        let scoped_sr=scoped_sr2.clone();
                        let res = (y3.id.clone().to_owned(),y3.name.clone().to_owned());
                        // log(&res.0);
                        scoped_sr.set(res);
                    })
                }
                        id={y.clone().name}
                        class="season">
                        if y.clone().img.is_some()
                            { <b class="seasonName" style={"background: url(".to_owned()+y.clone().img.unwrap().split(' ').collect::<Vec<&str>>().first().unwrap().to_string().as_str()+")"}>{y.clone().name} </b>}
                            else{<b>{y.clone().name} </b>}
                        </div>})
        }
    } else {
        a.push(
            html! {<Episodes name={let sr = scoped_sr.clone();sr.1.to_owned()} id={let sr = scoped_sr;sr.0.to_owned()} desc={format!("")}/>}
        )
    };
    a.into_iter().collect::<Html>()
}

#[function_component]
pub fn Search(props: &SearchProps) -> Html {
    let SearchProps { search_string } = props;
    let search_input_ref = use_node_ref();
    let selected_series = use_state(|| (String::from(""), String::from("")));

    let series_name = use_state(|| search_string.clone());
    let search_result: UseStateHandle<Vec<SearchResult>> = use_state(Vec::new);
    {
        let search_result = search_result.clone();
        let series_name = series_name.clone();
        let sr = selected_series.clone();
        let series_name2 = series_name.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    let new_msg: Result<JsValue, JsValue> = if series_name.is_empty() {
                        invoke_checked_no_arg("get_recs").await
                    } else {
                        invoke_checked(
                            "search_crunchy",
                            to_value(&SeriesName {
                                series_name: &series_name,
                            })
                            .unwrap(),
                        )
                        .await
                    };
                    let b = match new_msg {
                        Ok(c) => Some(c),
                        Err(c) => {
                            log(&c.as_string().unwrap());
                            None
                        }
                    };
                    if let Some(bs) = b {
                        let a: Result<Vec<SearchResult>, serde_wasm_bindgen::Error> =
                            serde_wasm_bindgen::from_value(bs);
                        match a {
                            Ok(mess) => {
                                search_result.set(mess);
                                sr.set(("".to_string(), "".to_string()));
                            }
                            Err(e) => log(&format!("{e}")),
                        }
                    }
                });
            },
            series_name2,
        );
    }
    let i = series_name.clone();
    let srs = series_name.clone();
    let sn = i.to_string();
    let search = {
        let search_input_ref = search_input_ref.clone();
        let series_name = series_name;
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                if let Some(input) = search_input_ref.cast::<HtmlInputElement>() {
                    series_name.set(input.value());
                }
            }
        })
    };
    let display_state = use_state(|| html!());
    let was_set_back = use_state(|| false);
    let b2: Callback<String> = {
        let display_state_final = display_state.clone();
        let sn = sn;
        let set_back = was_set_back.clone();
        Callback::from({
            move |name: String| {
                if name.contains("search") {
                    set_back.set(true);
                    display_state_final.set(html! { <Search search_string={sn.clone()}/>})
                }
                //  else if name.contains("series") {
                //     // set_back.set(true);
                //     // display_state_final.set(html! {
                //     //     <>
                //     //     <div class="searchrow" onkeypress={search}><button onclick={back} type="button" class="btn">{"Back"}</button>
                //     //     <input id="search-input" ref={search_input_ref} placeholder="Enter a series..." value={let sr = srs.clone();if !sr.clone().to_string().is_empty(){sr.to_string()}else{"".to_string()}}/>
                //     //     <button type="button" class="btn" >{"Search"}</button>
                //     // </div> <div class="seasonview">
                //     // <Series name={let sr = sr.clone();sr.1.to_owned()} id={let sr = sr.clone();sr.0.to_owned()} desc={"".to_string()}/></div></>})
                // }
            }
        })
    };
    let back = {
        Callback::from({
            move |_| {
                b2.emit("search".to_string());
            }
        })
    };
    let scoped_sr = selected_series.clone();
    if *was_set_back {
        let render = &*(display_state);
        return render.clone();
    }
    let second_part = if selected_series.1.is_empty() {
        html! {
         <div id="test"><div class="row" id="search-header"><b><div class="col">{"Name:"}</div></b><div class="col">{"Desc"}</div></div>
        {  (*search_result).iter().enumerate().map(|(x,y)|
            html!{<div class="row" key={x.to_string()}  id={"animetitle".to_owned()+&x.to_string()}><b onclick=
                    {
                        let sr= search_result.clone();
                        let scoped_sr=scoped_sr.clone();
                        Callback::from(move |_| {
                            let sr= sr.clone();
                            let scoped_sr=scoped_sr.clone();
                            let res = (sr.get(x).unwrap().id.clone().to_owned(),sr.get(x).unwrap().name.clone().to_owned());
                            log(&res.0);
                            scoped_sr.set(res);
                        })
                    }><div class="col"  >{y.name.to_owned()+ ":"}</div></b><div class="col" style={"background: url(".to_owned()+y.clone().img.unwrap().split(' ').collect::<Vec<&str>>().first().unwrap().to_string().as_str()+")"}>{y.desc.to_owned()}</div></div>
                }).collect::<Html>()
        }</div>}
    } else {
        html! {<div class="seasonview">
        <Series name={let sr = selected_series.clone();sr.1.to_owned()} id={let sr = selected_series.clone();sr.0.to_owned()} desc={"".to_string()}/></div>}
    };
    let sb = html! {
        <div class="searchrow" onkeypress={search}><button onclick={back} type="button" class="btn">{"Back"}</button>
        <input id="search-input" ref={search_input_ref} placeholder="Enter a series..." value={let sr = srs.clone();if !sr.clone().to_string().is_empty(){sr.to_string()}else{"".to_string()}}/>
        <button type="button" class="btn" >{"Search"}</button>
    </div>};

    html! {<>{sb}
    {second_part}</>
    }
}

#[derive(PartialEq, Properties)]
pub struct BackProps {
    pub location: String,
    pub callback: Callback<MouseEvent>,
}

#[function_component]
pub fn Backbtn(props: &BackProps) -> Html {
    let BackProps {
        location: _,
        callback,
    } = props;
    html! {<button onclick={callback} type="button" class="btn">{"Back"}</button>}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Timer {
    clock: Option<()>,
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_state(|| true);

    // use_effect(move || {
    //     if web_sys::window().unwrap().document().unwrap().onmousemove().unwrap()
    // });
    let maybe_display_link = move || -> Html {
        let state = state.clone();
        let cb: Callback<(), ()> = {
            let state = state.clone();
            Callback::from(move |_| state.set(false))
        };
        html! {<>

                         <Login username={"".to_string()} password={"".to_string()} state={false} on_logged_in={cb}/>
        if *state {} else {

                     <Search search_string={"".to_string()}/>
                 }
                 </>
             }
    };
    html! {
        <main class="container">
        {maybe_display_link()}
        </main>
    }
}
