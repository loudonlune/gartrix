
mod database;
mod web;
mod config;

type Error = ();

#[tokio::main]
async fn main() -> Result<(), Error> {
    web::initialize().await;
    


    config::write_config();
    Ok(())
}
