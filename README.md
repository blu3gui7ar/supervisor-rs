# supervisor-rs #

`supervisor-rs` used to be a manager of handle application running. `supervisor-rs` can start/restart/stop processing.

**Features**:

+ start different processing depend on particular yaml file when startup
+ start processing when have new config in load path
+ startup with particular server config
+ restart processing
+ stop processing

**Design**:

1. server/client mode
2. server start -> load config files from loadpaths (if it is not quiet mode) -> do job
3. start/stop/restart/tryrestart special processing (client side)

**Config yaml files format**:

server.yaml:

```yaml
#server side config
loadpaths:
  - /tmp/client/
  - /tmp/second/path
  
mode: "full"
```

each command's config:

```yaml
#each child config in loadpath of server config
command: /tmp/test
output:
  - stdout: aaaaaa
    mode: create

  - stderr: nnnnn
    mode: append
```

## Usage ##

You can download compiled binary file directly on release tag.

You can install from cargo.io, run `cargo install supervisor-rs`. Or you can build it by yourself.

### Server Side ###

Start server side application. After compiled, run `supervisor-rs-server /tmp/server.yml` in shell, you can change server config yaml file to wherever you want. If no config path given, supervisor will going to find `server.yml` in `/tmp`.

After server application start, if `mode` is **full**, then all **application yaml files under loadpath of server config** will be ran by application. So, that's means every yaml files in there should be legal application config file, or server cannot start.

Server side's default mode is `quiet`, means server will record `loadphths`, but won't start children automatically.

Each sub-processing is named with **filename** of yaml file. If have multi-loadpath, make sure **no yaml files have same name**. 

**command demo:**

run server with special config file:
`supervisor-rs-server ./test/server.yml` 

### Client Side ###

**Operate child processing**:

`supervisor-rs-client restart child0 on localhost` will restart processing `child0` on localhost;

`supervisor-rs-client restart child0 on 198.0.0.2` will restart processing `child0` on `192.0.0.2`, I assume you running server side application on this host;

`supervisor-rs-client restart child0 on "198.0.0.2, 198.0.0.3"` will restart processing `child0` on `192.0.0.2` and `192.0.0.3`, I assume you running server side application on these hosts;

After version `0.6` command upper equal with `supervisor-rs-client restart child0 on 198.0.0.2 on 198.0.0.3`

child name is not must for `check`/`kill` commands.

commands:

| command  | behavior                                                                                                                                                                                                                                                                                   |
| ---      | ---                                                                                                                                                                                                                                                                                        |
| restart  | restart child on server. this child has to be running (server application). Otherwise, use start instead                                                                                                                                                                                   |
| start    | start new child. This command can start one-time command, or new config just put in loadpath(s). And, start does not care what's happen in child itself. If it start and panic immediately, supervisor will return success message anyway. Use `check` command to check if it runs or not. |
| stop     | stop running child. Have to supply child name. If want to stop all children, use `stop all`                                                                                                                                                                                                |
| check    | return summary of all children who are **running**. If children are not running, no matter what reason, they will be cleaned from kindergarden's table.                                                                                                                                    |
| trystart | special command for CI/CD to start child processings. `restart` only works when child is running; `start` only works when child is not running. `trystart` will run child processing anyway, if it is running, restart; if it is not running, start it.                                    |
| kill     | kill will terminate server and return last words from server                                                                                                                                                                                                                               |

### Use key pairs authenticate clients ###

When server side turn on encrypt mode, server side will check if data it received can decrypt by public keys in `pub_keys_path`

#### Example: ####

**Server side configuration**:

```yaml
loadpaths:
  - /tmp/client/
  - /tmp/second/path
  
encrypt: "on"
pub_keys_path:
  - /tmp/pub_keys/
  - /tmp/pub_keys2/
```

When server start with `encrypt: "on"` (only support lowercase), server side will pick key's name out from command received from client and find same `filename` public key (only support `.pem` file) in the `pub_keys_path`. As same as children names, key's name also equal the key file's name. So, make sure there ain't any key files have same names. 


**Client side command**

On client side, just run `supervisor-rs-client restart child0 on 198.0.0.2 on 198.0.0.3 with /path/to/key/keyname1.pem`. 

Then supervisor will go find key file has named `keyname1`. As flexible as you can change child config after supervisor start, you can also put public key files in `pub_keys_path` while supervisor is running.

**Make keypairs**

Step 1: Make private key
`openssl genrsa -out private.pem 4096`

Remember: key size should less or equal 4096


Step 2: Make public key
`openssl rsa -in private.pem -outform PEM -pubout -out public.pem`

Then, put public key in one of server's `pub_keys_path`. Every commands you send to server side should has `with /path/to/private.pem`.


**FYI**

You cannot change encrypt mode when supervisor-rs running.

### Startup-with feature ###

If `server's config` mode is `half`, server will try to startup all children in `startup` list when it starts.

Demo:

```yaml
#server side config
loadpaths:
  - /tmp/client/
  - /tmp/second/path
  
mode: "half"
startup:
  - child1
  - child2
  - child3
```

server will try to start `child1`, `child2`, and `child3` when it startup

**QA:**

Q: if child3 not exist?

A: server will going to find children in `startup`, if some of them not exist in loadpaths, server will skip them.

Q: what if I forget write mode to half?

A: server default mode is quiet, and startup list only used in `half` mode, otherwise, `startup` won't effect anything.


### Repeat feature ###

if config of child has `repeat` field:

```yaml
#file name (child name) is demo.yml
command: /tmp/test
output:
  - stdout: aaaaaa
    mode: create

  - stderr: nnnnn
    mode: append
repeat:
  action: restart
  seconds: 5 #only support seconds now
  
```

then, when you start it, `supervisor-rs` will give a timer stand by and send back command when time is up. For example, config above will run `restart demo` every 5 seconds. 

`action`'s values are command we using in supervisor-rs-client, so you even can `stop` child with `repeat` field. But so, `supervisor-rs` won't create a timer stand by. 

Only `start`, `restart`, and `trystart` will let `supervisor-rs` create a timer, if `repeat` field exists.

If `action` is empty, `supervisor-rs` will give `restart` be default value. However, `seconds` has to have value, and it cannot be 0.


#### How to stop repeat ####

As I said above, `timer` be created right after child runs. So you cannot stop "next" action, but if you change child's config, like delete repeat field, then "next" action won't create a timer. 

This is because all `start`, `restart` and `trystart` will **reload** config of child before it does its job.

So, what will supervisor do if child has `stopped`, or `restart` manually before timer finish its waiting and send command to supervisor again, timer isn't outdated? Timer will check if child has same processing id as when it created timer. If this check passed, timer will do its job as normal, else, timer won't do anything because child current is not child before.

### Hooks feature ###

Each child can have two hooks, one `prehook`, one `posthook`. `prehook` command will run before main child `start`/`restart`. `posthook` will run after child `stop`.

If prehooks command child has anther prehook, means there is a prehooks chain, they will run one by one, and they cannot have hooks circle.

example:

```yaml
command: sleep 10
hooks:
  - prehook: start child
  - posthook: start child
```

### What if accident happens ###

* if supervisor-rs be killed by `kill`, children won't stop, they will be taken by system.
* if supervisor-rs panic, children won't stop.

Go to log to find more information if `supervisor-rs-server` have problem

## Cross compiling ##

`brew tap filosottile/musl-cross && brew install FiloSottile/musl-cross/musl-cross`

after install `musl-cross`, `which x86_64-linux-musl-gcc` will give a result, like `/usr/local/bin/x86_64-linux-musl-gcc`.

give configuration in `~/.cargo/config`

```
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
```

then, `cargo build --target=x86_64-unknown-linux-musl`, there is no errors in my local machine.
