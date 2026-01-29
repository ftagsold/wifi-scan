use crate::{Error, Result, Wifi, WlanScanner};
use jni::objects::{JObject, JString, JValue};
use jni::JNIEnv;
use std::sync::OnceLock;

pub static ANDROID_SCANNER: OnceLock<AndroidScanner> = OnceLock::new();

pub struct AndroidScanner {
    env: JNIEnv<'static>,
    context: JObject<'static>,
}

impl AndroidScanner {
    pub fn init_with_env(env: JNIEnv<'static>, context: JObject<'static>) -> Result<()> {
        ANDROID_SCANNER.get_or_init(AndroidScanner { env, context })?;
        Ok(())
    }
}

impl WlanScanner for AndroidScanner {
    fn scan(&mut self) -> Result<Vec<Wifi>> {
        let scanner = ANDROID_SCANNER
            .get()
            .ok_or_else(|| Error::JNIError("AndroidScanner not initialized".to_string()))?;

        let wifi_service = scanner
            .env
            .get_static_field(
                "android/content/Context",
                "WIFI_SERVICE",
                "Ljava/lang/String;",
            )
            .map_err(|e| Error::JNIError(e.to_string()))?
            .l()
            .map_err(|e| Error::JNIError(e.to_string()))?;

        let wifi_manager = scanner
            .env
            .call_method(
                scanner.context,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&wifi_service)],
            )
            .map_err(|e| Error::JNIError(e.to_string()))?
            .l()
            .map_err(|e| Error::JNIError(e.to_string()))?;

        // wifiManager.startScan()
        scanner
            .env
            .call_method(wifi_manager, "startScan", "()Z", &[])
            .map_err(|e| Error::JNIError(e.to_string()))?;

        // wifiManager.getScanResults()
        let scan_results = scanner
            .env
            .call_method(wifi_manager, "getScanResults", "()Ljava/util/List;", &[])
            .map_err(|e| Error::JNIError(e.to_string()))?
            .l()
            .map_err(|e| Error::JNIError(e.to_string()))?;

        // List.size()
        let size = scanner
            .env
            .call_method(scan_results, "size", "()I", &[])
            .map_err(|e| Error::JNIError(e.to_string()))?
            .i()
            .map_err(|e| Error::JNIError(e.to_string()))?;

        let mut networks = Vec::new();

        for i in 0..size {
            // List.get(i)
            let scan_result = scanner
                .env
                .call_method(
                    scan_results,
                    "get",
                    "(I)Ljava/lang/Object;",
                    &[JValue::Int(i)],
                )
                .map_err(|e| Error::JNIError(e.to_string()))?
                .l()
                .map_err(|e| Error::JNIError(e.to_string()))?;

            let ssid: String = scanner
                .env
                .get_field(scan_result, "SSID", "Ljava/lang/String;")
                .map_err(|e| Error::JNIError(e.to_string()))?
                .l()
                .and_then(|s| scanner.env.get_string(JString::from(s)))
                .map_err(|e| Error::JNIError(e.to_string()))?
                .into();

            let mac: String = scanner
                .env
                .get_field(scan_result, "BSSID", "Ljava/lang/String;")
                .map_err(|e| Error::JNIError(e.to_string()))?
                .l()
                .and_then(|s| scanner.env.get_string(JString::from(s)))
                .map_err(|e| Error::JNIError(e.to_string()))?
                .into();

            let signal_level = scanner
                .env
                .get_field(scan_result, "level", "I")
                .map_err(|e| Error::JNIError(e.to_string()))?
                .i()
                .map_err(|e| Error::JNIError(e.to_string()))?;

            let channel = scanner
                .env
                .get_field(scan_result, "frequency", "I")
                .map_err(|e| Error::JNIError(e.to_string()))?
                .i()
                .map_err(|e| Error::JNIError(e.to_string()))? as u32;

            networks.push(Wifi {
                mac,
                ssid,
                channel,
                signal_level,
                security: vec![],
            });
        }

        Ok(networks)
    }
}
