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


def serves(node, name, hosts):
    page = node.wait_until_succeeds(
        f"curl -sf --resolve {name}:80:127.0.0.1 http://{name}/closures.html",
        timeout=120,
    )
    for host in hosts:
        assert f"<code>{host}</code>" in page, f"{node.name} docs miss the {host} closure"


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
