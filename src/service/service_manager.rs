use crate::error::*;
use crate::service::Service;
use crate::stability::Stability;
use crate::{
    binder::{
        Binder,
        command_protocol::BinderReturn,
        devices::BinderDevice,
        flat_object::BinderFlatObject,
        transaction::{Transaction, TransactionFlag},
        transaction_data::BinderTransactionData,
    },
    parcel::Parcel,
};

use super::BinderService;
use super::service_listener::ServiceListener;

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
    pub fn new() -> Result<Self> {
        let binder = Binder::new(BinderDevice::Binder)?;
        // binder.become_context_manager()?;
        let sv_mgr = Self { binder };
        sv_mgr.ping()?;
        Ok(sv_mgr)
    }

    fn ping(&self) -> Result<()> {
        info!("Ping");
        self.binder.transaction(
            SERVICE_MANAGER_HANDLE,
            Transaction::Ping.into(),
            TransactionFlag::empty(),
            &mut Parcel::default(),
        )
    }

    pub fn get_service<'a>(
        &'a self,
        service_name: impl AsRef<str>,
        interface_name: &'a str,
    ) -> Result<Service<'a>> {
        let mut parcel = Parcel::default();
        parcel.write_interface_token(SERVICE_MANAGER_INTERFACE_TOKEN)?;
        parcel.write(service_name.as_ref())?;
        info!("[GetService] ");

        let mut handle = None;
        // we expect an reply
        self.binder.enter_loop()?;
        self.binder.transaction_with_parse(
            SERVICE_MANAGER_HANDLE,
            ServiceManagerFunctions::GetService as _,
            TransactionFlag::empty(),
            &mut parcel,
            |_, br, d| {
                if matches!(br, BinderReturn::Reply) {
                    let transacion_data = d.read::<BinderTransactionData>()?;
                    info!("[GetService] Transaction data: \n{transacion_data:#?}");
                    let mut parcel = transacion_data.to_parcel(None);

                    if !parcel.can_read::<u32>() {
                        return Ok(true);
                    }

                    let status = parcel.read::<u32>()?;
                    info!("[GetService] [Status] {status}");

                    info!("[GetService] FlatObject in Parcel: \n{parcel:#?}");
                    let obj = parcel.read_object(false)?;
                    handle = Some(obj.handle());
                    info!("[GetService] FlatObject: \n{obj:#?}");
                    return Ok(true);
                }
                Ok(false)
            },
        )?;

        self.binder.exit_loop()?;
        Ok(Service::new(self, interface_name.as_ref(), handle.unwrap()))
    }

    pub fn register_service<'a, BS: BinderService>(
        &'a self,
        service_delegate: &'a BS,
        name: impl AsRef<str>,
        interface_name: &'a str,
        allow_isolated: bool,
        dump_priority: u32,
    ) -> Result<ServiceListener<'a, BS>> {
        info!("Register Service");
        self.binder.enter_loop()?;

        let mut parcel = Parcel::new();
        parcel.write_interface_token(SERVICE_MANAGER_INTERFACE_TOKEN)?;
        parcel.write(name.as_ref())?;
        // this is write binder

        let mut binder_flat_obj = BinderFlatObject::default();
        binder_flat_obj.set_pointer(self as *const _ as usize);
        parcel.write_object(&binder_flat_obj, true)?;
        parcel.write::<i32>(&Stability::System.into())?;
        parcel.write(&allow_isolated)?;
        parcel.write(&dump_priority)?;

        info!("\n\n\nTransaction AddServices\n\n\n");
        // we add service
        // so we expect reply
        self.binder.transaction_with_parse(
            SERVICE_MANAGER_HANDLE,
            ServiceManagerFunctions::AddService as _,
            TransactionFlag::empty(),
            &mut parcel,
            |_binder, c, p| {
                if matches!(c, BinderReturn::Reply) {
                    let transacion_data = p.read::<BinderTransactionData>()?;
                    info!("[AddService] Transaction data: \n{transacion_data:#?}");
                    let parcel = transacion_data.to_parcel(None);
                    info!("[AddService] Parcel: {parcel:#?}");
                    // we just extract this
                    // no data require for this now
                    return Ok(true);
                }
                Ok(false)
            },
        )?;

        Ok(ServiceListener::new(service_delegate, self, interface_name))
    }

    pub fn binder(&self) -> &Binder {
        &self.binder
    }
}
