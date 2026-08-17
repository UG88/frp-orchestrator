pub mod commands;
pub mod doctor;
pub mod http_client;
pub mod installer;
pub mod wizard;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frp_arch_detection() {
        let res = installer::Installer::detect_frp_arch();
        assert!(res.is_ok());
        let (os, arch) = res.unwrap();
        assert!(!os.is_empty());
        assert!(!arch.is_empty());

        let url = installer::Installer::get_download_url("0.60.0", "linux", "amd64");
        assert_eq!(
            url,
            "https://github.com/fatedier/frp/releases/download/v0.60.0/frp_0.60.0_linux_amd64.tar.gz"
        );
    }
}
