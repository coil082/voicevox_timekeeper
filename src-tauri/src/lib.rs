// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{AppHandle, Emitter, Manager,State};

use chrono::{Local,Timelike};
use tokio::time::{interval,Duration};

use voicevox_core::{blocking::{Onnxruntime,OpenJtalk,Synthesizer,VoiceModelFile},CharacterMeta,StyleMeta};
use const_format::concatcp;
use std::{fs, io::{BufReader, Write as _}, sync::Mutex,path::PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize)]
//state用構造体、設定だけに限らない
struct AppState{
    read_text:Mutex<String>,
    time_duration:Mutex<u32>,
    duration_unit:Mutex<String>,
    setting_path: PathBuf
}
//jsonファイル保存用構造体、設定全部
#[derive(Debug,Serialize,Deserialize)]
struct Setting{
    read_text:String,
    time_duration:u32,
    duration_unit:String,
}

#[tauri::command]
fn setting(read_text:String,time_duration:u32,duration_unit:String,state:State<'_,AppState>){
    println!("setting_path: {:?}", state.setting_path);
    let mut vec:Vec<Setting> = Vec::new();
    vec.push(Setting{
        read_text:read_text.clone(),
        time_duration:time_duration.clone(),
        duration_unit:duration_unit.clone() ,
    });
    let serialized:String = serde_json::to_string(&vec).unwrap();
    fs::write(&state.setting_path.join("setting.json"),serialized.as_bytes()).unwrap();
    println!("{0}{1}{2}",read_text.as_str(),time_duration,duration_unit);
    *state.read_text.lock().unwrap() = read_text;
    *state.time_duration.lock().unwrap() = time_duration;
    *state.duration_unit.lock().unwrap() = duration_unit;
}

fn tts(state:&AppState,synth:&Synthesizer<OpenJtalk>){
    
    const TARGET_CHARACTER_NAME: &str = "ずんだもん";
    const TARGET_STYLE_NAME: &str = "ノーマル";
    let text =state.read_text.lock().unwrap();
    let StyleMeta{id:style_id,..}=synth
                .metas()
                .into_iter()
                .filter(|CharacterMeta{name,..}| name == TARGET_CHARACTER_NAME)
                .flat_map(|CharacterMeta{styles,..}| styles)
                .find(|StyleMeta{name,..}| name == TARGET_STYLE_NAME)
                .unwrap();

    eprintln!("Synthesizing");
    let wav = &synth.tts(text.as_str(),style_id).perform().unwrap();
    eprintln!("Playing the WAV");
    play(wav).unwrap();
    fn play(wav: &[u8]) -> anyhow::Result<()>{
        let tempfile = tempfile::Builder::new().suffix(".wav").tempfile().unwrap();
        (&tempfile).write_all(wav).unwrap();
        let tempfile = &tempfile.into_temp_path();
        println!("WAV:{:?}",tempfile);
        let sink_handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let file = BufReader::new(fs::File::open(&tempfile).unwrap());
        let _player = rodio::play(&sink_handle.mixer(),file).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(5));
        Ok(())
    }
}

async fn time_check(app: &AppHandle,synth:&Synthesizer<OpenJtalk>,state:&AppState) {
    let mut checker = interval(Duration::from_secs(1));
    let mut pre_time : Option<u32>= None;
    loop {
        checker.tick().await;
        let now = Local::now();
        let time = match state.duration_unit.lock().unwrap().as_str(){
            "hour" => now.hour(),
            "minute" => now.minute(),
            "second" => now.second(),
            _ => panic!("no value")
        };
        let duration = *state.time_duration.lock().unwrap();
        if let Some(pre) = pre_time{
            if time % duration == 0 && time!=pre{
                app.emit("1hour","一時間たった").unwrap();
                tts(state,synth);
            }
        }
        pre_time=Some(time);
        println!("Current time: {}", now.format("%Y-%m-%d %H:%M:%S"));
    }
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            const VVORT: &str = concatcp!("./voicevox_core/onnxruntime/lib/",Onnxruntime::LIB_RECOMMENDED_VERSIONED_FILENAME);
            const OJT_DIC: &str = "./voicevox_core/dict/open_jtalk_dic_utf_8-1.11";
            const VVM: &str = "./voicevox_core/models/vvms/0.vvm";
            fn synth ()->Synthesizer<OpenJtalk> {
                println!("cwd = {:?}", std::env::current_dir());
                println!("VVORT = {}", VVORT);
                println!("exists = {}", std::path::Path::new(VVORT).exists());
                let ort = Onnxruntime::load_once().filename(VVORT).perform().unwrap();
                let ojt = OpenJtalk::new(OJT_DIC).unwrap();
                let synth = Synthesizer::builder(ort).text_analyzer(ojt).build().unwrap();
                synth
            }
            let synth = synth();
            synth
                .load_voice_model(&VoiceModelFile::open(VVM).unwrap())
                .perform().unwrap();
            let app_data_dir = app.path().app_data_dir()?;
            println!("setting path: {:?}", app_data_dir.join("setting.json"));
            let input_setting:AppState = match fs::read_to_string(&app_data_dir.join("setting.json")){
                Ok(text)=>{
                    let deserialized: Vec<Setting> = serde_json::from_str(&text)?;
                    println!("{deserialized:?}");
                    let new_state = AppState{
                        read_text:Mutex::new(deserialized[0].read_text.clone()),
                        time_duration:Mutex::new(deserialized[0].time_duration.clone()),
                        duration_unit:Mutex::new(deserialized[0].duration_unit.clone()),
                        setting_path:app_data_dir.clone()
                    };
                    new_state
                }
                Err(e)=>{
                    if e.kind() != std::io::ErrorKind::NotFound{
                        println!("{e:?}");
                        panic!("error");
                    }
                    let new_state = AppState{
                        read_text:Mutex::new("一時間".to_owned()),
                        time_duration:Mutex::new(1),
                        duration_unit:Mutex::new("hour".to_owned()),
                        setting_path:app_data_dir.clone()
                    };
                    let new_setting = Setting{
                        read_text:new_state.read_text.lock().unwrap().clone(),
                        duration_unit:new_state.duration_unit.lock().unwrap().clone(),
                        time_duration:*new_state.time_duration.lock().unwrap()
                    };
                    let mut vec:Vec<&Setting> = Vec::new();
                    vec.push(&new_setting);
                    let  serialized:String = serde_json::to_string(&vec)?;
                    std::fs::create_dir_all(&app_data_dir)?;
                    fs::write(&app_data_dir.join("setting.json"),serialized.as_bytes())?;
                    new_state
                }
            };
            app.manage(input_setting);
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let input_setting= app_handle.state::<AppState>();
                time_check(&app_handle,&synth,&input_setting).await;
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![setting])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
