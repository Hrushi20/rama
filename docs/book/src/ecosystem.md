# 📣 Rama Ecosystem

For now there are only the rama crates found in this repository, also referred to as "official" rama crates.

We welcome however community contributions not only in the form of contributions to this repository,
but also have people write their own crates as extensions to the rama ecosystem.
E.g. perhaps you wish to support an alternative http/tls backend.

In case you have ideas for new features or stacks please let us know first.
Perhaps there is room for these within an official rama crate.
In case it is considered out of scope you are free to make your own community rama crate.
Please prefix all rama community crates with "rama-x", this way the crates are easy to find,
and are sufficiently different from "official" rama crates".

Once you have such a crate published do let us know it, such that we can list them here.

## 📦 | Rama Crates

The `rama` crate can be used as the one and only dependency.
However, as you can also read in the "DIY" chapter of the book
at <https://ramaproxy.org/book/diy.html#empowering>, you are able
to pick and choose not only what specific parts of `rama` you wish to use,
but also in fact what specific (sub) crates.

Here is a list of all `rama` crates:

- [`rama`](https://crates.io/crates/rama): one crate to rule them all
- [`rama-error`](https://crates.io/crates/rama-error): error utilities for rama and its users
- [`rama-macros`](https://crates.io/crates/rama-macros): contains the procedural macros used by `rama`
- [`rama-utils`](https://crates.io/crates/rama-utils): utilities crate for rama
- [`rama-core`](https://crates.io/crates/rama-core): core crate containing the service, layer and
  context used by all other `rama` code, as well as some other _core_ utilities
- [`rama-net`](https://crates.io/crates/rama-net): rama network types and utilities
- [`rama-dns`](https://crates.io/crates/rama-dns): DNS support for rama
- [`rama-tcp`](https://crates.io/crates/rama-tcp): TCP support for rama
- [`rama-tls`](https://crates.io/crates/rama-tls): TLS support for rama (types, `rustls` and `boring`)
- [`rama-proxy`](https://crates.io/crates/rama-proxy): proxy types and utilities for rama
- [`rama-haproxy`](https://crates.io/crates/rama-haproxy): rama HaProxy support
- [`rama-ua`](https://crates.io/crates/rama-ua): User-Agent (UA) support for `rama`
- [`rama-http-types`](https://crates.io/crates/rama-http-types): http types and utilities
- [`rama-http`](https://crates.io/crates/rama-http): rama http services, layers and utilities
- [`rama-http-backend`](https://crates.io/crates/rama-http-backend): default http backend for `rama`