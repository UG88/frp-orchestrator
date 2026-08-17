use crate::db::Database;
use anyhow::{bail, Result};
use frp_shared::models::{Gateway, PortPoolStatus, Protocol};
use std::collections::HashSet;
use tracing::{debug, info};

#[derive(Clone)]
pub struct PortManager {
    db: Database,
}

impl PortManager {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Allocate a port on the specified gateway for the given protocol.
    /// For `Protocol::Both`, it finds the lowest available port that is free in BOTH TCP and UDP pools.
    pub fn allocate_port(&self, gateway: &Gateway, protocol: Protocol) -> Result<u16> {
        let reserved_set: HashSet<u16> = gateway.reserved_ports.iter().copied().collect();

        match protocol {
            Protocol::Tcp => {
                let allocated_tcp = self.db.get_allocated_ports(&gateway.id, "tcp")?;
                for port in gateway.tcp_port_range_start..=gateway.tcp_port_range_end {
                    if !reserved_set.contains(&port) && !allocated_tcp.contains(&port) {
                        debug!(gateway = %gateway.id, port = port, "Allocated TCP port");
                        return Ok(port);
                    }
                }
                bail!("No available TCP ports on gateway {}", gateway.id);
            }
            Protocol::Udp => {
                let allocated_udp = self.db.get_allocated_ports(&gateway.id, "udp")?;
                for port in gateway.udp_port_range_start..=gateway.udp_port_range_end {
                    if !reserved_set.contains(&port) && !allocated_udp.contains(&port) {
                        debug!(gateway = %gateway.id, port = port, "Allocated UDP port");
                        return Ok(port);
                    }
                }
                bail!("No available UDP ports on gateway {}", gateway.id);
            }
            Protocol::Both | Protocol::Auto => {
                let allocated_tcp = self.db.get_allocated_ports(&gateway.id, "tcp")?;
                let allocated_udp = self.db.get_allocated_ports(&gateway.id, "udp")?;

                let start = gateway.tcp_port_range_start.max(gateway.udp_port_range_start);
                let end = gateway.tcp_port_range_end.min(gateway.udp_port_range_end);

                if start > end {
                    bail!(
                        "Gateway {} TCP and UDP ranges do not overlap, cannot allocate matching port for Both/Auto",
                        gateway.id
                    );
                }

                for port in start..=end {
                    if !reserved_set.contains(&port)
                        && !allocated_tcp.contains(&port)
                        && !allocated_udp.contains(&port)
                    {
                        info!(gateway = %gateway.id, port = port, "Allocated dual TCP+UDP port");
                        return Ok(port);
                    }
                }
                bail!("No matching dual TCP+UDP ports available on gateway {}", gateway.id);
            }
        }
    }

    /// Retrieve port pool statistics for a gateway.
    pub fn get_pool_status(&self, gateway: &Gateway) -> Result<PortPoolStatus> {
        let allocated_tcp = self.db.get_allocated_ports(&gateway.id, "tcp")?;
        let allocated_udp = self.db.get_allocated_ports(&gateway.id, "udp")?;
        let reserved_set: HashSet<u16> = gateway.reserved_ports.iter().copied().collect();

        let tcp_total = (gateway.tcp_port_range_end - gateway.tcp_port_range_start + 1) as u32;
        let udp_total = (gateway.udp_port_range_end - gateway.udp_port_range_start + 1) as u32;

        let mut tcp_usable = 0u32;
        for p in gateway.tcp_port_range_start..=gateway.tcp_port_range_end {
            if !reserved_set.contains(&p) {
                tcp_usable += 1;
            }
        }

        let mut udp_usable = 0u32;
        for p in gateway.udp_port_range_start..=gateway.udp_port_range_end {
            if !reserved_set.contains(&p) {
                udp_usable += 1;
            }
        }

        let tcp_allocated = allocated_tcp.len() as u32;
        let udp_allocated = allocated_udp.len() as u32;

        Ok(PortPoolStatus {
            gateway_id: gateway.id.clone(),
            tcp_total,
            tcp_allocated,
            tcp_available: tcp_usable.saturating_sub(tcp_allocated),
            udp_total,
            udp_allocated,
            udp_available: udp_usable.saturating_sub(udp_allocated),
        })
    }
}
