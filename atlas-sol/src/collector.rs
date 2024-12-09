use agave_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
    ReplicaTransactionInfoVersions, Result as GeyserResult, SlotStatus,
};
use atlas_core::{
    error::{AtlasError, AtlasResult},
    util::AtlasUtil,
};
use crossbeam::channel::{bounded, Receiver, Sender};
use log::{error, info};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

static SOLONA_CHANNEL_SIZE: usize = 10_000;

//=======================================================================
#[derive(Debug, Clone)]
pub struct SolonaGeyserConfig {}

//=======================================================================
#[derive(Debug, Clone)]
pub struct SolonaCollector {
    receiver: Receiver<i32>,
}

//=======================================================================
#[derive(Debug, Clone)]
pub struct SolonaGeyser {
    sender: Sender<i32>,
    thread_handle: Option<Arc<Mutex<std::thread::JoinHandle<()>>>>,
    collector: Arc<Mutex<SolonaCollector>>,
}

//=======================================================================
pub struct SolonaManager {
    config: SolonaGeyserConfig,
}

//=======================================================================
impl SolonaGeyser {
    //=======================================================================
    pub fn new() -> Self {
        let (sender, receiver) = bounded(SOLONA_CHANNEL_SIZE);
        let collector = SolonaCollector::new(receiver);
        SolonaGeyser {
            sender,
            collector: Arc::new(Mutex::new(collector)),
            thread_handle: None,
        }
    }

    //=======================================================================
    pub async fn run(&self) -> AtlasResult<()> {
        info!("SolonaGeyser starting...");
        for i in 0..10 {
            self.sender.send(i).unwrap();
            sleep(Duration::from_millis(500)).await;
        }
        self.sender.send(-1).unwrap();
        info!("SolonaGeyser done.");
        Ok(())
    }
}

//=======================================================================
impl GeyserPlugin for SolonaGeyser {
    //=======================================================================
    fn name(&self) -> &'static str {
        "AtlasSolonaGeyser"
    }

    //=======================================================================
    fn on_load(&mut self, config_file: &str, is_reload: bool) -> GeyserResult<()> {
        AtlasUtil::setup_logger().unwrap();
        info!("SolonaGeyser loading...");
        let collector = self.collector.clone();
        let handle = std::thread::spawn(move || {
            info!("SolonaCollector thread starting...");
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let collector = collector.lock().unwrap();
                collector.listen().await;
            });
        });
        self.thread_handle = Some(Arc::new(Mutex::new(handle)));
        info!("SolonaGeyser started.");
        Ok(())
    }

    //=======================================================================
    fn on_unload(&mut self) {
        info!("SolonaGeyser on_unload");
        if let Some(handle) = self.thread_handle.take() {
            let handle = Arc::try_unwrap(handle).unwrap().into_inner().unwrap();
            handle.join().unwrap();
        }
    }

    //=======================================================================
    fn update_account(
        &self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> GeyserResult<()> {
        info!(
            "SolonaGeyser update_account slot: {}, is_startup: {}",
            slot, is_startup
        );
        Ok(())
    }

    //=======================================================================
    fn update_slot_status(
        &self,
        slot: u64,
        parent: Option<u64>,
        status: &SlotStatus,
    ) -> GeyserResult<()> {
        info!("Updating slot {:?} at with status {:?}", slot, status);
        Ok(())
    }

    //=======================================================================
    fn notify_end_of_startup(&self) -> GeyserResult<()> {
        info!("Notifying the end of startup for accounts notifications");
        Ok(())
    }

    //=======================================================================
    fn notify_transaction(
        &self,
        transaction_info: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> GeyserResult<()> {
        info!("Notifying transaction");
        Ok(())
    }

    //=======================================================================
    fn notify_block_metadata(&self, block_info: ReplicaBlockInfoVersions) -> GeyserResult<()> {
        info!("Notifying block metadata");
        Ok(())
    }

    //=======================================================================
    fn account_data_notifications_enabled(&self) -> bool {
        true
    }

    //=======================================================================
    fn transaction_notifications_enabled(&self) -> bool {
        true
    }
}

//=======================================================================
impl SolonaCollector {
    //=======================================================================
    pub fn new(receiver: Receiver<i32>) -> Self {
        SolonaCollector { receiver }
    }

    //=======================================================================
    pub async fn listen(&self) {
        loop {
            sleep(Duration::from_secs(1)).await;
            println!("Listener awake, checking for messages...");
            let mut message_buffer = Vec::new();
            while let Ok(message) = self.receiver.try_recv() {
                if message == -1 {
                    return;
                }
                message_buffer.push(message);
            }
            info!("Message buffer: {:?}", message_buffer);
        }
    }
}

//=======================================================================
#[no_mangle]
#[allow(improper_ctypes_definitions)]
/// # Safety
///
/// This function returns the GeyserPluginPostgres pointer as trait GeyserPlugin.
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = SolonaGeyser::new();
    let plugin: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_solona_geyser() {}
}
