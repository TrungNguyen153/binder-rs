use crate::{
    binder::{
        Binder,
        command_protocol::BinderReturn,
        devices::BinderDevice,
        flat_object::BinderFlatObject,
        transaction::{Transaction, TransactionFlag},
    },
    error::Result,
    parcel::{self, Parcel},
    parcelable::Parcelable,
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

    pub fn get_service(
        &self,
        service_name: impl AsRef<str>,
        interface_name: impl AsRef<str>,
    ) -> Result<()> {
        let mut parcel = Parcel::default();
        parcel.write_interface_token(SERVICE_MANAGER_INTERFACE_TOKEN)?;
        parcel.write_str16(service_name.as_ref())?;
        let mut flat_object = None;
        info!("Get service");

        // we expect an reply
        self.binder.enter_loop()?;
        self.binder.transaction_with_parse(
            SERVICE_MANAGER_HANDLE,
            ServiceManagerFunctions::GetService as _,
            TransactionFlag::empty(),
            &mut parcel,
            |_, br, d| {
                if matches!(br, BinderReturn::Reply) {
                    let transacion_data = d.read_transaction_data()?;
                    info!("Transaction data: \n{transacion_data:#?}");
                    let mut parcel = unsafe {
                        Parcel::from_data_and_offsets(
                            transacion_data.data,
                            transacion_data.data_size as usize,
                            transacion_data.offsets,
                            transacion_data.offsets_size as usize / size_of::<usize>(),
                        )
                    };
                    info!("FlatObject in Parcel: \n{parcel:#?}");
                    info!("Parsing flat object");
                    let obj = BinderFlatObject::deserialize(&mut parcel)?;
                    flat_object.replace(obj);
                    info!("Parsing ok");
                    return Ok(true);
                }
                Ok(false)
            },
        )?;
        self.binder.exit_loop()?;
        info!("FlatObject: {flat_object:#?}");
        Ok(())
    }
}
