use nextblock_protos::api::api_client::ApiClient;
use nextblock_protos::api::{PingRequest, PongResponse};
use nextblock_protos::api::{PostSubmitBatchRequest, PostSubmitRequest, PostSubmitResponse};

use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use std::sync::{Arc, RwLock};
use tokio::time::sleep;
use tonic::service::Interceptor;
use tonic::transport::{Certificate, Endpoint};
use tonic::Request;
use tonic::{codegen::InterceptedService, transport::Channel, Status};
use utils::rnd::rnd_element;

const ADDRESSES: [Pubkey; 8] = [
    pubkey!("NEXTbLoCkB51HpLBLojQfpyVAMorm3zzKg7w9NFdqid"),
    pubkey!("nextBLoCkPMgmG8ZgJtABeScP35qLa2AMCNKntAP7Xc"),
    pubkey!("NextbLoCkVtMGcV47JzewQdvBpLqT9TxQFozQkN98pE"),
    pubkey!("NexTbLoCkWykbLuB1NkjXgFWkX9oAtcoagQegygXXA2"),
    pubkey!("NeXTBLoCKs9F1y5PJS9CKrFNNLU1keHW71rfh7KgA1X"),
    pubkey!("NexTBLockJYZ7QD7p2byrUa6df8ndV2WSd8GkbWqfbb"),
    pubkey!("neXtBLock1LeC67jYd1QdAa32kbVeubsfPNTJC1V5At"),
    pubkey!("nEXTBLockYgngeRmRrjDV31mGSekVPqZoMGhQEZtPVG"),
    /*
        "NeXTBLoCKs9F1y5PJS9CKrFNNLU1keHW71rfh7KgA1X",
    "NexTBLockJYZ7QD7p2byrUa6df8ndV2WSd8GkbWqfbb",
    "neXtBLock1LeC67jYd1QdAa32kbVeubsfPNTJC1V5At",
    "nEXTBLockYgngeRmRrjDV31mGSekVPqZoMGhQEZtPVG",
     */
];

#[inline(always)]
pub fn get_next_block_random_address() -> Pubkey {
    rnd_element(&ADDRESSES).to_owned()
}

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
    let client = ApiClient::with_interceptor(channel, interceptor);
    tokio::spawn({
        let client = client.clone();
        async move {
            let _ = ping_loop(client).await;
        }
    });
    client
}

async fn ping_loop(mut client: ApiClient<InterceptedService<Channel, ClientInterceptor>>) -> Result<(), Status> {
    //let mut ping_id = 0;
    loop {
        //ping_id += 1;
        let ping_msg = PingRequest {};
        let _ = client.ping(ping_msg).await;
        sleep(tokio::time::Duration::from_secs(15)).await; // Ping every 5 seconds
    }
}

pub async fn ping(
    client: &mut ApiClient<InterceptedService<Channel, ClientInterceptor>>,
    ping_request: PingRequest,
) -> Result<tonic::Response<PongResponse>, Status> {
    client.ping(ping_request).await
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
) -> Result<tonic::Response<PostSubmitResponse>, Status> {
    client.post_submit_batch_v2(request).await
}

/*
   "NEXTbLoCkB51HpLBLojQfpyVAMorm3zzKg7w9NFdqid",
   "nextBLoCkPMgmG8ZgJtABeScP35qLa2AMCNKntAP7Xc",
   "NextbLoCkVtMGcV47JzewQdvBpLqT9TxQFozQkN98pE",
   "NexTbLoCkWykbLuB1NkjXgFWkX9oAtcoagQegygXXA2",
   "NeXTBLoCKs9F1y5PJS9CKrFNNLU1keHW71rfh7KgA1X",
   "NexTBLockJYZ7QD7p2byrUa6df8ndV2WSd8GkbWqfbb",
   "neXtBLock1LeC67jYd1QdAa32kbVeubsfPNTJC1V5At",
   "nEXTBLockYgngeRmRrjDV31mGSekVPqZoMGhQEZtPVG",
*/
