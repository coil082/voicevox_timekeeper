// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{AppHandle,Emitter};

use chrono::{Local,Timelike};
use tokio::time::{interval,Duration};

use voicevox_core::{blocking::{Onnxruntime,OpenJtalk,Synthesizer,VoiceModelFile},CharacterMeta,StyleMeta};
use const_format::concatcp;
use std::{io::Write as _, };


fn tts(){
    const VVORT: &str = concatcp!("./voicevox_core/onnxruntime/lib/",Onnxruntime::LIB_RECOMMENDED_VERSIONED_FILENAME);
    const OJT_DIC: &str = "./voicevox_core/dict/open_jtalk_dic_utf_8-1.11";
    const VVM: &str = "./voicevox_core/models/vvms/0.vvm";
    const TARGET_CHARACTER_NAME: &str = "ずんだもん";
    const TARGET_STYLE_NAME: &str = "ノーマル";
    const TEXT: &str = "こんにちは";
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
    let StyleMeta{id:style_id,..}=synth
        .metas()
        .into_iter()
        .filter(|CharacterMeta{name,..}| name == TARGET_CHARACTER_NAME)
        .flat_map(|CharacterMeta{styles,..}| styles)
        .find(|StyleMeta{name,..}| name == TARGET_STYLE_NAME)
        .unwrap();
    eprintln!("Synthesizing");
    let wav = &synth.tts(TEXT,style_id).perform().unwrap();
    eprintln!("Playing the WAV");
    play(wav).unwrap();
    fn play(wav: &[u8]) -> anyhow::Result<()>{
        let tempfile = tempfile::Builder::new().suffix(".wav").tempfile().unwrap();
        (&tempfile).write_all(wav).unwrap();
        let tempfile = &tempfile.into_temp_path();
        println!("WAV:{:?}",tempfile);
        open::that(&tempfile).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(5));
        Ok(())
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

async fn time_check(app: AppHandle) {
    let mut checker = interval(Duration::from_secs(1));
    loop {
        checker.tick().await;
        let now = Local::now();
        let minute = now.second();
        if minute == 0 {
            app.emit("1hour","一時間たった").unwrap();
            tts();
        }
        println!("Current time: {}", now.format("%Y-%m-%d %H:%M:%S"));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                time_check(app_handle).await;
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
