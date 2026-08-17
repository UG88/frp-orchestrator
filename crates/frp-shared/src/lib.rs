pub mod api_types;
pub mod config;
pub mod crypto;
pub mod dns;
pub mod models;

#[cfg(test)]
mod tests {
    use super::*;
    use models::Protocol;

    #[test]
    fn test_protocol_handling() {
        assert!(Protocol::Tcp.is_tcp());
        assert!(!Protocol::Tcp.is_udp());

        assert!(!Protocol::Udp.is_tcp());
        assert!(Protocol::Udp.is_udp());

        assert!(Protocol::Both.is_tcp());
        assert!(Protocol::Both.is_udp());
    }

    #[test]
    fn test_crypto_token_verification() {
        let secret = "super-secret-gateway-token-12345";
        let hash = crypto::hash_token(secret);
        assert!(crypto::verify_token(secret, &hash));
        assert!(!crypto::verify_token("wrong-secret", &hash));
    }

    #[test]
    fn test_config_port_range_validation() {
        let valid_range = config::PortRangeConfig {
            start: 30000,
            end: 40000,
        };
        assert!(valid_range.validate("test").is_ok());
        assert_eq!(valid_range.total_ports(), 10001);
        assert!(valid_range.contains(35000));
        assert!(!valid_range.contains(29999));

        let invalid_range = config::PortRangeConfig {
            start: 40000,
            end: 30000,
        };
        assert!(invalid_range.validate("test").is_err());
    }
}
