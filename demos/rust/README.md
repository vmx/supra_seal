SupraSeal demo
==============

Building
--------

Run the [./build.sh] from the root directory of this repo.


Running PC1/PC2
---------------

There is an example script at [scripts/pc1pc2_cc.sh] that runs the Precommit 1 and Precommit 2 phase for CC sectors.

```console
> echo '{"output_dir": "/tmp/out", "parents_cache_path":"/var/tmp/filecoin-parents/v28-sdr-parent-55c7d1e6bb501cc8be94438f89b577fddda4fafa71ee9ca72eabe2f0265aefa6.cache","replica_ids":["0xd93f7c0618c236179361de2164ce34ffaf26ecf3be7bf7e6b8f0cfcf886ad0d0","0x516de970419d50c025f57ed6eb1135278aca99d2d2a27017e54bc43580389478"],"supraseal_config_path":"/path/to/supra_seal.cfg"}'|./scripts/pc1_pc2_cc.sh
```


[build.sh]: ../../build.sh
[scripts/pc1pc2_cc.sh]: ./scripts/pc1_pc2_cc.sh
