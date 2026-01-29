use crate::{Error, Result, Wifi};
use jni::objects::{GlobalRef, JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};
use std::sync::OnceLock;

pub static ANDROID_SCANNER: OnceLock<AndroidScanner> = OnceLock::new();

pub struct AndroidScanner {
    java_vm: JavaVM,
    context: GlobalRef,
}

impl AndroidScanner {
    pub fn init_with_env(env: &mut JNIEnv, context: JObject) -> Result<()> {
        ANDROID_SCANNER.get_or_init(|| AndroidScanner {
            java_vm: env.get_java_vm().unwrap(),
            context: env.new_global_ref(context).unwrap(),
        });

        Ok(())
    }
}

impl AndroidScanner {
    pub fn scan() -> Result<Vec<Wifi>> {
        let scanner = ANDROID_SCANNER
            .get()
            .ok_or_else(|| Error::JNIError("AndroidScanner not initialized".to_string()))?;

        let mut env = scanner
            .java_vm
            .attach_current_thread()
            .map_err(|e| Error::JNIError(e.to_string()))?;

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
                &scanner.context,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&wifi_service)],
            )
            .map_err(|e| Error::JNIError(e.to_string()))?
            .l()
            .map_err(|e| Error::JNIError(e.to_string()))?;

        // wifiManager.startScan()
        env.call_method(&wifi_manager, "startScan", "()Z", &[])
            .map_err(|e| Error::JNIError(e.to_string()))?;

        // wifiManager.getScanResults()
        let scan_results = env
            .call_method(&wifi_manager, "getScanResults", "()Ljava/util/List;", &[])
            .map_err(|e| Error::JNIError(e.to_string()))?
            .l()
            .map_err(|e| Error::JNIError(e.to_string()))?;

        // List.size()
        let size = env
            .call_method(&scan_results, "size", "()I", &[])
            .map_err(|e| Error::JNIError(e.to_string()))?
            .i()
            .map_err(|e| Error::JNIError(e.to_string()))?;

        let mut networks = Vec::new();

        for i in 0..size {
            // List.get(i)
            let scan_result = env
                .call_method(
                    &scan_results,
                    "get",
                    "(I)Ljava/lang/Object;",
                    &[JValue::Int(i)],
                )
                .map_err(|e| Error::JNIError(e.to_string()))?
                .l()
                .map_err(|e| Error::JNIError(e.to_string()))?;

            let ssid_j_string = env
                .get_field(&scan_result, "SSID", "Ljava/lang/String;")
                .map_err(|e| Error::JNIError(e.to_string()))?
                .l();

            let ssid: String = env
                .get_string(&JString::from(
                    ssid_j_string.map_err(|e| Error::JNIError(e.to_string()))?,
                ))
                .map_err(|e| Error::JNIError(e.to_string()))?
                .into();

            let mac_J_str = env
                .get_field(&scan_result, "BSSID", "Ljava/lang/String;")
                .map_err(|e| Error::JNIError(e.to_string()))?
                .l();

            let mac: String = env
                .get_string(&JString::from(
                    mac_J_str.map_err(|e| Error::JNIError(e.to_string()))?,
                ))
                .map_err(|e| Error::JNIError(e.to_string()))?
                .into();

            let signal_level = env
                .get_field(&scan_result, "level", "I")
                .map_err(|e| Error::JNIError(e.to_string()))?
                .i()
                .map_err(|e| Error::JNIError(e.to_string()))?;

            let channel = env
                .get_field(&scan_result, "frequency", "I")
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
