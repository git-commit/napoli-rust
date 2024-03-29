use bytes::{Buf, BufMut, Bytes, BytesMut};
use http::header::{ACCEPT, CONTENT_TYPE};
use napoli_lib::napoli::{GetOrdersReply, GetOrdersRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let msg = GetOrdersRequest {};

    let byt = encode_body(msg.to_owned());
    println!("BYTES={:?}", byt);
    // write file to disk
    std::fs::write("request.bin", byt)?;

    // a good old http/1.1 request
    let request = http::Request::builder()
        .version(http::Version::HTTP_11)
        .method(http::Method::POST)
        .uri("http://[::1]:50051/napoli.OrderService/GetOrders")
        .header(CONTENT_TYPE, "application/grpc-web")
        .header(ACCEPT, "application/grpc-web")
        .body(hyper::Body::from(encode_body(msg)))
        .unwrap();

    let client = hyper::Client::new();

    let response = client.request(request).await.unwrap();

    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        "application/grpc-web+proto"
    );

    let body = response.into_body();
    let reply = decode_body::<GetOrdersReply>(body).await;

    println!("REPLY={:?}", reply);

    Ok(())
}

// one byte for the compression flag plus four bytes for the length
const GRPC_HEADER_SIZE: usize = 5;

fn encode_body<T>(msg: T) -> Bytes
where
    T: prost::Message,
{
    let mut buf = BytesMut::with_capacity(1024);

    // first skip past the header
    // cannot write it yet since we don't know the size of the
    // encoded message
    buf.reserve(GRPC_HEADER_SIZE);
    unsafe {
        buf.advance_mut(GRPC_HEADER_SIZE);
    }

    // write the message
    msg.encode(&mut buf).unwrap();

    // now we know the size of encoded message and can write the
    // header
    let len = buf.len() - GRPC_HEADER_SIZE;
    {
        let mut buf = &mut buf[..GRPC_HEADER_SIZE];

        // compression flag, 0 means "no compression"
        buf.put_u8(0);

        buf.put_u32(len as u32);
    }

    buf.split_to(len + GRPC_HEADER_SIZE).freeze()
}

async fn decode_body<T>(body: hyper::Body) -> T
where
    T: Default + prost::Message,
{
    let mut body = hyper::body::to_bytes(body).await.unwrap();

    // ignore the compression flag
    body.advance(1);

    let len = body.get_u32();
    #[allow(clippy::let_and_return)]
    let msg = T::decode(&mut body.split_to(len as usize)).unwrap();

    msg
}
