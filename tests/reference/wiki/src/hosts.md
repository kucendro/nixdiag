# Hosts

## 🖥️ jerry {#host-jerry}

|                          |                                             |
|--------------------------|---------------------------------------------|
| Location                 | home                                        |
| Platform                 | `x86_64-linux`                              |
| State version            | `24.05`                                     |
| Users                    | admin                                       |
| System packages          | 125                                         |
| Firewall                 | [1 finding](./firewall.html#firewall-jerry) |
| Default gateway          | `192.168.1.1` via `eth0`                    |
| Repo-configured services | 3                                           |

**Interfaces:**

| Interface    | Kind     | Addresses         | Over   |
|--------------|----------|-------------------|--------|
| `eth0`       | physical | `192.168.1.10/24` | —      |
| `iot`        | vlan 20  | `192.168.20.1/24` | `eth0` |
| `tailscale0` | mesh     | —                 | —      |

**Services:**

- **headscale** — `modules/mesh.nix`
- **nginx** — `modules/web.nix`
- **tailscale** — `hosts/jerry/default.nix`

**Topology:**

<div class="d2 d2-light">{{#include topology-jerry-light.svg}}</div>
<div class="d2 d2-dark">{{#include topology-jerry-dark.svg}}</div>

**Modules:**

<div class="d2 d2-light">{{#include modules-jerry-light.svg}}</div>
<div class="d2 d2-dark">{{#include modules-jerry-dark.svg}}</div>

## 🖥️ tom {#host-tom}

|                          |                                            |
|--------------------------|--------------------------------------------|
| Location                 | home                                       |
| Platform                 | `x86_64-linux`                             |
| State version            | `24.05`                                    |
| Users                    | admin                                      |
| System packages          | 124                                        |
| Firewall                 | [2 findings](./firewall.html#firewall-tom) |
| Default gateway          | `192.168.1.1` via `eth0`                   |
| Repo-configured services | 2                                          |

**Interfaces:**

| Interface    | Kind     | Addresses         | Over |
|--------------|----------|-------------------|------|
| `eth0`       | physical | `192.168.1.20/24` | —    |
| `tailscale0` | mesh     | —                 | —    |

**Services:**

- **grafana** — `modules/monitoring.nix`
- **tailscale** — `hosts/tom/default.nix`

**Topology:**

<div class="d2 d2-light">{{#include topology-tom-light.svg}}</div>
<div class="d2 d2-dark">{{#include topology-tom-dark.svg}}</div>

**Modules:**

<div class="d2 d2-light">{{#include modules-tom-light.svg}}</div>
<div class="d2 d2-dark">{{#include modules-tom-dark.svg}}</div>
