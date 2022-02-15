# cloud-op

cita-cloud 的运维工具提供：消块功能（已支持）以及备份功能（未来支持）。

```shell
$ cloud-op --help
cloud-op 0.1.0
Simple program to greet a person

USAGE:
    cloud-op <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    backup     hot backup executor_evm & chain db, ONLY USE IN EVM MODE
    help       Print this message or the help of the given subcommand(s)
    recover    recover chain status to specified height, ONLY USE IN EVM MODE
```



## cita-cloud version

cita-cloud version >= v6.3.3 or choice branch: support-op


## function
### backup

未来支持

### recover

:rotating_light: **该功能仅支持选择 `EXECUTOR_EVM` 作为执行微服务的链使用，如果使用 `KMS_ETH` 加密微服务，请更换 `features` 后，重新编译再使用。**

#### 消块功能命令说明：

```shell
$ cloud-op recover --help
cloud-op-recover 
recover chain status to specified height, ONLY USE IN EVM MODE

USAGE:
    cloud-op recover -c <CONFIG_PATH> -n <NODE_ROOT> <HEIGHT>

ARGS:
    <HEIGHT>    the specified height that you want to recover to

OPTIONS:
    -c <CONFIG_PATH>        chain config path [default: config.toml]
    -h, --help              Print help information
    -n <NODE_ROOT>          node root path
```

#### 示例：

```shell
$ cloud-op recover 2 -n <YOUR_NODE_PATH> -c <YOUR_CONFIG_PATH>
```

#### 示例结果：

```
lock_id1000 never change from genesis
lock_id1001 never change from genesis
lock_id1002 never change from genesis
lock_id1003 never change from genesis
lock_id1004 never change from genesis
lock_id1005 never change from genesis
lock_id1006 never change from genesis
```

