use crate::{
    binder::{
        command_protocol::BinderReturn, transaction::TransactionFlag,
        transaction_data::BinderTransactionData,
    },
    error::*,
};
use service_manager::ServiceManager;

use crate::parcel::Parcel;

pub mod service_listener;
pub mod service_manager;

pub trait BinderService {
    fn progress_request(&self, code: u32, data: &mut Parcel) -> Parcel;
}

pub struct Service<'a> {
    mgr: &'a ServiceManager,
    // handle from resolve service interface
    handle: u32,
    interface_name: &'a str,
}

impl<'a> Service<'a> {
    pub fn new(mgr: &'a ServiceManager, interface_name: &'a str, handle: u32) -> Self {
        Self {
            mgr,
            handle,
            interface_name,
        }
    }

    pub fn call(&self, function_idx: u32, data: &mut Parcel) -> Result<()> {
        let mut parcel = Parcel::new();
        parcel.write_interface_token(self.interface_name)?;
        if !data.is_empty() {
            parcel.append_all_from(data)?;
        }

        // we transaction request
        // so we expect service reply
        self.mgr.binder().transaction_with_parse(
            self.handle,
            function_idx,
            TransactionFlag::AcceptFds | TransactionFlag::CollectNotedAppOps,
            &mut parcel,
            |_binder, cmd, in_parcel| {
                if matches!(cmd, BinderReturn::Reply) {
                    let tx = in_parcel.read::<BinderTransactionData>()?;
                    info!("Transaction data: \n{tx:#?}");
                    let mut parcel = tx.to_parcel(None);

                    let status = parcel.read::<u32>()?;
                    if status != 0 {
                        panic!("Service call failed: {parcel:#?}");
                    }

                    info!("FlatObject in Parcel: \n{parcel:#?}");
                    info!("Parsing flat object");
                    let obj = parcel.read_object(false)?;
                    info!("FlatObject: \n{obj:#?}");
                    info!("Parsing ok");
                }
                Ok(false)
            },
        )?;

        Ok(())
    }
}
