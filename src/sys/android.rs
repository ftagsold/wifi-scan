use crate::{Error, Result, Wifi};
use jni::objects::{JObject, JString, JValue};
use jni::JNIEnv;

pub fn scan(env: &mut JNIEnv, context: JObject) -> Result<Vec<Wifi>> {
    let wifi_service = env
        .get_static_field(
            "android/content/Context",
            "WIFI_SERVICE",
            "Ljava/lang/String;",
        )
        .map_err(|e| Error::JNIError(e.to_string()))?
        .l()
        .map_err(|e| Error::JNIError(e.to_string()))?;

    let wifi_manager = env
        .call_method(
            context,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[JValue::Object(&wifi_service)],
        )
        .map_err(|e| Error::JNIError(e.to_string()))?
        .l()
        .map_err(|e| Error::JNIError(e.to_string()))?;

    // wifiManager.startScan()
    env.call_method(wifi_manager, "startScan", "()Z", &[])
        .map_err(|e| Error::JNIError(e.to_string()))?;

    // wifiManager.getScanResults()
    let scan_results = env
        .call_method(wifi_manager, "getScanResults", "()Ljava/util/List;", &[])
        .map_err(|e| Error::JNIError(e.to_string()))?
        .l()
        .map_err(|e| Error::JNIError(e.to_string()))?;

    // List.size()
    let size = env
        .call_method(scan_results, "size", "()I", &[])
        .map_err(|e| Error::JNIError(e.to_string()))?
        .i()
        .map_err(|e| Error::JNIError(e.to_string()))?;

    let mut networks = Vec::new();

    for i in 0..size {
        // List.get(i)
        let scan_result = env
            .call_method(
                scan_results,
                "get",
                "(I)Ljava/lang/Object;",
                &[JValue::Int(i)],
            )
            .map_err(|e| Error::JNIError(e.to_string()))?
            .l()
            .map_err(|e| Error::JNIError(e.to_string()))?;

        let ssid: String = env
            .get_field(scan_result, "SSID", "Ljava/lang/String;")
            .map_err(|e| Error::JNIError(e.to_string()))?
            .l()
            .and_then(|s| env.get_string(JString::from(s)))
            .map_err(|e| Error::JNIError(e.to_string()))?
            .into();

        let mac: String = env
            .get_field(scan_result, "BSSID", "Ljava/lang/String;")
            .map_err(|e| Error::JNIError(e.to_string()))?
            .l()
            .and_then(|s| env.get_string(JString::from(s)))
            .map_err(|e| Error::JNIError(e.to_string()))?
            .into();

        let signal_level = env
            .get_field(scan_result, "level", "I")
            .map_err(|e| Error::JNIError(e.to_string()))?
            .i()
            .map_err(|e| Error::JNIError(e.to_string()))?;

        let channel = env
            .get_field(scan_result, "frequency", "I")
            .map_err(|e| Error::JNIError(e.to_string()))?
            .i()
            .map_err(|e| Error::JNIError(e.to_string()))? as u32;

        networks.push(Wifi {
            mac,
            ssid,
            channel,
            signal_level,
            security: vec![]
        });
    }

    Ok(networks)
}