SupraSeal demo
==============

These instructions are about 512MiB sectors as you likely want to try things out before running it on production sizes.


Quickstart
----------

### Building

Build the C++ based Supraseal base library first. Run it from the root directory of this repository (not this one):

```
./build.sh 512MiB
```

Then build the Rust-based binaries (from the directory this README is in):

```
cargo run --release --bins
```


### Running PC1/PC2

There is an example script at [scripts/pc1pc2_cc.sh] that runs the Precommit 1 and Precommit 2 phase for CC sectors.

```console
> echo '{"output_dir": "/tmp/out", "parents_cache_path":"/var/tmp/filecoin-parents/v28-sdr-parent-7ba215a1d2345774ab90b8cb1158d296e409d6068819d7b8c7baf0b25d63dc34.cache","replica_ids":["0xd93f7c0618c236179361de2164ce34ffaf26ecf3be7bf7e6b8f0cfcf886ad0d0","0x516de970419d50c025f57ed6eb1135278aca99d2d2a27017e54bc43580389478"],"supraseal_config_path":"/path/to/supra_seal.cfg"}'|./scripts/pc1_pc2_cc.sh
```


Running on AWS
--------------

Server requirements:
 - Nvidia GPU (e.g. Tesla T4)
 - CPU with SHA extension (e.g. AMD EPYC)
 - >= 24GiB RAM
 - An additional NVMe drive that is not your root filesystem (> 128 GiB)

For the following instructions a [g5.2xlarge] instance was used.

### Setup the system

For the CUDA installation, it's a requirement that the Linux kernel was compiled with the same version of GCC as the currently installed default compiler. Hence upgrade the kernel. Then install all the additional packages we need and finally reboot.

```
sudo apt dist-upgrade
sudo apt install jo moreutils
sudo reboot
```

### CUDA

Get the current CUDA toolkit from https://developer.nvidia.com/cuda-downloads?target_os=Linux&target_arch=x86_64&Distribution=Ubuntu&target_version=22.04&target_type=runfile_local and install it:

```
wget https://developer.download.nvidia.com/compute/cuda/12.2.1/local_installers/cuda_12.2.1_535.86.10_linux.run
sudo sh cuda_12.2.1_535.86.10_linux.run -- --silent --driver --toolkit
```

So that you don't need to adapt all commands, make sure that CUDA is found when run or used for compilation:

```
echo '/usr/local/cuda/lib64'|sudo tee /etc/ld.so.conf.d/cuda.conf
sudo ldconfig
echo $'\nPATH="/usr/local/cuda/bin:$PATH"' >> ~/.profile
echo 'export LIBRARY_PATH="/usr/local/cuda/targets/x86_64-linux/lib:$LIBRARY_PATH"' >> ~/.profile
```


### Supraseal

Get Rust:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable --profile minimal -y
```

Enable huge pages:

```
sudo vi /etc/default/grub
GRUB_CMDLINE_LINUX_DEFAULT="default_hugepagesz=1G hugepagesz=1G hugepages=18"
GRUB_CMDLINE_LINUX="default_hugepagesz=1G hugepagesz=1G hugepages=18"
sudo update-grub
sudo reboot
```

Verify that that hugepages are set correctly:

```console
> grep '^Huge' /proc/meminfo
HugePages_Total:      18
HugePages_Free:       18
HugePages_Rsvd:        0
HugePages_Surp:        0
Hugepagesize:    1048576 kB
Hugetlb:        37748736 kB
```

Get Supraseal and build it:

```
git clone https://github.com/supranational/supra_seal
cd supra_seal
git remote add vmx https://github.com/vmx/supra_seal
git fetch vmx && git checkout vmx/binaries
# This installs some python packages globally if you don't run it in a virtualenv
./build.sh 512MiB
```

The next command sets SPDK up, it needs to be run after **every** reboot:

```
sudo env NRHUGE=18 ./deps/spdk-v22.09/scripts/setup.sh
```

If you get an error message like:

```
No valid drivers found [vfio-pci, uio_pci_generic, igb_uio]. Please enable one of the kernel modules.
```

Then IOMMU is enabled. That might happen on AWS, I haven't found a way to disable IOMMU there (the `iommu=off` and `amd_iommu=off`/`intel_iommu=off` kernel parameters didn't make a difference). In order to work around that, you can [enable the vfio no-iommu mode](https://doc.dpdk.org/guides/linux_gsg/linux_drivers.html#vfio-noiommu):

```
echo 'options vfio enable_unsafe_noiommu_mode=1'|tee sudo /etc/modprobe.d/vfio-noiommu.conf
echo 'vfio-pci'|sudo tee /etc/modules-load.d/vfio-pci.conf'
sudo modprobe vfio-pci
```

If you run the SPDK setup again, it should now contain something like this:

```console
> sudo env NRHUGE=36 ./deps/spdk-v22.09/scripts/setup.sh
…
0000:00:1f.0 (1d0f cd01): nvme -> vfio-pci
…
```

Identify device to use. The `Driver` should be `vfio-pci`.

```console
> sudo ./deps/spdk-v22.09/scripts/setup.sh status
Hugepages
node     hugesize     free /  total
node0   1048576kB       36 /     36
node0      2048kB        0 /      0

Type     BDF             Vendor Device NUMA    Driver           Device     Block devices
NVMe     0000:00:04.0    1d0f   8061   0       nvme             nvme0      nvme0n1
NVMe     0000:00:1f.0    1d0f   cd01   0       vfio-pci         -          -
> cd ..
```

Edit `demos/rust/supra_seal.cfg` and add your NVMe devices e.g. `0000:00:1f.0`.


### Sealing setup

Get the parameters for 512MiB sectors:

```
mkdir /var/tmp/filecoin-proof-parameters
wget --directory-prefix=/var/tmp/filecoin-proof-parameters https://proofs.filecoin.io/v28-stacked-proof-of-replication-merkletree-poseidon_hasher-8-0-0-sha256_hasher-6babf46ce344ae495d558e7770a585b2382d54f225af8ed0397b8be7c3fcd472.vk
wget --directory-prefix=/var/tmp/filecoin-proof-parameters https://proofs.filecoin.io/v28-stacked-proof-of-replication-merkletree-poseidon_hasher-8-0-0-sha256_hasher-6babf46ce344ae495d558e7770a585b2382d54f225af8ed0397b8be7c3fcd472.params
```

Generate the parents cache file for a 512MiB sector in `/var/tmp/filecoin-parents/`:

```
git clone https://github.com/filecoin-project/rust-fil-proofs
cd rust-fil-proofs
cargo run --release --bin gen_graph_cache -- --size 536870912
cd ..
```


### Run pc1pc2

Compile all binaries first. It's only needed once, you could even skip it, but that makes the output of the script below less confusing.

```
cd supraseal/demos/rust
cargo build --release --bins
```

Now actually run it for two 512MiB CC sectors:

```
echo '{"output_dir": "/tmp/512mib2sec", "parents_cache_path":"/var/tmp/filecoin-parents/v28-sdr-parent-7ba215a1d2345774ab90b8cb1158d296e409d6068819d7b8c7baf0b25d63dc34.cache","replica_ids":["0x8229407e385f82a7dd85c4ff7bc8488deb79e59c39d6a5b5ed9ab5e0762f6d3f","0x516de970419d50c025f57ed6eb1135278aca99d2d2a27017e54bc43580389478"],"supraseal_config_path":"/home/ubuntu/supra_seal/demos/rust/supra_seal.cfg"}'|sudo --preserve-env env PATH="${PATH}" /usr/bin/time -v ./scripts/pc1_pc2_cc.sh|ts '[%Y-%m-%d %H:%M:%S]'|tee ../../../pc1pc2_512mibgib2sec_001.log
```

The time it takes depends highly on the performance of your NVMe. You can test your performance via:

```
> sudo ./deps/spdk-v22.09/build/examples/perf --allowed-pci-addr 0000:00:1f.0 --io-depth 64 --io-size 4096 --io-pattern randrw --rwmixread 50 --time
…
Device Information                     :       IOPS      MiB/s    Average        min        max
PCIE (0000:00:1f.0) NSID 1 from core  0:  137788.90     538.24     464.48      58.50    1729.93
========================================================
Total                                  :  137788.90     538.24     464.48      58.50    1729.93
```

The PC1 for two 512MiB sectors needs to read/write about 800GiB. With a performance of about 450MiB/s, it then takes about 30mins.

[scripts/pc1pc2_cc.sh]: ./scripts/pc1_pc2_cc.sh
[g5.2xlarge]: https://instances.vantage.sh/aws/ec2/g5.2xlarge
