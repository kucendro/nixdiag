<!-- Auto-generated from the Nix config by nixdiag. Do not edit. -->

# Hosts

## 🖥️ luna

| | |
|---|---|
| Platform | `x86_64-linux` |
| State version | `24.05` |
| Users | admin |
| System packages | 124 |
| Open TCP ports | 22, 443 |
| Open UDP ports | — |
| Repo-configured services | 2 |

**Services:**

- **grafana** — `modules/monitoring.nix`
- **tailscale** — `hosts/luna/default.nix`

## 🖥️ sol

| | |
|---|---|
| Platform | `x86_64-linux` |
| State version | `24.05` |
| Users | admin |
| System packages | 124 |
| Open TCP ports | 22, 443 |
| Open UDP ports | — |
| Repo-configured services | 2 |

**Services:**

- **headscale** — `modules/mesh.nix`
- **nginx** — `modules/web.nix`
