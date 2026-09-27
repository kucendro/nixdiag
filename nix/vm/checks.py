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


def reaches(src, dst, port, open):
    probe = f"timeout 3 bash -c '</dev/tcp/{dst.name}/{port}'"
    if open:
        src.succeed(probe)
    else:
        src.fail(probe)


def listens_only(node, claimed):
    for line in node.succeed("ss -tlnH").splitlines():
        host, port = line.split()[3].rsplit(":", 1)
        loopback = host.startswith(("127.", "[::1]"))
        assert loopback or int(port) in claimed, (
            f"{node.name} listens on {port}, no adapter claims it"
        )
