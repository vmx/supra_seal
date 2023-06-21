#!/bin/bash

# Copyright Supranational LLC

set -e
set -x

SECTOR_SIZE="" # Compile for all sector sizes
while getopts r flag
do
    case "${flag}" in
        r) SECTOR_SIZE="-DRUNTIME_SECTOR_SIZE";;
    esac
done

CC=${CC:-cc}
CXX=${CXX:-c++}
NVCC=${NVCC:-nvcc}

CUDA_ARCH="-arch=sm_80 -gencode arch=compute_70,code=sm_70 -t0"
CXXSTD=`$CXX -dM -E -x c++ /dev/null | \
        awk '{ if($2=="__cplusplus" && $3<"2017") print "-std=c++17"; }'`

# Check for the default result directory
if [ ! -d "/tmp/supra_seal" ]; then
    mkdir -p /tmp/supra_seal
fi

rm -fr obj
mkdir -p obj

rm -fr bin
mkdir -p bin

mkdir -p deps
if [ ! -d "deps/sppark" ]; then
    git clone https://github.com/supranational/sppark.git deps/sppark
fi
if [ ! -d "deps/blst" ]; then
    git clone https://github.com/supranational/blst.git deps/blst
    (cd deps/blst
     ./build.sh -march=native)
fi

# Generate .h files for the Poseidon constants
xxd -i poseidon/constants/constants_2  > obj/constants_2.h
xxd -i poseidon/constants/constants_4  > obj/constants_4.h
xxd -i poseidon/constants/constants_8  > obj/constants_8.h
xxd -i poseidon/constants/constants_11 > obj/constants_11.h
xxd -i poseidon/constants/constants_16 > obj/constants_16.h
xxd -i poseidon/constants/constants_24 > obj/constants_24.h
xxd -i poseidon/constants/constants_36 > obj/constants_36.h

# tree-r CPU only
$CXX $SECTOR_SIZE $CXXSTD -pthread -g -O3 -march=native \
    -Wall -Wextra -Werror -Wno-subobject-linkage \
    tools/tree_r.cpp poseidon/poseidon.cpp \
    -o bin/tree_r_cpu -Iposeidon -Ideps/sppark -Ideps/blst/src -L deps/blst -lblst

# tree-r CPU + GPU
$NVCC $SECTOR_SIZE -DNO_SPDK -DSTREAMING_NODE_READER_FILES \
     $CUDA_ARCH -std=c++17 -g -O3 -Xcompiler -march=native \
     -Xcompiler -Wall,-Wextra,-Werror \
     -Xcompiler -Wno-subobject-linkage,-Wno-unused-parameter \
     -x cu tools/tree_r.cpp -o bin/tree_r \
     -Iposeidon -Ideps/sppark -Ideps/sppark/util -Ideps/blst/src -L deps/blst -lblst -lconfig++

# Standalone GPU pc2
$NVCC $SECTOR_SIZE -DNO_SPDK -DSTREAMING_NODE_READER_FILES \
     $CUDA_ARCH -std=c++17 -g -O3 -Xcompiler -march=native \
     -Xcompiler -Wall,-Wextra,-Werror \
     -Xcompiler -Wno-subobject-linkage,-Wno-unused-parameter \
     -x cu tools/tree_r.cpp -o bin/tree_r \
     -Iposeidon -Ideps/sppark -Ideps/sppark/util -Ideps/blst/src -L deps/blst -lblst -lconfig++

# Standalone GPU pc2
$NVCC $SECTOR_SIZE -DNO_SPDK -DSTREAMING_NODE_READER_FILES \
     $CUDA_ARCH -std=c++17 -g -O3 -Xcompiler -march=native \
     -Xcompiler -Wall,-Wextra,-Werror \
     -Xcompiler -Wno-subobject-linkage,-Wno-unused-parameter \
     -x cu tools/pc2.cu -o bin/pc2 \
     -Iposeidon -Ideps/sppark -Ideps/sppark/util -Ideps/blst/src -L deps/blst -lblst -lconfig++
