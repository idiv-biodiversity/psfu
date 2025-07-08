psfu
====

Process Fu. Do stuff with processes.


Usage
-----

`psfu` currently has one major command called `tree` which is used to `show` or
`modify` all processes in that process tree.

### show commands

Show plain process tree for current shell session:

```console
$ psfu tree show plain --arguments $$
3772 -bash
└── 109966 psfu tree show plain --arguments 3772
```

Other show commands are:

- **affinity**: show CPU affinity aka core binding
- **backtrace**: show process/thread traces (using `gdb`, may require admin
  privileges)
- **nice**: show niceness

### modify commands

Modify commands are:

- **affinity**: modify CPU affinity
- **nice**: modify niceness

### tips

1.  Commands can be abbreviated to save on typing, as long as they are
    unambiguous:

    ```bash
    # these are the same
    psfu tree show affinity
    psfu t s a
    ```

1.  Use piped input to show forest or to modify multiple process trees:

    ```console
    $ pgrep 'tmux: server' | psfu t s a
    1459 tmux: server [0, 1, 2, 3]
    ├── 1460 bash [0, 1, 2, 3]
    │   └── 3764 emacs [0, 1, 2, 3]
    │       ├── 108151 aspell [0, 1, 2, 3]
    │       └── 111340 rust-analyzer [0, 1, 2, 3]
    │           └── 111370 rust-analyzer [0, 1, 2, 3]
    └── 3772 bash [0, 1, 2, 3]
        └── 114364 psfu [0, 1, 2, 3]

    $ pgrep emacs | psfu t m a 0

    $ pgrep 'tmux: server' | psfu t s a
    1459 tmux: server [0, 1, 2, 3]
    ├── 1460 bash [0, 1, 2, 3]
    │   └── 3764 emacs [0]
    │       ├── 108151 aspell [0]
    │       └── 111340 rust-analyzer [0]
    │           └── 111370 rust-analyzer [0]
    └── 3772 bash [0, 1, 2, 3]
        └── 114399 psfu [0, 1, 2, 3]
    ```


Installation
------------

### Arch Linux

Install the [psfu AUR package][aur-package]:

```bash
pacaur -S psfu
```

### cargo install

```bash
cargo install psfu
```

### from source

```bash
git clone https://github.com/idiv-biodiversity/psfu.git
cd psfu
cargo build --release
install -Dm755 target/release/psfu ~/bin/psfu
```


[aur-package]: https://aur.archlinux.org/packages/psfu "psfu AUR package"
