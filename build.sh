#!/bin/sh

set -xe

# https://stackoverflow.com/a/61181216/19856457
jimage extract --dir=stdlib $JAVA_HOME/lib/modules
