use crate::{
    binder::{
        Binder,
        devices::BinderDevice,
        transaction::{Transaction, TransactionFlag},
    },
    error::BinderResult,
    parcel::Parcel,
};

const SERVICE_MANAGER_HANDLE: u32 = 0;
const SERVICE_MANAGER_INTERFACE_TOKEN: &str = "android.os.IServiceManager";

#[repr(u32)]
enum ServiceManagerFunctions {
    GetService = 1,
    CheckService = 2,
    AddService = 3,
    ListServices = 4,
}

pub struct ServiceManager {
    binder: Binder,
}

impl ServiceManager {
    pub fn new() -> BinderResult<Self> {
        let binder = Binder::new(BinderDevice::Binder)?;
        binder.become_context_manager()?;
        let sv_mgr = Self { binder };
        sv_mgr.ping()?;
        Ok(sv_mgr)
    }

    fn ping(&self) -> BinderResult<()> {
        self.binder.transaction(
            SERVICE_MANAGER_HANDLE,
            Transaction::Ping.into(),
            TransactionFlag::empty(),
            &mut Parcel::default(),
        )
    }

    pub fn get_service(
        &self,
        service_name: impl AsRef<str>,
        interface_name: impl AsRef<str>,
    ) -> BinderResult<()> {
        let mut parcel = Parcel::default();
        parcel.write_interface_token(SERVICE_MANAGER_INTERFACE_TOKEN)?;
        parcel.write_str16(service_name.as_ref())?;
        self.binder.transaction_with_parse(
            SERVICE_MANAGER_HANDLE,
            ServiceManagerFunctions::GetService as _,
            TransactionFlag::empty(),
            &mut parcel,
            |parcel| {
                //
                false
            },
        )?;
        Ok(())
    }
}
