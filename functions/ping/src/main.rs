use lambda_http::{
    service_fn,
    Request,
    Error,
    IntoResponse,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(func)).await?;
    Ok(())
}

async fn func(_: Request) -> Result<impl IntoResponse, Error> {
    Ok("Pong")
}
