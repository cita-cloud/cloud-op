# cloud-op

cita-cloud 的运维工具提供：消块功能以及备份功能。

:rotating_light: **该功能仅支持选择 `EXECUTOR_EVM` 作为执行微服务的链使用**

```shell
$ cloud-op --help
cloud-op 0.2.2
Yieazy <yuitta@163.com>
Simple program to greet a person

USAGE:
    cloud-op <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help             Print this message or the help of the given subcommand(s)
    recover          recover chain status to specified height, ONLY USE IN EVM MODE
    state-backup     hot backup executor_evm & chain db, ONLY USE IN EVM MODE
    state-recover    recover chain from early state, ONLY USE IN EVM MODE
```

## cita-cloud version

cita-cloud version choice: v6.5.0

## 部署

### k8s 部署

```shell
$ cd cloud-op
$ vim cloud-op.yaml
```

修改`cloud-op.yaml`中的`spec.containers.name`字段为需要的PodName，以及`spec.volumes.persistentVolumeClaim.claimName`字段为**需要操作节点**挂载PvcName。

```shell
$ kubectl apply -f cloud-op.yaml
```

##### ### 本地编译

```shell
$ cd cloud-op
$ cargo install --path .
```


## function

### recover

消块功能：将节点数据库中区块数据消块致指定高度的状态，**当前块高>=指定的消块块高**

#### 消块功能命令说明：

```shell
$ cloud-op recover --help
cloud-op-recover 
recover chain status to specified height, ONLY USE IN EVM MODE

USAGE:
    cloud-op recover --config-path <CONFIG_PATH> --node-root <NODE_ROOT> --crypto <CRYPTO> --consensus <CONSENSUS> <HEIGHT>

ARGS:
    <HEIGHT>    the specified height that you want to recover to

OPTIONS:
    -c, --config-path <CONFIG_PATH>    chain config path [default: config.toml]
        --consensus <CONSENSUS>        choice consensus server, bft or raft [default: bft]
        --crypto <CRYPTO>              choice crypto server, sm or eth [default: sm]
    -h, --help                         Print help information
    -n, --node-root <NODE_ROOT>        node root path [default: .]
```

#### 示例：

##### 执行

```shell
$ cd <YOUR_NODE_PATH> // eg. /mnt/test-chain-0
$ cloud-op recover 2 // ues default node-root and config-path
```

#### 示例结果：

```
lock_id(1000) never change from genesis
lock_id(1001) never change from genesis
lock_id(1002) never change from genesis
lock_id(1003) never change from genesis
lock_id(1004) never change from genesis
lock_id(1005) never change from genesis
lock_id(1006) never change from genesis
```

### state-backup

快照备份功能：备份指定块高的`executor`状态数据，不包含区块数据（区块，交易）

```shell
$ cloud-op state-backup --help
cloud-op-state-backup 
hot backup executor_evm & chain db, ONLY USE IN EVM MODE

USAGE:
    cloud-op state-backup --config-path <CONFIG_PATH> --node-root <NODE_ROOT> --backup-path <BACKUP_PATH> --crypto <CRYPTO> --consensus <CONSENSUS> <HEIGHT>

ARGS:
    <HEIGHT>    

OPTIONS:
    -b, --backup-path <BACKUP_PATH>    backup path dir [default: backup/state]
    -c, --config-path <CONFIG_PATH>    chain config path [default: config.toml]
        --consensus <CONSENSUS>        choice consensus server, bft or raft [default: bft]
        --crypto <CRYPTO>              choice crypto server, sm or eth [default: sm]
    -h, --help                         Print help information
    -n, --node-root <NODE_ROOT>        node root path [default: .]

```

#### 示例：

##### 执行

```shell
$ cloud-op state-backup -c config/test-chain-4/config.toml 30 -n config/test-chain-4 
```

#### 示例结果：

```
extract_backup finish.
```

```shell
$ tree backup

backup/
└── state
    └── 6
        ├── 000005.log
        ├── CURRENT
        ├── IDENTITY
        ├── LOCK
        ├── LOG
        ├── MANIFEST-000004
        ├── OPTIONS-000019
        └── OPTIONS-000021

2 directories, 8 files
```

### state-recover

```shell
$ cloud-op state-recover --help
cloud-op-state-recover 
recover chain from early state, ONLY USE IN EVM MODE

USAGE:
    cloud-op state-recover --config-path <CONFIG_PATH> --node-root <NODE_ROOT> --backup-path <BACKUP_PATH> --crypto <CRYPTO> --consensus <CONSENSUS> <HEIGHT>

ARGS:
    <HEIGHT>    

OPTIONS:
    -b, --backup-path <BACKUP_PATH>    backup path dir [default: backup/state]
    -c, --config-path <CONFIG_PATH>    chain config path [default: config.toml]
        --consensus <CONSENSUS>        choice consensus server, bft or raft [default: bft]
        --crypto <CRYPTO>              choice crypto server, sm or eth [default: sm]
    -h, --help                         Print help information
    -n, --node-root <NODE_ROOT>        node root path [default: .]
```

快照恢复功能：经`state-backup`备份快照数据恢复，需要指定备份数据的块高，经快照恢复的节点会失去历史状态信息，也因此节省硬盘空间

#### 示例：

##### 执行

```shell
$ cloud-op state-recover -c config/test-chain-3/config.toml 6 -n config/test-chain-3
```

#### 示例结果：

```
lock_id(1000) never change from genesis
lock_id(1001) never change from genesis
lock_id(1002) never change from genesis
lock_id(1003) never change from genesis
lock_id(1004) never change from genesis
lock_id(1005) never change from genesis
lock_id(1006) never change from genesis
```

