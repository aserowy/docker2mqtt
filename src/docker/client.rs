use bollard::Docker;
use tracing::{error, instrument};

#[instrument(level = "debug")]
pub fn new() -> Docker {
    match Docker::connect_with_unix_defaults() {
        Ok(client) => client,
        Err(e) => {
            error!("failed to create docker client: {}", e);
            panic!();
        }
    }
}
