def listens(node, port):
    node.wait_until_succeeds(f"ss -tlnH 'sport = :{port}' | grep -q .", timeout=120)


def through(node, name, port):
    scheme = "https" if port == 443 else "http"
    node.wait_until_succeeds(
        f"curl -skf -o /dev/null --resolve {name}:{port}:127.0.0.1 {scheme}://{name}:{port}/",
        timeout=120,
    )


def answers(node, url):
    node.wait_until_succeeds(f"curl -s -o /dev/null {url}", timeout=120)


def page(node, name, path):
    return node.wait_until_succeeds(
        f"curl -sf --resolve {name}:80:127.0.0.1 http://{name}/{path}",
        timeout=120,
    )


def serves(node, name, hosts, endpoints):
    closures = page(node, name, "closures.html")
    headings = page(node, name, "hosts.html")
    for host in hosts:
        assert f"<code>{host}</code>" in closures, (
            f"{node.name} docs miss the {host} closure"
        )
        assert f"{host}</a></h2>" in headings, f"{node.name} docs miss the {host} host"
    rows = page(node, name, "endpoints.html").splitlines()
    for host, entry, port in endpoints:
        cells = [f"<code>{entry}</code>", f"<td>{port}</td>", f"<td>{host}</td>"]
        assert any(all(c in r for c in cells) for r in rows), (
            f"{node.name} docs miss {entry}:{port} on {host}"
        )


def reaches(src, dst, port, open):
    probe = f"timeout 3 bash -c '</dev/tcp/{dst.name}/{port}'"
    if open:
        src.succeed(probe)
    else:
        src.fail(probe)


def listens_only(node, lan, claimed):
    for line in node.succeed("ss -tlnH").splitlines():
        host, port = line.split()[3].rsplit(":", 1)
        reachable = host.strip("[]") in ["0.0.0.0", "::", "*", *lan]
        assert not reachable or int(port) in claimed, (
            f"{node.name} listens on {port}, no adapter claims it"
        )
