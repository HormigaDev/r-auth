use r_auth_api::run_app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_app().await
}
