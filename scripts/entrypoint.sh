#!/bin/bash
#
# SNPE container entrypoint
#

export PATH=${SNPE_ROOT}/bin/x86_64-linux-clang:${PATH}

source /root/.profile

if [ ! -f "${SNPE_ROOT}/bin/envsetup.sh" ]; then
    echo -e "\033[1;31mERROR: \033[0mSNPE directory \033[0;35m${SNPE_ROOT} \033[0mdoesn't contain \033[0;34mbin/envsetup.sh\033[0m!"
    echo -e "\033[0mYou must specify your SNPE binaries directory (top folder including models/ LICENSE.pdf, etc)"
    echo -e "\033[0mas docker mountpoint at \033[0;35m${SNPE_ROOT}\033[0m (e.g. \033[0;37mdocker run ... -v <local_path_to_snpe>:${SNPE_ROOT}\033[0m)"
    exit 1;
fi

# Start bash with user-provided command arguments
source ${SNPE_ROOT}/bin/envsetup.sh
/bin/bash -- $@
