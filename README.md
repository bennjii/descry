# `descry`

A Github Action-Based CLI code executor for dynamic deployment of code.
> This project is built upon RedL0tus' [`rifling`](https://github.com/RedL0tus/rifling) rust crate, handling the Github Webhook Parsing.

## Installation
Install the crate from cargo (this requires [rust](https://www.rust-lang.org/tools/install) to be installed)

```
cargo install descry
```

Once installed, simply run

```
descry -c <file>.yaml
```

Where `<file>` represents the name of the configuration file. 
For a reference configuration file, see `descry.yaml` in the projects root directory.


-----
*Build for the [`reseda-vpn`](https://github.com/reseda-vpn) project*
