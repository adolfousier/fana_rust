mod input_process;

use input_process::process_user_input;
use serde::{Deserialize, Serialize};
use std::env;


async fn run_interactive_mode(client: Client, groq_api_key: String, session_manager: SessionManager, ip_addr: IP_ADDR) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    loop {
        print!("\nYou:\n");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim().to_string();
        info!("User input: {}", user_input);

        if user_input.eq_ignore_ascii_case("exit") {
            info!("User requested exit");
            break;
        }

        if let Err(e) = process_user_input(user_input.clone(), &mut messages, &client, &groq_api_key, ip_addr).await {
            error!("Error processing user input: {}", e);
        }
    }

    Ok(())
}
