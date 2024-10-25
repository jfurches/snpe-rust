#!/bin/bash
cd $1
source bin/envsetup.sh

if [ $? -ne 0 ]; then
    exit 1
fi

env
