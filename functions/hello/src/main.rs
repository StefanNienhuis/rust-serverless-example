use lambda_http::{
    service_fn,
    Request,
    Error,
    IntoResponse,
    Body,
};

use serde_json::{Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(func)).await?;
    Ok(())
}

async fn func(request: Request) -> Result<impl IntoResponse, Error> {
    let body = request.body();

    match body {
        Body::Empty => Ok("No body provided".into_response()),
        Body::Text(string) => {
            let body: Value = serde_json::from_str(string.as_str())?;
            
            let message = format!(
                "Hello, {}",
                body.get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("mysterious person")
            );

            Ok(String::from(message).into_response())
        }
        Body::Binary(_) => Ok("Body should be text".into_response()),
    }
}
