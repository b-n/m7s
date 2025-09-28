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

`yaml_parser` crate seems to do the trick and provides a CST. It's built ontop of rowan which is
used by rust-analyzer.

At first glance it doesn't look it's possible to edit the CST, so this is mostly going to be useful
only when doing yaml validation, and possibly openapiv3 schema validation. 

Other things ruled out

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

`kube-rs` provides a `kube-client` crate which wraps the kubernetes API nicely. Notes:

- `list_core_api_versions` to get the supported versions of core kubernetes apis
- `list_core_api_resources` to get availalbe core resources
- `list_api_groups` to get the api groups and supported/preferred versions
- `list_api_group_resources` to get the resource of a group at a specific version

This crate doesn't look like it provides methods for the openapiv3 spec endpoints, however it does
have a request builder, which means we can use that to serialize the Openapiv3 spec responses.
Notes:

- `$SERVER/openapi/v3` - gives a root reference to all the openapiv3 specs
  - `$SERVER/openapi/v3/api/v1` - gives the spec that the server has for core/v1 specs
  - `$SERVER/openapi/v3/apis/apps/v1` - gives the spec that the server has for apps/v1 as above
  - `$SERVER/openapi/v3/apis/<group>/<version>` - gives teh spec for the group and specific version
