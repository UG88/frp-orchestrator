# Fast Reverse Proxy (FRP) Integration Specs

## Invariants for AI Coding Agents

1. **FRP Version Target**:
   - Modern FRP (v0.50+ and v0.60+) using TOML configuration format.
   - Do NOT use legacy INI configuration syntax (`frpc.ini` / `frps.ini`).

2. **Main Configuration Structure (`frpc.toml`)**:
   ```toml
   serverAddr = "<GATEWAY_IP>"
   serverPort = 7000
   auth.method = "token"
   auth.token = "<TOKEN>"
   transport.tls.enable = true

   webServer.addr = "127.0.0.1"
   webServer.port = 7400
   webServer.user = "admin"
   webServer.password = "<ADMIN_PASS>"

   includes = ["/opt/frp/conf.d/*.toml"]
   ```

3. **Proxy TOML Block Format (`conf.d/<name>.toml`)**:
   ```toml
   [[proxies]]
   name = "mc_<id>_tcp"
   type = "tcp"
   localIP = "<LOCAL_IP>"
   localPort = <LOCAL_PORT>
   remotePort = <GATEWAY_PORT>
   ```
   For `both`, generate two `[[proxies]]` blocks in the same file: one for `type = "tcp"` and one for `type = "udp"`.

4. **Hot-Reload Trigger**:
   - Issue authenticated `GET` / `POST` request to `http://127.0.0.1:7400/api/reload`.
   - Fallback: Execute `frpc reload -c /opt/frp/frpc.toml`.
