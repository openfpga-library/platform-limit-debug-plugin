use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Clone, Serialize, ToBytes)]
#[encoding(Json)]
enum PluginMessage {
    Choice {
        name: String,
        query: String,
        choices: Vec<String>,
    },
    Text {
        name: String,
        query: String,
    },
    Exit,
}

#[host_fn]
unsafe extern "ExtismHost" {
    fn open_url(url: &str) -> ();
    fn print_msg(message: &str) -> ();
}

fn println(msg: &str) -> () {
    unsafe {
        let _ = print_msg(&format!("{msg}\n"));
    };
}

#[derive(Deserialize)]
struct PlatformFile {
    platform: PlatformData,
}

#[derive(Deserialize, Default)]
struct PlatformData {
    name: String,
    manufacturer: String,
    category: Option<String>,
    year: u16,
}

#[derive(Deserialize)]
struct CoreFile {
    core: CoreMetadata,
}

#[derive(Deserialize)]
struct CoreMetadata {
    metadata: CoreMeta,
}

#[derive(Deserialize, Default)]
struct CoreMeta {
    #[serde(default)]
    platform_ids: Vec<String>,
}

#[plugin_fn]
pub fn start() -> FnResult<PluginMessage> {
    let mut num_platforms = 0;
    let mut total_bytes = 0;
    let mut name_len = 0;
    let mut mfg_len = 0;
    let mut ctg_len = 0;

    if let Ok(entries) = fs::read_dir("pocket/Platforms") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                num_platforms += 1;

                if let Ok(meta) = fs::metadata(&path) {
                    total_bytes += meta.len();
                }

                if let Ok(json_str) = fs::read_to_string(&path) {
                    if let Ok(pf) = serde_json::from_str::<PlatformFile>(&json_str) {
                        name_len += pf.platform.name.len();
                        mfg_len += pf.platform.manufacturer.len();
                        ctg_len += pf.platform.category.unwrap_or_default().len();
                    }
                }
            }
        }
    }

    let mut cores_represented = 0;
    if let Ok(entries) = fs::read_dir("pocket/Cores") {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let core_path = entry.path().join("core.json");
                if let Ok(json_str) = fs::read_to_string(core_path) {
                    if let Ok(cf) = serde_json::from_str::<CoreFile>(&json_str) {
                        cores_represented += cf.core.metadata.platform_ids.len();
                    }
                }
            }
        }
    }

    let output = format!(
        "Platforms: {} | Bytes: {} | NameLen: {} | MfgLen: {} | CatLen: {} | CoresRep: {}",
        num_platforms, total_bytes, name_len, mfg_len, ctg_len, cores_represented
    );

    println(&output);

    Ok(PluginMessage::Exit)
}
