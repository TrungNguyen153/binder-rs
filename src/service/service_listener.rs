use num_traits::FromPrimitive;

use super::{BinderService, service_manager::ServiceManager};
use crate::{
    binder::{
        command_protocol::BinderReturn,
        transaction::{Transaction, TransactionFlag},
        transaction_data::BinderTransactionData,
    },
    error::*,
    parcel::Parcel,
};

pub struct ServiceListener<'a, BS: BinderService> {
    service_delegate: &'a BS,
    mgr: &'a ServiceManager,
    interface_name: &'a str,
}

impl<'a, BS: BinderService> ServiceListener<'a, BS> {
    pub fn new(service_delegate: &'a BS, mgr: &'a ServiceManager, interface_name: &'a str) -> Self {
        Self {
            service_delegate,
            mgr,
            interface_name,
        }
    }

    pub fn binder_loop(&self) -> Result<()> {
        self.mgr.binder().enter_loop()?;
        let mut in_parcel = Parcel::default();
        let mut out_parcel = Parcel::default();

        info!("[BinderLoop] Enter");

        loop {
            self.mgr.binder().binder_write(&mut out_parcel)?;
            self.mgr.binder().binder_read(&mut in_parcel)?;

            // waiting for transaction request
            // then we will reply it
            self.mgr
                .binder()
                .binder_parse(&mut in_parcel, |binder, cmd, in_parcel| {
                    match cmd {
                        BinderReturn::Transaction => {
                            let tx = in_parcel.read::<BinderTransactionData>()?;
                            info!("[BinderLoop] Transaction data: \n{tx:#?}");
                            let mut parcel = tx.to_parcel(None);

                            let status = parcel.read::<u32>()?;

                            info!("[BinderLoop] Transaction status: {status}");

                            info!("[BinderLoop] FlatObject in Parcel: \n{parcel:#?}");
                            let obj = parcel.read_object(false)?;
                            info!("[BinderLoop] FlatObject: \n{obj:#?}");

                            let transaction_code = Transaction::from_u32(tx.code);
                            if let Some(transaction_code) = transaction_code {
                                info!("[BinderLoop] We recieved transaction code: {transaction_code:?}");
                                match transaction_code {
                                    Transaction::Interface => {
                                        out_parcel.write(&0u32)?;
                                        out_parcel.write(self.interface_name)?;
                                        binder.reply(
                                            &mut out_parcel,
                                            tx.flags | TransactionFlag::AcceptFds,
                                        )?;
                                        return Ok(true);
                                    }
                                    _ => {
                                        warn!("[BinderLoop] Unhandled transaction code.");
                                    }
                                }
                            }

                            // calling resolver
                            if tx.code >= Transaction::FirstCall.into()
                                && tx.code <= Transaction::LastCall.into()
                            {
                                info!("[BinderLoop] Progress RPC...");
                                binder.reply(
                                    &mut self.service_delegate.progress_request(tx.code, in_parcel),
                                    tx.flags,
                                )?;
                                return Ok(true);
                            }
                        }
                        _ => {}
                    }
                    //
                    Ok(false)
                })?;
        }

        // self.mgr.binder().exit_loop()?;
    }
}
