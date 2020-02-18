#!/usr/bin/env bash
#nohup ./bench.sc client "$@" &> /dev/null &
str="'$*'"
./bench.sc compile
nohup ./bench.sc client "$@" &> nohup-"$str".out &
#nohup amm bench.sc client "$@"; echo "test" &> nohup-"$str".out &
echo $!
