SSEplex
=======

Current status: a toy project.

This is a project along the same lines as
[esper](https://github.com/mikeycgto/esper), a server sent event hub
written in Rust. A difference is that this one is using
[actix](https://actix.rs/) as the underlying framework.

So far there is JWT authentication (configured in the code), the
messages are just generated tick messages and it's not at all useful
except possibly to look at how I'm using actix.
