# Hosts

## 🖥️ luna

|                          |                |
|--------------------------|----------------|
| Platform                 | `x86_64-linux` |
| State version            | `24.05`        |
| Users                    | admin          |
| System packages          | 124            |
| Open TCP ports           | 22, 443        |
| Open UDP ports           | —              |
| Repo-configured services | 2              |

**Services:**

- **grafana** — `modules/monitoring.nix`
- **tailscale** — `hosts/luna/default.nix`

**Modules:**

<div class="d2 d2-light">{{#include modules-luna-light.svg}}</div>
<div class="d2 d2-dark">{{#include modules-luna-dark.svg}}</div>

## 🖥️ sol

|                          |                |
|--------------------------|----------------|
| Platform                 | `x86_64-linux` |
| State version            | `24.05`        |
| Users                    | admin          |
| System packages          | 124            |
| Open TCP ports           | 22, 443        |
| Open UDP ports           | —              |
| Repo-configured services | 2              |

**Services:**

- **headscale** — `modules/mesh.nix`
- **nginx** — `modules/web.nix`

**Modules:**

<div class="d2 d2-light">{{#include modules-sol-light.svg}}</div>
<div class="d2 d2-dark">{{#include modules-sol-dark.svg}}</div>
