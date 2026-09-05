# Demo contract

## CLI entry point

Run `kpe demo`. The command needs no kubectl, kubeconfig, cluster, or network.
It creates a new directory under the operating system's temporary directory
and prints that path.

Use `kpe demo --output <new-directory>` for a chosen location. The command
refuses to reuse an existing directory, so it cannot overwrite real files.

The output contains:

- `rbac-snapshot.json`: one Role, one ClusterRole, and two bindings;
- `matrix.json`: four bounded access questions for Alice;
- `evidence.md`: the readable packet; and
- `evidence.json`: the structured packet.

The expected result is two allowed and two denied checks. The sample is built
into the binary from `examples/` and never contacts a cluster.

## Browser entry point

Open <https://kube-permission-evidence.sociobot.in/demo/>. One click from the
landing page reaches this route.

The persistent banner reads **Demo — sample data, nothing is saved**. The
page starts with a populated allowed result and offers all four sample checks.

Browser demo state uses only the `demo:kpe:` session-storage namespace.
**Reset demo** deletes that namespace and restores the first sample.
**Start for real** deletes demo state and returns to `/`.

Demo mode never reads or writes the `sb_license:` real-data namespace. The
sample makes no request to kubectl, a Kubernetes API, or any external origin.
