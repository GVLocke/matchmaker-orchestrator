use supabase::Client;
use std::env;
use dotenvy::dotenv;

pub async fn authenticate_supabase_client() -> supabase::Result<Client> {
    dotenv().ok();
    let endpoint = env::var("SUPABASE_ENDPOINT").expect("SUPABASE-ENDPOINT must be set");
    let service_key = env::var("SERVICE_KEY").expect("SERVICE_KEY must be set");

    Client::new(&endpoint, &service_key)
}
