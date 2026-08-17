use frp_shared::models::Protocol;
use std::collections::HashMap;
use tracing::debug;

pub struct ProtocolDetector {
    egg_overrides: HashMap<String, Protocol>,
}

impl ProtocolDetector {
    pub fn new(egg_overrides: HashMap<String, Protocol>) -> Self {
        Self { egg_overrides }
    }

    /// Detect network protocol based on server name, egg/docker metadata, and environment.
    pub fn detect(
        &self,
        explicit_protocol: Protocol,
        server_name: Option<&str>,
        egg_name: Option<&str>,
        docker_image: Option<&str>,
    ) -> Protocol {
        if explicit_protocol != Protocol::Auto {
            return explicit_protocol;
        }

        // Check if egg matches configured overrides
        if let Some(egg) = egg_name {
            if let Some(proto) = self.egg_overrides.get(egg) {
                debug!(egg = %egg, protocol = %proto, "Protocol detected from egg overrides");
                return *proto;
            }
        }

        let combined = format!(
            "{} {} {}",
            server_name.unwrap_or_default(),
            egg_name.unwrap_or_default(),
            docker_image.unwrap_or_default()
        )
        .to_lowercase();

        // Check for Geyser / Bedrock / Floodgate indicators
        if combined.contains("geyser") || combined.contains("floodgate") {
            debug!("Geyser/Floodgate detected, setting protocol to Both (TCP+UDP)");
            return Protocol::Both;
        }

        if combined.contains("bedrock")
            || combined.contains("nukkit")
            || combined.contains("pocketmine")
            || combined.contains("powernukkit")
        {
            debug!("Bedrock standalone detected, setting protocol to UDP");
            return Protocol::Udp;
        }

        // Default for standard Minecraft Java Edition
        Protocol::Tcp
    }
}
