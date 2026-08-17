use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use frp_shared::models::{Allocation, AllocationStatus, AuditLog, Gateway, Mapping, Node, Protocol};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open SQLite database")?;
        
        // Performance & durability settings: WAL mode, foreign keys, synchronous NORMAL
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to open in-memory SQLite database")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS gateways (
                id TEXT PRIMARY KEY,
                region TEXT NOT NULL,
                public_ip TEXT NOT NULL,
                control_port INTEGER NOT NULL,
                tcp_start INTEGER NOT NULL,
                tcp_end INTEGER NOT NULL,
                udp_start INTEGER NOT NULL,
                udp_end INTEGER NOT NULL,
                reserved_ports_json TEXT NOT NULL DEFAULT '[]',
                is_healthy INTEGER NOT NULL DEFAULT 1,
                last_heartbeat TEXT,
                token TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                pterodactyl_node_id INTEGER NOT NULL UNIQUE,
                assigned_gateway_id TEXT NOT NULL REFERENCES gateways(id) ON DELETE CASCADE,
                local_ip TEXT NOT NULL,
                is_healthy INTEGER NOT NULL DEFAULT 1,
                last_heartbeat TEXT,
                agent_token TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS allocations (
                id TEXT PRIMARY KEY,
                node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
                server_id TEXT,
                server_name TEXT,
                pterodactyl_allocation_id INTEGER NOT NULL,
                local_ip TEXT NOT NULL,
                local_port INTEGER NOT NULL,
                protocol TEXT NOT NULL,
                custom_alias TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(node_id, pterodactyl_allocation_id)
            );

            CREATE TABLE IF NOT EXISTS mappings (
                id TEXT PRIMARY KEY,
                allocation_id TEXT NOT NULL REFERENCES allocations(id) ON DELETE CASCADE,
                gateway_id TEXT NOT NULL REFERENCES gateways(id) ON DELETE CASCADE,
                protocol TEXT NOT NULL,
                gateway_port INTEGER NOT NULL,
                target_ip TEXT NOT NULL,
                target_port INTEGER NOT NULL,
                fqdn TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                error_message TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(gateway_id, protocol, gateway_port)
            );

            CREATE TABLE IF NOT EXISTS allocated_ports (
                gateway_id TEXT NOT NULL REFERENCES gateways(id) ON DELETE CASCADE,
                protocol TEXT NOT NULL,
                port INTEGER NOT NULL,
                mapping_id TEXT NOT NULL REFERENCES mappings(id) ON DELETE CASCADE,
                created_at TEXT NOT NULL,
                PRIMARY KEY (gateway_id, protocol, port)
            );

            CREATE TABLE IF NOT EXISTS audit_logs (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                details TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_mappings_allocation ON mappings(allocation_id);
            CREATE INDEX IF NOT EXISTS idx_mappings_gateway ON mappings(gateway_id);
            CREATE INDEX IF NOT EXISTS idx_allocations_node ON allocations(node_id);
            CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_logs(created_at);
            "#,
        )?;
        Ok(())
    }

    // --- Gateway Operations ---

    pub fn upsert_gateway(&self, gw: &Gateway) -> Result<()> {
        let conn = self.conn.lock();
        let reserved_json = serde_json::to_string(&gw.reserved_ports)?;
        let last_hb = gw.last_heartbeat.map(|dt| dt.to_rfc3339());
        let created_at = gw.created_at.to_rfc3339();

        conn.execute(
            r#"
            INSERT INTO gateways (
                id, region, public_ip, control_port, tcp_start, tcp_end,
                udp_start, udp_end, reserved_ports_json, is_healthy, last_heartbeat, token, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                region = excluded.region,
                public_ip = excluded.public_ip,
                control_port = excluded.control_port,
                tcp_start = excluded.tcp_start,
                tcp_end = excluded.tcp_end,
                udp_start = excluded.udp_start,
                udp_end = excluded.udp_end,
                reserved_ports_json = excluded.reserved_ports_json,
                is_healthy = excluded.is_healthy,
                last_heartbeat = excluded.last_heartbeat,
                token = excluded.token
            "#,
            params![
                gw.id,
                gw.region,
                gw.public_ip,
                gw.control_port,
                gw.tcp_port_range_start,
                gw.tcp_port_range_end,
                gw.udp_port_range_start,
                gw.udp_port_range_end,
                reserved_json,
                if gw.is_healthy { 1 } else { 0 },
                last_hb,
                gw.token,
                created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_gateway(&self, id: &str) -> Result<Option<Gateway>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, region, public_ip, control_port, tcp_start, tcp_end,
                   udp_start, udp_end, reserved_ports_json, is_healthy, last_heartbeat, token, created_at
            FROM gateways WHERE id = ?1
            "#,
        )?;

        let gw = stmt
            .query_row(params![id], |row| {
                let reserved_json: String = row.get(8)?;
                let reserved_ports: Vec<u16> = serde_json::from_str(&reserved_json).unwrap_or_default();
                let is_healthy_int: i64 = row.get(9)?;
                let last_hb_str: Option<String> = row.get(10)?;
                let last_heartbeat = last_hb_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc)));
                let created_at_str: String = row.get(12)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Gateway {
                    id: row.get(0)?,
                    region: row.get(1)?,
                    public_ip: row.get(2)?,
                    control_port: row.get(3)?,
                    tcp_port_range_start: row.get(4)?,
                    tcp_port_range_end: row.get(5)?,
                    udp_port_range_start: row.get(6)?,
                    udp_port_range_end: row.get(7)?,
                    reserved_ports,
                    is_healthy: is_healthy_int == 1,
                    last_heartbeat,
                    token: row.get(11)?,
                    created_at,
                })
            })
            .optional()?;

        Ok(gw)
    }

    pub fn list_gateways(&self) -> Result<Vec<Gateway>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, region, public_ip, control_port, tcp_start, tcp_end,
                   udp_start, udp_end, reserved_ports_json, is_healthy, last_heartbeat, token, created_at
            FROM gateways ORDER BY id ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let reserved_json: String = row.get(8)?;
            let reserved_ports: Vec<u16> = serde_json::from_str(&reserved_json).unwrap_or_default();
            let is_healthy_int: i64 = row.get(9)?;
            let last_hb_str: Option<String> = row.get(10)?;
            let last_heartbeat = last_hb_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc)));
            let created_at_str: String = row.get(12)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Gateway {
                id: row.get(0)?,
                region: row.get(1)?,
                public_ip: row.get(2)?,
                control_port: row.get(3)?,
                tcp_port_range_start: row.get(4)?,
                tcp_port_range_end: row.get(5)?,
                udp_port_range_start: row.get(6)?,
                udp_port_range_end: row.get(7)?,
                reserved_ports,
                is_healthy: is_healthy_int == 1,
                last_heartbeat,
                token: row.get(11)?,
                created_at,
            })
        })?;

        let mut gateways = Vec::new();
        for r in rows {
            gateways.push(r?);
        }
        Ok(gateways)
    }

    pub fn update_gateway_heartbeat(&self, gateway_id: &str, is_healthy: bool) -> Result<()> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE gateways SET is_healthy = ?1, last_heartbeat = ?2 WHERE id = ?3",
            params![if is_healthy { 1 } else { 0 }, now, gateway_id],
        )?;
        Ok(())
    }

    // --- Node Operations ---

    pub fn upsert_node(&self, node: &Node) -> Result<()> {
        let conn = self.conn.lock();
        let last_hb = node.last_heartbeat.map(|dt| dt.to_rfc3339());
        let created_at = node.created_at.to_rfc3339();

        conn.execute(
            r#"
            INSERT INTO nodes (
                id, name, pterodactyl_node_id, assigned_gateway_id, local_ip,
                is_healthy, last_heartbeat, agent_token, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                pterodactyl_node_id = excluded.pterodactyl_node_id,
                assigned_gateway_id = excluded.assigned_gateway_id,
                local_ip = excluded.local_ip,
                is_healthy = excluded.is_healthy,
                last_heartbeat = excluded.last_heartbeat,
                agent_token = excluded.agent_token
            "#,
            params![
                node.id,
                node.name,
                node.pterodactyl_node_id as i64,
                node.assigned_gateway_id,
                node.local_ip,
                if node.is_healthy { 1 } else { 0 },
                last_hb,
                node.agent_token,
                created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_node(&self, id: &str) -> Result<Option<Node>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, pterodactyl_node_id, assigned_gateway_id, local_ip,
                   is_healthy, last_heartbeat, agent_token, created_at
            FROM nodes WHERE id = ?1
            "#,
        )?;

        let node = stmt
            .query_row(params![id], |row| {
                let ptero_id: i64 = row.get(2)?;
                let is_healthy_int: i64 = row.get(5)?;
                let last_hb_str: Option<String> = row.get(6)?;
                let last_heartbeat = last_hb_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc)));
                let created_at_str: String = row.get(8)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Node {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    pterodactyl_node_id: ptero_id as u64,
                    assigned_gateway_id: row.get(3)?,
                    local_ip: row.get(4)?,
                    is_healthy: is_healthy_int == 1,
                    last_heartbeat,
                    agent_token: row.get(7)?,
                    created_at,
                })
            })
            .optional()?;

        Ok(node)
    }

    pub fn get_node_by_ptero_id(&self, ptero_node_id: u64) -> Result<Option<Node>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, pterodactyl_node_id, assigned_gateway_id, local_ip,
                   is_healthy, last_heartbeat, agent_token, created_at
            FROM nodes WHERE pterodactyl_node_id = ?1
            "#,
        )?;

        let node = stmt
            .query_row(params![ptero_node_id as i64], |row| {
                let ptero_id: i64 = row.get(2)?;
                let is_healthy_int: i64 = row.get(5)?;
                let last_hb_str: Option<String> = row.get(6)?;
                let last_heartbeat = last_hb_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc)));
                let created_at_str: String = row.get(8)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Node {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    pterodactyl_node_id: ptero_id as u64,
                    assigned_gateway_id: row.get(3)?,
                    local_ip: row.get(4)?,
                    is_healthy: is_healthy_int == 1,
                    last_heartbeat,
                    agent_token: row.get(7)?,
                    created_at,
                })
            })
            .optional()?;

        Ok(node)
    }

    pub fn list_nodes(&self) -> Result<Vec<Node>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, pterodactyl_node_id, assigned_gateway_id, local_ip,
                   is_healthy, last_heartbeat, agent_token, created_at
            FROM nodes ORDER BY id ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let ptero_id: i64 = row.get(2)?;
            let is_healthy_int: i64 = row.get(5)?;
            let last_hb_str: Option<String> = row.get(6)?;
            let last_heartbeat = last_hb_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc)));
            let created_at_str: String = row.get(8)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Node {
                id: row.get(0)?,
                name: row.get(1)?,
                pterodactyl_node_id: ptero_id as u64,
                assigned_gateway_id: row.get(3)?,
                local_ip: row.get(4)?,
                is_healthy: is_healthy_int == 1,
                last_heartbeat,
                agent_token: row.get(7)?,
                created_at,
            })
        })?;

        let mut nodes = Vec::new();
        for r in rows {
            nodes.push(r?);
        }
        Ok(nodes)
    }

    pub fn update_node_heartbeat(&self, node_id: &str, is_healthy: bool) -> Result<()> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE nodes SET is_healthy = ?1, last_heartbeat = ?2 WHERE id = ?3",
            params![if is_healthy { 1 } else { 0 }, now, node_id],
        )?;
        Ok(())
    }

    // --- Allocation Operations ---

    pub fn upsert_allocation(&self, alloc: &Allocation) -> Result<()> {
        let conn = self.conn.lock();
        let protocol_str = alloc.protocol.to_string();
        let status_str = alloc.status.to_string();
        let created_at = alloc.created_at.to_rfc3339();
        let updated_at = alloc.updated_at.to_rfc3339();

        conn.execute(
            r#"
            INSERT INTO allocations (
                id, node_id, server_id, server_name, pterodactyl_allocation_id,
                local_ip, local_port, protocol, custom_alias, status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(node_id, pterodactyl_allocation_id) DO UPDATE SET
                server_id = excluded.server_id,
                server_name = excluded.server_name,
                local_ip = excluded.local_ip,
                local_port = excluded.local_port,
                protocol = excluded.protocol,
                custom_alias = excluded.custom_alias,
                status = excluded.status,
                updated_at = excluded.updated_at
            "#,
            params![
                alloc.id,
                alloc.node_id,
                alloc.server_id,
                alloc.server_name,
                alloc.pterodactyl_allocation_id as i64,
                alloc.local_ip,
                alloc.local_port,
                protocol_str,
                alloc.custom_alias,
                status_str,
                created_at,
                updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_allocation(&self, id: &str) -> Result<Option<Allocation>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, node_id, server_id, server_name, pterodactyl_allocation_id,
                   local_ip, local_port, protocol, custom_alias, status, created_at, updated_at
            FROM allocations WHERE id = ?1
            "#,
        )?;

        let alloc = stmt
            .query_row(params![id], |row| {
                let ptero_alloc_id: i64 = row.get(4)?;
                let proto_str: String = row.get(7)?;
                let status_str: String = row.get(9)?;
                let created_str: String = row.get(10)?;
                let updated_str: String = row.get(11)?;

                let protocol = match proto_str.as_str() {
                    "tcp" => Protocol::Tcp,
                    "udp" => Protocol::Udp,
                    "both" => Protocol::Both,
                    _ => Protocol::Auto,
                };

                let status = match status_str.as_str() {
                    "active" => AllocationStatus::Active,
                    "error" => AllocationStatus::Error,
                    "orphaned" => AllocationStatus::Orphaned,
                    "deleted" => AllocationStatus::Deleted,
                    _ => AllocationStatus::Pending,
                };

                Ok(Allocation {
                    id: row.get(0)?,
                    node_id: row.get(1)?,
                    server_id: row.get(2)?,
                    server_name: row.get(3)?,
                    pterodactyl_allocation_id: ptero_alloc_id as u64,
                    local_ip: row.get(5)?,
                    local_port: row.get(6)?,
                    protocol,
                    custom_alias: row.get(8)?,
                    status,
                    created_at: DateTime::parse_from_rfc3339(&created_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                })
            })
            .optional()?;

        Ok(alloc)
    }

    pub fn list_allocations(&self) -> Result<Vec<Allocation>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, node_id, server_id, server_name, pterodactyl_allocation_id,
                   local_ip, local_port, protocol, custom_alias, status, created_at, updated_at
            FROM allocations ORDER BY created_at DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let ptero_alloc_id: i64 = row.get(4)?;
            let proto_str: String = row.get(7)?;
            let status_str: String = row.get(9)?;
            let created_str: String = row.get(10)?;
            let updated_str: String = row.get(11)?;

            let protocol = match proto_str.as_str() {
                "tcp" => Protocol::Tcp,
                "udp" => Protocol::Udp,
                "both" => Protocol::Both,
                _ => Protocol::Auto,
            };

            let status = match status_str.as_str() {
                "active" => AllocationStatus::Active,
                "error" => AllocationStatus::Error,
                "orphaned" => AllocationStatus::Orphaned,
                "deleted" => AllocationStatus::Deleted,
                _ => AllocationStatus::Pending,
            };

            Ok(Allocation {
                id: row.get(0)?,
                node_id: row.get(1)?,
                server_id: row.get(2)?,
                server_name: row.get(3)?,
                pterodactyl_allocation_id: ptero_alloc_id as u64,
                local_ip: row.get(5)?,
                local_port: row.get(6)?,
                protocol,
                custom_alias: row.get(8)?,
                status,
                created_at: DateTime::parse_from_rfc3339(&created_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut allocs = Vec::new();
        for r in rows {
            allocs.push(r?);
        }
        Ok(allocs)
    }

    pub fn delete_allocation(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM allocations WHERE id = ?1", params![id])?;
        Ok(())
    }

    // --- Mapping Operations ---

    pub fn insert_mapping(&self, mapping: &Mapping) -> Result<()> {
        let conn = self.conn.lock();
        let proto_str = mapping.protocol.to_string();
        let created_at = mapping.created_at.to_rfc3339();
        let updated_at = mapping.updated_at.to_rfc3339();

        conn.execute(
            r#"
            INSERT INTO mappings (
                id, allocation_id, gateway_id, protocol, gateway_port,
                target_ip, target_port, fqdn, is_active, error_message, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                mapping.id,
                mapping.allocation_id,
                mapping.gateway_id,
                proto_str,
                mapping.gateway_port,
                mapping.target_ip,
                mapping.target_port,
                mapping.fqdn,
                if mapping.is_active { 1 } else { 0 },
                mapping.error_message,
                created_at,
                updated_at,
            ],
        )?;

        // Record in allocated_ports table for fast conflict and pool queries
        if mapping.protocol.is_tcp() {
            conn.execute(
                "INSERT OR REPLACE INTO allocated_ports (gateway_id, protocol, port, mapping_id, created_at) VALUES (?1, 'tcp', ?2, ?3, ?4)",
                params![mapping.gateway_id, mapping.gateway_port, mapping.id, created_at],
            )?;
        }
        if mapping.protocol.is_udp() {
            conn.execute(
                "INSERT OR REPLACE INTO allocated_ports (gateway_id, protocol, port, mapping_id, created_at) VALUES (?1, 'udp', ?2, ?3, ?4)",
                params![mapping.gateway_id, mapping.gateway_port, mapping.id, created_at],
            )?;
        }

        Ok(())
    }

    pub fn get_mapping_by_allocation(&self, allocation_id: &str) -> Result<Option<Mapping>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, allocation_id, gateway_id, protocol, gateway_port,
                   target_ip, target_port, fqdn, is_active, error_message, created_at, updated_at
            FROM mappings WHERE allocation_id = ?1
            "#,
        )?;

        let mapping = stmt
            .query_row(params![allocation_id], |row| {
                let proto_str: String = row.get(3)?;
                let protocol = match proto_str.as_str() {
                    "tcp" => Protocol::Tcp,
                    "udp" => Protocol::Udp,
                    "both" => Protocol::Both,
                    _ => Protocol::Auto,
                };
                let is_active_int: i64 = row.get(8)?;
                let created_str: String = row.get(10)?;
                let updated_str: String = row.get(11)?;

                Ok(Mapping {
                    id: row.get(0)?,
                    allocation_id: row.get(1)?,
                    gateway_id: row.get(2)?,
                    protocol,
                    gateway_port: row.get(4)?,
                    target_ip: row.get(5)?,
                    target_port: row.get(6)?,
                    fqdn: row.get(7)?,
                    is_active: is_active_int == 1,
                    error_message: row.get(9)?,
                    created_at: DateTime::parse_from_rfc3339(&created_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                })
            })
            .optional()?;

        Ok(mapping)
    }

    pub fn list_mappings_for_node(&self, node_id: &str) -> Result<Vec<Mapping>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            r#"
            SELECT m.id, m.allocation_id, m.gateway_id, m.protocol, m.gateway_port,
                   m.target_ip, m.target_port, m.fqdn, m.is_active, m.error_message, m.created_at, m.updated_at
            FROM mappings m
            JOIN allocations a ON m.allocation_id = a.id
            WHERE a.node_id = ?1 AND m.is_active = 1
            ORDER BY m.gateway_port ASC
            "#,
        )?;

        let rows = stmt.query_map(params![node_id], |row| {
            let proto_str: String = row.get(3)?;
            let protocol = match proto_str.as_str() {
                "tcp" => Protocol::Tcp,
                "udp" => Protocol::Udp,
                "both" => Protocol::Both,
                _ => Protocol::Auto,
            };
            let is_active_int: i64 = row.get(8)?;
            let created_str: String = row.get(10)?;
            let updated_str: String = row.get(11)?;

            Ok(Mapping {
                id: row.get(0)?,
                allocation_id: row.get(1)?,
                gateway_id: row.get(2)?,
                protocol,
                gateway_port: row.get(4)?,
                target_ip: row.get(5)?,
                target_port: row.get(6)?,
                fqdn: row.get(7)?,
                is_active: is_active_int == 1,
                error_message: row.get(9)?,
                created_at: DateTime::parse_from_rfc3339(&created_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut mappings = Vec::new();
        for r in rows {
            mappings.push(r?);
        }
        Ok(mappings)
    }

    pub fn list_mappings(&self) -> Result<Vec<Mapping>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, allocation_id, gateway_id, protocol, gateway_port,
                   target_ip, target_port, fqdn, is_active, error_message, created_at, updated_at
            FROM mappings ORDER BY created_at DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let proto_str: String = row.get(3)?;
            let protocol = match proto_str.as_str() {
                "tcp" => Protocol::Tcp,
                "udp" => Protocol::Udp,
                "both" => Protocol::Both,
                _ => Protocol::Auto,
            };
            let is_active_int: i64 = row.get(8)?;
            let created_str: String = row.get(10)?;
            let updated_str: String = row.get(11)?;

            Ok(Mapping {
                id: row.get(0)?,
                allocation_id: row.get(1)?,
                gateway_id: row.get(2)?,
                protocol,
                gateway_port: row.get(4)?,
                target_ip: row.get(5)?,
                target_port: row.get(6)?,
                fqdn: row.get(7)?,
                is_active: is_active_int == 1,
                error_message: row.get(9)?,
                created_at: DateTime::parse_from_rfc3339(&created_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut mappings = Vec::new();
        for r in rows {
            mappings.push(r?);
        }
        Ok(mappings)
    }

    pub fn delete_mapping(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM allocated_ports WHERE mapping_id = ?1", params![id])?;
        conn.execute("DELETE FROM mappings WHERE id = ?1", params![id])?;
        Ok(())
    }

    // --- Port Allocation Queries ---

    pub fn get_allocated_ports(&self, gateway_id: &str, protocol: &str) -> Result<std::collections::HashSet<u16>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT port FROM allocated_ports WHERE gateway_id = ?1 AND protocol = ?2",
        )?;

        let rows = stmt.query_map(params![gateway_id, protocol], |row| {
            let port: u16 = row.get(0)?;
            Ok(port)
        })?;

        let mut set = std::collections::HashSet::new();
        for r in rows {
            set.insert(r?);
        }
        Ok(set)
    }

    // --- Audit Logs ---

    pub fn add_audit_log(&self, log: &AuditLog) -> Result<()> {
        let conn = self.conn.lock();
        let created_at = log.created_at.to_rfc3339();
        conn.execute(
            "INSERT INTO audit_logs (id, event_type, resource_id, details, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![log.id, log.event_type, log.resource_id, log.details, created_at],
        )?;
        Ok(())
    }

    pub fn list_audit_logs(&self, limit: usize) -> Result<Vec<AuditLog>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, event_type, resource_id, details, created_at FROM audit_logs ORDER BY created_at DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let created_str: String = row.get(4)?;
            Ok(AuditLog {
                id: row.get(0)?,
                event_type: row.get(1)?,
                resource_id: row.get(2)?,
                details: row.get(3)?,
                created_at: DateTime::parse_from_rfc3339(&created_str).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut logs = Vec::new();
        for r in rows {
            logs.push(r?);
        }
        Ok(logs)
    }
}
