# cloud-op

`cita-cloud`的运维工具，可以对CITA-Cloud节点数据进行各种操作。

`cita-cloud`节点数据包含：

1. `storage`微服务中存储区块，交易等数据的`storage_db`。
2. 如果使用`storage_opendal`并配置了`cloud_storage`。用于存储共享历史数据的对象存储。
3. `executor_evm`微服务中存储`evm`状态的`statedb`。

对这些数据可以进行的操作有：

1. `rollback` 回滚。
    
    当节点因为断电等意外导致本节点数据与链上其他节点的数据不一致时，可以通过回滚本节点到之前一致时的高度进行恢复。
    
    该操作会同时回滚`storage_db`和`statedb`到指定的高度。

    但是不会操作`cloud_storage`，当然对于这种情况也不需要操作`cloud_storage`。机制保证了不一致的数据不会写入`cloud_storage`。

2. `cloud-rollback` 云存储回滚。

    当链上出现了预期之外的交易，需要链上所有节点都回滚的时候。这种情况比较罕见，请一定要事先确认好。

    如果使用`storage_opendal`并配置了`cloud_storage`。不但要对所有节点进行回滚操作，还需要对云存储进行回滚操作。

    该操作单独将云存储回滚到指定的高度。

3. `backup` 备份。

    出于可靠性的考虑，建议定时对链上数据进行备份。以便将来可以直接从备份中恢复节点的数据。
    
    该操作会同时将`storage_db`和`statedb`备份到指定的路径。

    将来恢复时直接将备份数据拷贝到节点目录下即可。

    备份操作又分两种：一种是直接拷贝数据库文件，并将备份数据回滚到指定的高度，以避免数据处于中间状态；另外一种是新建空的备份数据库，将现有数据库中的数据导出，重新导入新的备份数据库。

    适用于：
    
    a. 新增节点时快速同步，新增节点无需从头同步区块，直接达到比较接近最新的高度。

    b. 节点存储损坏，导致节点数据全部丢失。

    c. 版本升级过程中存储格式发生变化，新老版本不兼容。这时要采用导出/导入的方式进行备份，同时完成了格式转换。

```shell
$ cloud-op --help
cloud-op to operate data of cita-cloud node

Usage: cloud-op <COMMAND>

Commands:
  rollback        rollback chain status to specified height, ONLY USE IN EVM MODE
  cloud-rollback  rollback cloud storage status to specified height, ONLY USE IN EVM MODE
  backup          backup executor and storage data of a specified height
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## 使用

本工具为`cita-cloud`节点数据的运维工具，需要`Attach`到被操作的节点上，继承节点的配置等信息才能使用。

因此，下文中的配置文件`config.toml`均指被操作的节点的配置文件，`node-root`是指被操作节点的数据所在的路径。

### Rollback

```shell
$ cloud-op rollback -h
rollback chain status to specified height

Usage: cloud-op rollback [OPTIONS] <HEIGHT>

Arguments:
  <HEIGHT>  the specified height that you want to rollback to

Options:
  -c, --config-path <CONFIG_PATH>  chain config path [default: config.toml]
  -n, --node-root <NODE_ROOT>      node root path [default: .]
      --clean                      whether to clean consensus data
  -h, --help                       Print help
```

注意：`--clean`开关仅在前述第二种情况--需要链上所有节点都回滚的时候--才需要打开。这种情况比较罕见，请一定要事先确认好。

### cloud-rollback

```shell
$ cloud-op cloud-rollback -h
rollback cloud storage status to specified height

Usage: cloud-op cloud-rollback [OPTIONS] <HEIGHT>

Arguments:
  <HEIGHT>  the specified height that you want to rollback to

Options:
  -c, --config-path <CONFIG_PATH>  chain config path [default: config.toml]
  -n, --node-root <NODE_ROOT>      node root path [default: .]
  -h, --help                       Print help
```

### backup

```shell
$ cloud-op backup -h
backup executor and storage data of a specified height

Usage: cloud-op backup [OPTIONS] <HEIGHT>

Arguments:
  <HEIGHT>  backup height

Options:
  -c, --config-path <CONFIG_PATH>  chain config path [default: config.toml]
  -n, --node-root <NODE_ROOT>      node root path [default: .]
  -b, --backup-path <BACKUP_PATH>  backup path dir [default: backup]
      --export                     whether to export database or copy database
  -h, --help                       Print help
```

注意：`--export`开关仅在前述`3.c`情况才需要打开，耗时比不加的时候要长，请一定要事先确认好。

## 示例：

#### rollback

```shell
$ cloud-op rollback -c config.toml -n . 190000
current height: 197292
rollback height: 190000
lock_id(1000) never change from genesis
lock_id(1001) never change from genesis
lock_id(1002) keep change
lock_id(1003) keep change
lock_id(1004) keep change
lock_id(1005) keep change
lock_id(1006) never change from genesis
lock_id(1007) keep change
storage rollback done!

executor rollback done
```

### backup

对节点做备份(直接copy)

```shell
$ cloud-op backup -c config.toml -n . -b /tmp/backup/ 190000
current height: 197451         
backup height: 190000         
backup excutor state done!    
copy excutor chain_db done!
copy storage chain_data done!                                                                                                          
executor rollback done!   
lock_id(1000) never change from genesis
lock_id(1001) never change from genesis
lock_id(1002) keep change 
lock_id(1003) keep change 
lock_id(1004) keep change
lock_id(1005) keep change
lock_id(1006) never change from genesis                                                                                                
lock_id(1007) keep change
storage rollback done!
backup done!

# tree /tmp/backup/ -L 2 
/tmp/backup/                            
└── 190000                              
    ├── chain_data                                                                                                                     
    └── data
```

对节点做备份(导入导出方式)

```shell
$ cloud-op backup -c config.toml -n . -b /tmp/backup/ --export 190000
current height: 198260
backup height: 190000
exporting: 8/8
export stat done!
export excutor state done!
copy excutor chain_db done!
exporting: 190000/190000
export block done!
export utxo done!
executor rollback done!
lock_id(1000) never change from genesis
lock_id(1001) never change from genesis
lock_id(1002) keep change
lock_id(1003) keep change
lock_id(1004) keep change
lock_id(1005) keep change
lock_id(1006) never change from genesis
lock_id(1007) keep change
storage rollback done!
backup done!

# tree /tmp/backup/ -L 2 
/tmp/backup/                            
└── 190000                              
    ├── chain_data                                                                                                                     
    └── data
```

从备份恢复

```shell
$ rm -rf chain_data/ data/
$ cp -r /tmp/backup/190000/* ./
```
