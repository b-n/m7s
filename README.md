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
- Able to be called directly from `k9s`...

# Meta

This section should die hopefully...

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
