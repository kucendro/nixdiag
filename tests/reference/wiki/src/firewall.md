# Firewall

_Generated from the configuration as of 2026-09-21 14:13 UTC, `0123abc`._

## tom {#firewall-tom}

| Interface       | TCP         | UDP        |
|-----------------|-------------|------------|
| every interface | `22`, `443` | —          |
| `tailscale0`    | `3000`      | —          |
| `lo` (trusted)  | every port  | every port |

**Findings:**

- `443/tcp` is open on every interface, but nothing here uses it

## jerry {#firewall-jerry}

| Interface       | TCP         | UDP        |
|-----------------|-------------|------------|
| every interface | `22`, `443` | —          |
| `lo` (trusted)  | every port  | every port |

No findings.
