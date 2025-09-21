m7s - m(anifest)s

A tool to help with the generation of kubernetes manifests

# Running

```bash
cargo run m7s
```

# Goals and non goals

Goals:

- TUI interface for helping devs to generate manifests
- Support of core kubernetes resources, along with CRDs
- Loading and saving existing manifests whilst preserving formatting

Non-goals:

- YAML ast parser (although we might need to make one...)

# Meta

This section should die hopefully...

## YAML parsing

Parsing YAML seems to be a solved problem. Doing round trip does not.

Possible crates providing round trip:

- https://crates.io/crates/yaml-edit
  - Can't find the source anywhere, looks like author killed it?
  - Can't handle objects in arrays. Get's into some sort of loop/pin's a core
- serde-yaml
  - Doesn't preserve formatting/whitespace/comments
- https://crates.io/crates/rust-yaml
  - Ported from a PyPI library by the looks
  - No commits/updates since original posting
  - Initial commit = the whole project. Will keep an eye to see if it ever gets updates.

## Links

- $SERVER/api/v1 gives a list of the core types (e.g. pods, services, configmap)
- $SERVER/apis/ gives a list of the api groups
- $SERVER/apis/$GROUP gives a list of the typs in the group (e.g. /apis/apps/v1 has deployments, daemonsets, statefulsets)
- $SERVER/apis/apiextensions.k8s.io/v1/customresourcedefinitions/ gives all the CRDS with their openapiv3 specs

- $SERVER/openapi/v3 - gives a root reference to all the openapiv3 specs
  - $SERVER/openapi/v3/api/v1 - gives the spec that the server has for core/v1 as above
  - $SERVER/openapi/v3/apis/apps/v1 - gives the spec that the server has for apps/v1 as above
