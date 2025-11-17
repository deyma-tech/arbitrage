pub mod instruction;

use solana_trader_proto::api::api_client::ApiClient;
use solana_trader_proto::api::{
    PostSubmitBatchRequest, PostSubmitBatchResponse, PostSubmitPaladinRequest, PostSubmitRequest, PostSubmitResponse,
    TransactionMessage,
};
use std::sync::{Arc, RwLock};
use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::{Certificate, Channel, Endpoint};

#[derive(Clone)]
pub struct ClientInterceptor {
    /// The token added to each request header.
    auth_header: Arc<RwLock<String>>,
}

impl ClientInterceptor {
    fn new(auth_header: String) -> Self {
        Self {
            auth_header: Arc::new(RwLock::new(auth_header)),
        }
    }
}

impl Interceptor for ClientInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let authorization = self.auth_header.read().unwrap();
        if !authorization.is_empty() {
            request
                .metadata_mut()
                .insert("authorization", authorization.parse().unwrap());
        }

        Ok(request)
    }
}

pub async fn create_grpc_channel(url: &str, cert_path: &str) -> Channel {
    let mut endpoint = Endpoint::from_shared(url.to_string()).expect("invalid url");
    if url.contains("https") {
        // macos - /etc/ssl/cert.pem
        let pem = tokio::fs::read(cert_path)
            .await
            .expect("oh no, the cert file wasn't loaded");
        let cert = Certificate::from_pem(pem);
        endpoint = endpoint
            .tls_config(tonic::transport::ClientTlsConfig::new().ca_certificate(cert))
            .unwrap();
    }
    endpoint.connect().await.unwrap()
}

pub async fn create_grpc_client(
    url: &str,
    cert_path: &str,
    auth: String,
) -> ApiClient<InterceptedService<Channel, ClientInterceptor>> {
    let channel = create_grpc_channel(url, cert_path).await;
    let interceptor = ClientInterceptor::new(auth);
    ApiClient::with_interceptor(channel, interceptor)
}

pub async fn post_submit_request_v2(
    client: &mut ApiClient<InterceptedService<Channel, ClientInterceptor>>,
    request: PostSubmitRequest,
) -> Result<tonic::Response<PostSubmitResponse>, Status> {
    client.post_submit_v2(request).await
}

pub async fn post_submit_batch(
    client: &mut ApiClient<InterceptedService<Channel, ClientInterceptor>>,
    request: PostSubmitBatchRequest,
) -> Result<tonic::Response<PostSubmitBatchResponse>, Status> {
    client.post_submit_batch_v2(request).await
}

pub async fn post_submit_paladin(
    client: &mut ApiClient<InterceptedService<Channel, ClientInterceptor>>,
    request: PostSubmitPaladinRequest,
) -> Result<tonic::Response<PostSubmitResponse>, Status> {
    client.post_submit_paladin_v2(request).await
}

use crate::instruction::{tip_ix, versioned_tx_to_string};
use solana_program::instruction::Instruction;
use solana_program::message::{v0, VersionedMessage};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::hash::Hash;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::VersionedTransaction;
use tonic::{Request, Status};

pub fn send_to_bloxroute(
    hash: Hash,
    keypair: &Keypair,
    compute_unit_price: u64,
    compute_unit_limit: u32,
    tip: u64,
    instructions: Vec<Instruction>,
) -> PostSubmitRequest {
    let t_ix = tip_ix(tip, &keypair.pubkey());
    let mut ixs = vec![];
    //     ComputeBudgetInstruction::set_compute_unit_price(compute_unit_price),
    //     ComputeBudgetInstruction::set_compute_unit_limit(compute_unit_limit),
    // ];
    ixs.extend(instructions);
    ixs.push(ComputeBudgetInstruction::set_compute_unit_price(compute_unit_price));
    ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(compute_unit_limit));
    ixs.push(t_ix);
    let bloxroute_tx = VersionedTransaction::try_new(
        VersionedMessage::V0(v0::Message::try_compile(&keypair.pubkey(), &ixs, &[], hash).unwrap()),
        &[keypair],
    )
    .unwrap();
    let encoded = versioned_tx_to_string(&bloxroute_tx);
    PostSubmitRequest {
        transaction: Some(TransactionMessage {
            content: encoded,
            is_cleanup: false,
        }),
        skip_pre_flight: true,
        front_running_protection: Some(false),
        tip: Some(tip),
        fast_best_effort: None,
        use_staked_rp_cs: Some(true),
        allow_back_run: None,
        revenue_address: None,
        sniping: None,
        submit_protection: None,
        timestamp: None,
    }
}

pub fn send_to_bloxroute_jito(
    hash: Hash,
    keypair: &Keypair,
    compute_unit_price: u64,
    compute_unit_limit: u32,
    tip: u64,
    instructions: Vec<Instruction>,
) -> PostSubmitRequest {
    let t_ix = tip_ix(tip, &keypair.pubkey());
    let mut ixs = vec![];
    //     ComputeBudgetInstruction::set_compute_unit_price(compute_unit_price),
    //     ComputeBudgetInstruction::set_compute_unit_limit(compute_unit_limit),
    // ];
    ixs.extend(instructions);
    ixs.push(ComputeBudgetInstruction::set_compute_unit_price(compute_unit_price));
    ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(compute_unit_limit));
    ixs.push(t_ix);
    let bloxroute_tx = VersionedTransaction::try_new(
        VersionedMessage::V0(v0::Message::try_compile(&keypair.pubkey(), &ixs, &[], hash).unwrap()),
        &[keypair],
    )
    .unwrap();
    let encoded = versioned_tx_to_string(&bloxroute_tx);
    PostSubmitRequest {
        transaction: Some(TransactionMessage {
            content: encoded,
            is_cleanup: false,
        }),
        skip_pre_flight: true,
        front_running_protection: Some(true),
        tip: Some(tip),
        fast_best_effort: Some(true),
        use_staked_rp_cs: Some(false),
        allow_back_run: None,
        revenue_address: None,
        sniping: None,
        submit_protection: None,
        timestamp: None,
    }
}
