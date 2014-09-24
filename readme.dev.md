# Asset Store

[Api Documentation](tyoverby.com/asset_store/asset_store/index.html)

A unified method for easily reading and caching files over the filesystem
and network.

Calls to `load()` process asynchronously, so it is possible to load files
from different sources in parallel.

### Read files from disk

When reading a files out of a directory store, it is impossible to read outside
of the directory specified.

^code(examples/basic_file.rs)

### Read files over http

You can also read files off of a web server.

^code(examples/basic_web.rs)

### Read files out of memory

If your files are small enough, you can bundle them into your binary by using
[resources-package](https://github.com/tomaka/rust-package.git).

^code(examples/static_resources.rs)

### Combine different stores into one.

Having multiple stores laying around for different sources is a pain, so
by combining them into one and using prefixes you can access many
file stores of different types.

^code(examples/multi.rs)
