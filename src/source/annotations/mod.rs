//! `#:` topology annotations — comment lines in the documented repo's own
//! module files. Parsed render-side with rnix (comments are invisible to
//! eval); attachment and edge targets resolve against the evaluated facts,
//! so annotations describe real state, not strings.
//!
//! Grammar (one statement per line; a malformed line is a reported error):
//!   #: <role>                                  role (implicit verb)
//!   #: expose <port>[/udp] [scope] [name=<fqdn>]
//!   #: -> <host[/service] | fqdn | internet | lan> [label] [name=<fqdn>[:port]]
//!      (and `<-`; `name=` marks the fronted endpoint the annotated node
//!      serves for that target — an Endpoints page row)
//!   #: name <fqdn>                             address-book entry
//!   #: scope public|mesh|lan
//!   #: unit <[host/]name>                      declare an unprojected node
//!      (the host pin is for files several hosts' import graphs reach)
//! Any fqdn position accepts `<sub>@<key>`: the domain map (CLI `--domain`,
//! flake `nixdiag.domains`, mkDocs `domains`) supplies the suffix at render
//! time, so the domain literal never has to appear in the repo source.
//! A `unit` in the file-leading doc comment is the file's default attachment:
//! file-level lines anywhere in that file attach to it (per-binding
//! attachment still wins), so data files feeding a service defined elsewhere
//! can carry their annotations next to the data.
//! `# nixdiag:` is the long alias; the same lines are recognized inside a
//! file-leading `/** */` doc comment. A contiguous run of annotation lines
//! forms one block; a `unit` declaration re-attaches its whole block.

mod attach;
mod diag;
mod grammar;
mod model;
mod resolve;
mod scan;
mod stmt;

pub use diag::Sev;
pub use grammar::{resolve_edition, VERSION};
pub use model::{Endpoint, Model, NodeInfo, Scope};
pub use resolve::collect;
