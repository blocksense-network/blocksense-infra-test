use crate::feeds::feed_allocator::ConcurrentAllocator;
use crate::providers::provider::ProviderStatus;
use crate::providers::provider::SharedRpcProviders;
use crate::reporters::reporter::init_shared_reporters;
use crate::reporters::reporter::SharedReporters;
use blockchain_data_model::in_mem_db::InMemDb;
use config::AllFeedsConfig;
use config::FeedConfig;
use config::SequencerConfig;
use data_feeds::feeds_processing::VotedFeedUpdate;
use feed_registry::feed_registration_cmds::FeedsManagementCmds;
use feed_registry::registry::new_feeds_meta_data_reg_from_config;
use feed_registry::registry::{AllFeedsReports, FeedAggregateHistory, FeedMetaDataRegistry};
use prometheus::metrics::FeedsMetrics;
use rdkafka::producer::FutureProducer;
use rdkafka::ClientConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;
use utils::logging::SharedLoggingHandle;

pub struct SequencerState {
    pub registry: Arc<RwLock<FeedMetaDataRegistry>>,
    pub reports: Arc<RwLock<AllFeedsReports>>,
    pub providers: SharedRpcProviders,
    pub log_handle: SharedLoggingHandle,
    pub reporters: SharedReporters,
    pub feed_id_allocator: Arc<RwLock<Option<ConcurrentAllocator>>>,
    pub aggregated_votes_to_block_creator_send: UnboundedSender<VotedFeedUpdate>,
    pub feeds_metrics: Arc<RwLock<FeedsMetrics>>,
    pub active_feeds: Arc<RwLock<HashMap<u32, FeedConfig>>>,
    pub sequencer_config: Arc<RwLock<SequencerConfig>>,
    pub feed_aggregate_history: Arc<RwLock<FeedAggregateHistory>>,
    pub feeds_management_cmd_to_block_creator_send: UnboundedSender<FeedsManagementCmds>,
    pub feeds_slots_manager_cmd_send: UnboundedSender<FeedsManagementCmds>,
    pub blockchain_db: Arc<RwLock<InMemDb>>,
    pub kafka_endpoint: Option<FutureProducer>,
    pub provider_status: Arc<RwLock<HashMap<String, ProviderStatus>>>,
    // pub voting_recv_channel: Arc<RwLock<mpsc::UnboundedReceiver<(String, String)>>>,
}

impl SequencerState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        feeds_config: AllFeedsConfig,
        providers: SharedRpcProviders,
        log_handle: SharedLoggingHandle,
        sequencer_config: &SequencerConfig,
        metrics_prefix: Option<&str>,
        feed_id_allocator: Option<ConcurrentAllocator>,
        aggregated_votes_to_block_creator_send: UnboundedSender<VotedFeedUpdate>,
        feeds_management_cmd_to_block_creator_send: UnboundedSender<FeedsManagementCmds>,
        feeds_slots_manager_cmd_send: UnboundedSender<FeedsManagementCmds>,
    ) -> SequencerState {
        let provider_status: HashMap<String, ProviderStatus> = sequencer_config
            .providers
            .iter()
            .map(|(provider_name, provider)| {
                let initial_state = if provider.is_enabled {
                    ProviderStatus::AwaitingFirstUpdate
                } else {
                    ProviderStatus::Disabled
                };
                (provider_name.clone(), initial_state)
            })
            .collect();
        let provider_status = Arc::new(RwLock::new(provider_status));

        SequencerState {
            registry: Arc::new(RwLock::new(new_feeds_meta_data_reg_from_config(
                &feeds_config,
            ))),
            reports: Arc::new(RwLock::new(AllFeedsReports::new())),
            providers,
            log_handle,
            reporters: init_shared_reporters(sequencer_config, metrics_prefix),
            feed_id_allocator: Arc::new(RwLock::new(feed_id_allocator)),
            aggregated_votes_to_block_creator_send,
            feeds_metrics: Arc::new(RwLock::new(
                FeedsMetrics::new(metrics_prefix.unwrap_or(""))
                    .expect("Failed to allocate feed_metrics"),
            )),
            active_feeds: Arc::new(RwLock::new(
                feeds_config
                    .feeds
                    .into_iter()
                    .map(|feed| (feed.id, feed))
                    .collect(),
            )),
            sequencer_config: Arc::new(RwLock::new(sequencer_config.clone())),
            feed_aggregate_history: Arc::new(RwLock::new(FeedAggregateHistory::new())),
            feeds_management_cmd_to_block_creator_send,
            feeds_slots_manager_cmd_send,
            blockchain_db: Arc::new(RwLock::new(InMemDb::new())),
            kafka_endpoint: sequencer_config
                .kafka_report_endpoint
                .url
                .as_ref()
                .map(|url| {
                    create_kafka_producer(url)
                        .expect("Could not create kafka communication channel.")
                }),
            provider_status,
        }
    }
}

fn create_kafka_producer(
    bootstrap_server: &str,
) -> Result<FutureProducer, Box<dyn std::error::Error>> {
    Ok(ClientConfig::new()
        .set("bootstrap.servers", bootstrap_server)
        .set("queue.buffering.max.ms", "0")
        .create()?)
}
