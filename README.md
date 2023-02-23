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

## Getting Started
First things first, you will need to get your webhook setup!
You will need to go to the repo you wish to deploy descry to, go to settings, followed by webhooks and **Add Webhook**.

Then, in your payload URL - you will need to put where you have deployed descry to. It is suggested to use a service such as nginx to map your port deployment to a `/hook` subroute. I.e. `https://reseda.app/hook`. Leave the type as url-encoded.

Lastly, you will need to generate a secret. This should never be exposed. An example of what one should look like is shown in `descry.yaml`. This needs to be put in your yaml config wherever descry is deployed.

Then, simply chose what events you want this hook to listen to and activate it!

## Documentation
The `descry.yaml` file contains everything needed for this project, lets walk through it.

### Settings
- **ip:port** `host`:  Host defines the location where descry will try to deploy itself. If you need a custom port, specify it here.
- **string** `secret` It's your action secret!
- **bool** `print_commands`  Leave this as `true` if you wish to keep a log of any commands run.
- **bool** `exit_on_error`  Should descry quit when recieving an error?

### Events
Every event is the name it is given from GitHub. The following supported tags are:

|Tag | Meaning  | RunType|
--- | --- | ---|
|common|Is common to all events, is run on every action, as well as the action's specific tag. |on-every|
|all| Same as common. |on-every|
|push|Run on push. |on-specific|
|watch|Run on every new user watching repository, to find out who; check the hydrated tags.|on-specific|
|ping|Run on GitHub Action ping command, useful for testing.|on-specific|
|else|Anything... else...|on-specific|

Inside, you can put bash code (.sh) that will be run whenever the event is called.
The following properties can be used, which will be hydrated upon event run:
- `{id}` ID of the action
- `{event}` Action's event type
- `{signature}` Action's signature
- `{payload}` Action's payload content
- `{request_body}` Action's request body

## What could I build with this?
There's infinite possibilities, but here's a few suggestions to get the ball rolling!
- Build &/or deploy code on a succesful commit to master.
- Notify your server of a new watcher, or stargazer; thank them!
- Keep yourself (and others) up to date with changes to the repo, like issues or stars.
- Let everyone know about a new version release!
- Many more!


-----
*Build for the [`reseda-vpn`](https://github.com/reseda-vpn) project*
