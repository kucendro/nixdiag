# Firewall

## tom {#firewall-tom}

| Interface       | TCP         | UDP        |
|-----------------|-------------|------------|
| every interface | `22`, `443` | —          |
| `tailscale0`    | `3000`      | —          |
| `lo` (trusted)  | every port  | every port |

**Findings:**

- `22/tcp` is open on every interface, but nothing here uses it
- `443/tcp` is open on every interface, but nothing here uses it

## jerry {#firewall-jerry}

| Interface       | TCP         | UDP        |
|-----------------|-------------|------------|
| every interface | `22`, `443` | —          |
| `lo` (trusted)  | every port  | every port |

**Findings:**

- `22/tcp` is open on every interface, but nothing here uses it
