#!/bin/bash

JAR="./generator/target/scala-2.12/Benchmark Suite Visualisation Generator-assembly-1.0.0.jar"

if [ ! -f "$JAR" ]; then
    sbt generator/assembly
fi

java -jar "$JAR"  $@
