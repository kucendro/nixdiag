<!-- Auto-generated from the Nix config by nixdiag. Do not edit. -->

# Hosts

## 🖥️ diddy

| | |
|---|---|
| Platform | `x86_64-linux` |
| State version | `24.05` |
| Users | admin |
| System packages | 124 |
| Open TCP ports | 22, 443 |
| Open UDP ports | — |
| Repo-configured services | 2 |

**Services** (configured in this repo):

- **grafana** — `modules/monitoring.nix`
- **tailscale** — `hosts/diddy/default.nix`

## 🖥️ epstein

Edge node: mesh control plane and the fleet's reverse proxy.

| | |
|---|---|
| Platform | `x86_64-linux` |
| State version | `24.05` |
| Users | admin |
| System packages | 124 |
| Open TCP ports | 22, 443 |
| Open UDP ports | — |
| Repo-configured services | 2 |

**Services** (configured in this repo):

- **headscale** — `modules/mesh.nix`
- **nginx** — `modules/web.nix`
