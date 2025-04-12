use jni::{JNIEnv, JNIVersion, JavaVM, objects::JObject};

use crate::{
    parcel::Parcel,
    service::{BinderService, service_manager::ServiceManager},
};

struct MyService;
impl BinderService for MyService {
    fn progress_request(
        &self,
        code: u32,
        data: &mut crate::parcel::Parcel,
    ) -> crate::parcel::Parcel {
        info!("We got code: {code}");
        Parcel::default()
    }
}

#[tokio::main]
async fn service_root() {
    let handler = MyService;
    let service = ServiceManager::new().unwrap();
    service
        .register_service(&handler, "myservice", "com.example.IMyService", true, 0)
        .unwrap()
        .binder_loop()
        .unwrap();

    info!("Graceful exit!");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn Java_com_example_binderserver_BinderServer_00024Companion_loadService<
    'local,
>(
    mut _env: JNIEnv<'local>,
    _obj: JObject<'local>,
) {
    std::thread::spawn(service_root);
    std::thread::spawn(client_service_root);
}

#[tokio::main]
async fn client_service_root() {
    std::thread::sleep_ms(3000);
    info!("\n\n\nClientRoot\n\n\n");
    let service = ServiceManager::new().unwrap();
    service
        .get_service("myservice", "com.example.IMyService")
        .unwrap();

    info!("Graceful exit!");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn Java_com_example_binderclient_BinderClient_00024Companion_loadService<
    'local,
>(
    mut _env: JNIEnv<'local>,
    _obj: JObject<'local>,
) {
    std::thread::spawn(client_service_root);
}

#[unsafe(no_mangle)]
unsafe extern "C" fn JNI_OnLoad(_vm: JavaVM, _: *const u8) -> i32 {
    init_logger("BinderRs");
    warn!("JNI_OnLoad");

    JNIVersion::V6.into()
}

fn init_logger(tag: &str) {
    use tracing_subscriber::layer::SubscriberExt;
    let stdout_log = tracing_subscriber::fmt::layer()
        .compact()
        .with_line_number(true);
    let subscriber = tracing_subscriber::Registry::default().with(stdout_log);

    // Add panic hook
    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::capture();
        error!("{backtrace:?}");
        error!("{panic_info}");
    }));

    // Upgrade logger on android
    #[cfg(target_os = "android")]
    let subscriber = {
        match tracing_android::layer(&tag) {
            Ok(android_layer) => subscriber.with(android_layer),
            Err(e) => {
                error!("Unsuccess logcat create (maybe already exist): {e}");
                return;
            }
        }
    };

    match tracing::subscriber::set_global_default(subscriber) {
        Ok(()) => {}
        Err(e) => {
            error!("Unsuccess set global tracing default: {e}");
        }
    };

    #[cfg(target_os = "android")]
    {
        warn!(
            "[{}] Android logging enabled! Layer created.",
            get_arch_name()
        );
    }
}

fn get_arch_name() -> &'static str {
    #[cfg(target_arch = "x86")]
    return "x86";

    #[cfg(target_arch = "x86_64")]
    return "x86_64";

    #[cfg(target_arch = "arm")]
    return "arm";

    #[cfg(target_arch = "aarch64")]
    return "aarch64";

    #[cfg(not(any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64",
    )))]
    return "unknown";
}
