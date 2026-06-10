libflatpakcli is a CLI for flatpak with machine readable output.

you may run `libflatpakcli end-points`:
```
Commands:
    install         <user/system> <remote> <reference>    -> List<MyFlatpakTransactionOperation> followed by NULL, followed by progress percentage of each operation delimited by NULL.
    remove          <user/system> <reference>             -> List<MyFlatpakTransactionOperation> followed by NULL, followed by progress percentage of each operation delimited by NULL.
    upgrade         <user/system> <reference>             -> List<MyFlatpakTransactionOperation> followed by NULL, followed by progress percentage of each operation delimited by NULL.
    update-cache    <user/system> <remote>
    list-installed  <user/system>                         -> List<reference : String>
    list-upgradable <user/system>                         -> List<reference : String>
    appstream-path  <user/system> <remote>                -> path : String
    add-remote      <user/system> <name> <url>               !!UNTESTED!!
    remove-remote   <user/system> <name>                     !!UNTESTED!!
    remotes         <user/system>                         -> List<FlatpakRemote>
    info            <user/system> <remote> <reference>    -> MyFlatpakRemoteRefInfo
```