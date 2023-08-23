#!/bin/sh
set -eu
#set -o xtrace


# Usage help if nothing is piped in.
if [ -t 0 ]; then
    cat << EOF
Usage: echo '{}' | $(basename "${0}")

Perform a PreCommit 1 and PreCommit 2 for a single sector.

It prints to stdout the result CommR formatted as JSON, e.g.
{"comm_r":"0x9dabeaa4e2b53153152ac485c6b8ede4d750be12d0fae4fa265161dc0ff5502a"}

The input parameters are given by piping in JSON with the following keys:
 - output_dir: The directory where all files (e.g. the trees) are stored.
 - parents_cache_path: The path to the parents cache file. For 32GiB sectors, that's usually at
   /var/tmp/filecoin-parents/v28-sdr-parent-55c7d1e6bb501cc8be94438f89b577fddda4fafa71ee9ca72eabe2f0265aefa6.cache.
 - replica_ids: A list of Replica IDs formatted in hex with leading 0x.
 - supraseal_config_path: The path to the Supraseal configuration file.

Example JSON:
{
  "output_dir": "/path/to/some/dir",
  "parents_cache_path": "/path/to/filecoin-parents/v28-sdr-parent-abcdef.cache",
  "replica_ids": [
    "0xd93f7c0618c236179361de2164ce34ffaf26ecf3be7bf7e6b8f0cfcf886ad0d0",
    "0x516de970419d50c025f57ed6eb1135278aca99d2d2a27017e54bc43580389478"
  ],
  "supraseal_config_path": "/path/to/supra_seal.cfg"
}
EOF
     exit 1
fi


# Define default options for commands
CARGO="${CARGO:=cargo run --release}"
JQ='jq -r'
JO='jo --'

export RUST_LOG=trace


# Make sure all tools we need for this scripts are installed.
if ! command -v jq > /dev/null
then
    echo "'jq' not found." && exit 2
fi
if ! command -v jo > /dev/null
then
    echo "'jo' not found." && exit 3
fi


# Parse the input data.
read -r input_args
output_dir=$(echo "${input_args}" | ${JQ} '.output_dir')
parents_cache_path=$(echo "${input_args}" | ${JQ} '.parents_cache_path')
# Parse Replica ID's in a format that `jo` understands: '-s 0xaa -s 0xbb'
replica_ids=$(echo "${input_args}" | ${JQ} '.replica_ids | @sh' | tr -d "'" | sed -e 's/0x/-s 0x/g')
num_sectors=$(echo "${input_args}" | ${JQ} '.replica_ids | length')
supraseal_config_path=$(echo "${input_args}" | ${JQ} '.supraseal_config_path')

if [ "${output_dir}" = 'null' ]; then
    echo "'output_dir' not set." && exit 4
fi
if [ "${parents_cache_path}" = 'null' ]; then
    echo "'parents_cache_path' not set." && exit 5
fi
if [ "${replica_ids}" = 'null' ]; then
    echo "'replica_ids' not set." && exit 6
fi
if [ "${supraseal_config_path}" = 'null' ]; then
    echo "'supraseal_config_path' not set." && exit 7
fi

mkdir -p "${output_dir}"

# Run SDR.
# shellcheck disable=SC2086 # replica_ids should be split.
${JO} -s parents_cache_path="${parents_cache_path}" replica_ids="$(jo -a -- ${replica_ids})" -s supraseal_config_path="${supraseal_config_path}" | ${CARGO} --bin sdr

# Run PC2.
${JO} num_sectors="${num_sectors}" -s output_dir="${output_dir}" -s supraseal_config_path="$supraseal_config_path" | ${CARGO} --bin pc2-cc
