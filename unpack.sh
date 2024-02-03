#!/bin/bash

rm -rf ./tmp
mkdir tmp

mv .git/objects/pack/* tmp/
git unpack-objects < tmp/*.pack
