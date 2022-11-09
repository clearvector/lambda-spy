#!/bin/bash

BASENAME="$(basename $(pwd))"
EXT_DIR="./build/extensions"
BIN_DIR="./build/${BASENAME}"


if [ ! -d ${EXT_DIR} ]
then
    mkdir -p ${EXT_DIR}
fi
if [ ! -d ${BIN_DIR} ]
then
    mkdir -p ${BIN_DIR}
fi

mkdir "${BIN_DIR}/x86_64"
mkdir "${BIN_DIR}/arm64"

cp "bootstrap.sh" "${EXT_DIR}/${BASENAME}"
cargo lambda build --target x86_64-unknown-linux-musl --release --extension
mv "./target/lambda/extensions/lambda-spy" "${BIN_DIR}/x86_64/${BASENAME}"
cargo lambda build --target aarch64-unknown-linux-musl --release --extension
mv "./target/lambda/extensions/lambda-spy" "${BIN_DIR}/arm64/${BASENAME}"

rm "./target/lambda/extensions/${BASENAME}.zip"
cd "${EXT_DIR}/.."
zip -r "../target/lambda/extensions/${BASENAME}.zip" .
