# Firewall

## tom {#firewall-tom}

| Interface       | TCP         | UDP        |
|-----------------|-------------|------------|
| every interface | `22`, `443` | —          |
| `lo` (trusted)  | every port  | every port |

**Findings:**

- `22/tcp` is open on every interface, but nothing here uses it
- `443/tcp` is open on every interface, but nothing here uses it
- `grafana` exposes `3000/tcp` to mesh, but it is closed on `tailscale0`
- `jerry/nginx` connects to `3000/tcp`, but it is closed on `eth0`, `tailscale0`

## jerry {#firewall-jerry}

| Interface       | TCP         | UDP        |
|-----------------|-------------|------------|
| every interface | `22`, `443` | —          |
| `lo` (trusted)  | every port  | every port |

**Findings:**

- `22/tcp` is open on every interface, but nothing here uses it
