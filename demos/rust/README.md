SupraSeal demo
==============

These instructions are about 512MiB sectors as you likely want to try things out before running it on production sizes.


Building
--------

Build the C++ based Supraseal base library first. Run it from the root directory of this repository (not this one):

```
./build.sh 512MiB
```

Then build the Rust-based binaries (from the directory this README is in):

```
cargo run --release --bins
```


Running PC1/PC2
---------------

There is an example script at [scripts/pc1pc2_cc.sh] that runs the Precommit 1 and Precommit 2 phase for CC sectors.

```console
> echo '{"output_dir": "/tmp/out", "parents_cache_path":"/var/tmp/filecoin-parents/v28-sdr-parent-7ba215a1d2345774ab90b8cb1158d296e409d6068819d7b8c7baf0b25d63dc34.cache","replica_ids":["0xd93f7c0618c236179361de2164ce34ffaf26ecf3be7bf7e6b8f0cfcf886ad0d0","0x516de970419d50c025f57ed6eb1135278aca99d2d2a27017e54bc43580389478"],"supraseal_config_path":"/path/to/supra_seal.cfg"}'|./scripts/pc1_pc2_cc.sh
```

[scripts/pc1pc2_cc.sh]: ./scripts/pc1_pc2_cc.sh
