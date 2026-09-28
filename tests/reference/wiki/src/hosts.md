# Hosts

## 🖥️ jerry

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

<div class="d2 d2-light">{{#include modules-jerry-light.svg}}</div>
<div class="d2 d2-dark">{{#include modules-jerry-dark.svg}}</div>

## 🖥️ tom

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
- **tailscale** — `hosts/tom/default.nix`

**Modules:**

<div class="d2 d2-light">{{#include modules-tom-light.svg}}</div>
<div class="d2 d2-dark">{{#include modules-tom-dark.svg}}</div>
