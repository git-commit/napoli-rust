use napoli_lib::napoli::order_service_client::OrderServiceClient;
use napoli_lib::napoli::GetOrdersRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // creating a channel ie connection to server
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;
    // creating gRPC client from channel
    let mut client = OrderServiceClient::new(channel);
    // creating a new Request
    let request = tonic::Request::new(GetOrdersRequest {});
    // sending request and waiting for response
    let response = client.get_orders(request).await?.into_inner();
    println!("RESPONSE={:?}", response);
    Ok(())
}
